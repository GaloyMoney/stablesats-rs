mod adjust_funding;
mod poll_transfers;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::instrument;
use uuid::{uuid, Uuid};

use std::collections::HashMap;

use galoy_client::GaloyClient;
use okex_client::OkexClient;
use shared::{
    pubsub::{CorrelationId, Publisher},
    sqlxmq::JobExecutor,
};

use crate::{error::*, okex_transfers::OkexTransfers, synth_usd_liability::*};

const _POLL_TRANSFER_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000002");

#[derive(Debug, Clone)]
struct OkexPollDelay(std::time::Duration);

pub async fn start_job_runner(
    pool: sqlx::PgPool,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_transfers: OkexTransfers,
    galoy: GaloyClient,
    publisher: Publisher,
    delay: std::time::Duration,
) -> Result<OwnedHandle, FundingError> {
    let mut registry = JobRegistry::new(&[adjust_funding]);
    registry.set_context(synth_usd_liability);
    registry.set_context(okex);
    registry.set_context(publisher);
    registry.set_context(OkexPollDelay(delay));
    registry.set_context(okex_transfers);
    registry.set_context(galoy);

    Ok(registry.runner(&pool).run().await?)
}

#[instrument(skip_all, fields(error, error.level, error.message), err)]
pub async fn spawn_poll_transfers(
    pool: &sqlx::PgPool,
    duration: std::time::Duration,
) -> Result<(), FundingError> {
    match JobBuilder::new_with_id(_POLL_TRANSFER_ID, "poll_transfers")
        .set_delay(duration)
        .spawn(pool)
        .await
    {
        Err(sqlx::Error::Database(err)) if err.message().contains("duplicate key") => Ok(()),
        Err(e) => {
            shared::tracing::insert_error_fields(tracing::Level::ERROR, &e);
            Err(e.into())
        }
        Ok(_) => Ok(()),
    }
}

#[derive(Serialize, Deserialize)]
struct AdjustFundingData {
    correlation_id: CorrelationId,
    #[serde(flatten)]
    tracing_data: HashMap<String, String>,
}

#[instrument(skip_all, fields(error, error.message) err)]
pub async fn spawn_adjust_funding<'a>(
    tx: impl Executor<'a, Database = Postgres>,
    correlation_id: CorrelationId,
) -> Result<(), FundingError> {
    match JobBuilder::new_with_id(Uuid::from(correlation_id), "adjust_funding")
        .set_json(&AdjustFundingData {
            tracing_data: shared::tracing::extract_tracing_data(),
            correlation_id,
        })
        .expect("Couldn't set json")
        .spawn(tx)
        .await
    {
        Err(sqlx::Error::Database(err)) if err.message().contains("duplicate key") => Ok(()),
        Err(e) => {
            shared::tracing::insert_error_fields(tracing::Level::ERROR, &e);
            Err(e.into())
        }
        Ok(_) => Ok(()),
    }
}

#[job(name = "poll_transfers", channel_name = "funding")]
async fn poll_okex(
    mut current_job: CurrentJob,
    OkexPollDelay(delay): OkexPollDelay,
    okex: OkexClient,
    okex_transfers: OkexTransfers,
) -> Result<(), FundingError> {
    JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|_| async move { poll_transfers::_execute(okex_transfers, okex).await })
        .await?;
    spawn_poll_transfers(current_job.pool(), delay).await?;
    Ok(())
}

#[job(name = "adjust_funding", channel_name = "funding", ordered)]
async fn adjust_funding(
    mut current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_transfers: OkexTransfers,
    galoy: GaloyClient,
) -> Result<(), FundingError> {
    JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|data| async move {
            let data: AdjustFundingData = data.expect("no AdjustFundingData available");
            adjust_funding::execute(
                data.correlation_id,
                synth_usd_liability,
                okex,
                okex_transfers,
                galoy,
            )
            .await?;
            Ok::<_, FundingError>(data)
        })
        .await?;
    Ok(())
}
