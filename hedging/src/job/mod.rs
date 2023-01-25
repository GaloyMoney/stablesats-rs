mod adjust_funding;
mod adjust_hedge;
mod poll_okex;

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

use crate::{
    adjustment_action::*, app::FundingSectionConfig, error::*, okex_orders::OkexOrders,
    okex_transfers::OkexTransfers, rebalance_action::*, synth_usd_liability::*,
};

pub const POLL_OKEX_ID: Uuid = uuid!("10000000-0000-0000-0000-000000000001");

#[derive(Debug, Clone)]
struct OkexPollDelay(std::time::Duration);

#[allow(clippy::too_many_arguments)]
pub async fn start_job_runner(
    pool: sqlx::PgPool,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_orders: OkexOrders,
    okex_transfers: OkexTransfers,
    galoy: GaloyClient,
    publisher: Publisher,
    delay: std::time::Duration,
    funding_adjustment: FundingAdjustment,
    hedging_adjustment: HedgingAdjustment,
    funding_config: FundingSectionConfig,
) -> Result<OwnedHandle, HedgingError> {
    let mut registry = JobRegistry::new(&[adjust_hedge, poll_okex, adjust_funding]);
    registry.set_context(synth_usd_liability);
    registry.set_context(okex);
    registry.set_context(okex_orders);
    registry.set_context(okex_transfers);
    registry.set_context(galoy);
    registry.set_context(publisher);
    registry.set_context(OkexPollDelay(delay));
    registry.set_context(funding_adjustment);
    registry.set_context(hedging_adjustment);
    registry.set_context(funding_config);

    Ok(registry.runner(&pool).run().await?)
}

#[instrument(skip_all, fields(error, error.level, error.message), err)]
pub async fn spawn_poll_okex(
    pool: &sqlx::PgPool,
    duration: std::time::Duration,
) -> Result<(), HedgingError> {
    match JobBuilder::new_with_id(POLL_OKEX_ID, "poll_okex")
        .set_channel_name("poll_okex")
        .set_retries(6)
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
struct AdjustHedgeData {
    correlation_id: CorrelationId,
    #[serde(flatten)]
    tracing_data: HashMap<String, String>,
}

#[instrument(skip_all, fields(error, error.message) err)]
pub async fn spawn_adjust_hedge<'a>(
    tx: impl Executor<'a, Database = Postgres>,
    correlation_id: CorrelationId,
) -> Result<(), HedgingError> {
    match adjust_hedge
        .builder()
        .set_ordered(true)
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

#[job(name = "poll_okex")]
async fn poll_okex(
    mut current_job: CurrentJob,
    OkexPollDelay(delay): OkexPollDelay,
    okex: OkexClient,
    okex_orders: OkexOrders,
    okex_transfers: OkexTransfers,
    publisher: Publisher,
    funding_config: FundingSectionConfig,
) -> Result<(), HedgingError> {
    JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|_| async move {
            poll_okex::execute(okex_orders, okex_transfers, okex, publisher, funding_config).await
        })
        .await?;
    spawn_poll_okex(current_job.pool(), delay).await?;
    Ok(())
}

#[job(
    name = "adjust_hedge",
    channel_name = "adjust_hedging",
    ordered,
    retries = 6
)]
async fn adjust_hedge(
    mut current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_orders: OkexOrders,
    hedging_adjustment: HedgingAdjustment,
) -> Result<(), HedgingError> {
    JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|data| async move {
            let data: AdjustHedgeData = data.ok_or(HedgingError::NoJobDataPresent)?;
            adjust_hedge::execute(
                data.correlation_id,
                synth_usd_liability,
                okex,
                okex_orders,
                hedging_adjustment,
            )
            .await?;
            Ok::<_, HedgingError>(data)
        })
        .await?;
    Ok(())
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
) -> Result<(), HedgingError> {
    match JobBuilder::new_with_id(Uuid::from(correlation_id), "adjust_funding")
        .set_channel_name("adjust_funding")
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

#[job(name = "adjust_funding")]
async fn adjust_funding(
    mut current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_transfers: OkexTransfers,
    galoy: GaloyClient,
    funding_adjustment: FundingAdjustment,
) -> Result<(), HedgingError> {
    JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|data| async move {
            let data: AdjustFundingData = data.ok_or(HedgingError::NoJobDataPresent)?;
            adjust_funding::execute(
                data.correlation_id,
                synth_usd_liability,
                okex,
                okex_transfers,
                galoy,
                funding_adjustment,
            )
            .await?;
            Ok::<_, HedgingError>(data)
        })
        .await?;
    Ok(())
}
