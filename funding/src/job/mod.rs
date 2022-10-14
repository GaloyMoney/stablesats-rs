mod adjust_funding;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::instrument;
use uuid::Uuid;

use std::collections::HashMap;

use galoy_client::GaloyClient;
use okex_client::OkexClient;
use shared::{
    pubsub::{CorrelationId, Publisher},
    sqlxmq::JobExecutor,
};

use crate::{error::*, okex_transfers::OkexTransfers, synth_usd_liability::*};

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

#[derive(Serialize, Deserialize)]
struct AdjustHedgeData {
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
        .set_json(&AdjustHedgeData {
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
            let data: AdjustHedgeData = data.expect("no AdjustHedgeData available");
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
