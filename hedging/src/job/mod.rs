mod adjust_hedge;
mod poll_okex;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::instrument;
use uuid::{uuid, Uuid};

use std::collections::HashMap;

use okex_client::OkexClient;
use shared::{
    pubsub::{CorrelationId, Publisher},
    sqlxmq::JobExecutor,
};

use crate::{error::*, okex_orders::OkexOrders, synth_usd_liability::*};

const POLL_OKEX_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

#[derive(Debug, Clone)]
struct OkexPollDelay(std::time::Duration);

pub async fn start_job_runner(
    pool: sqlx::PgPool,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_orders: OkexOrders,
    publisher: Publisher,
    delay: std::time::Duration,
) -> Result<OwnedHandle, HedgingError> {
    let mut registry = JobRegistry::new(&[adjust_hedge, poll_okex]);
    registry.set_context(synth_usd_liability);
    registry.set_context(okex);
    registry.set_context(publisher);
    registry.set_context(OkexPollDelay(delay));
    registry.set_context(okex_orders);

    Ok(registry.runner(&pool).run().await?)
}

#[instrument(skip_all, fields(error, error.level, error.message), err)]
pub async fn spawn_poll_okex(
    pool: &sqlx::PgPool,
    duration: std::time::Duration,
) -> Result<(), HedgingError> {
    match JobBuilder::new_with_id(POLL_OKEX_ID, "poll_okex")
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
    match JobBuilder::new_with_id(Uuid::from(correlation_id), "adjust_hedge")
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

#[job(name = "poll_okex", channel_name = "hedging", retries = 20)]
async fn poll_okex(
    mut current_job: CurrentJob,
    OkexPollDelay(delay): OkexPollDelay,
    okex: OkexClient,
    okex_orders: OkexOrders,
    publisher: Publisher,
) -> Result<(), HedgingError> {
    JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|_| async move { poll_okex::execute(okex_orders, okex, publisher).await })
        .await?;
    spawn_poll_okex(current_job.pool(), delay).await?;
    Ok(())
}

#[job(name = "adjust_hedge", channel_name = "hedging", ordered)]
async fn adjust_hedge(
    mut current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_orders: OkexOrders,
) -> Result<(), HedgingError> {
    JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|data| async move {
            let data: AdjustHedgeData = data.expect("no AdjustHedgeData available");
            adjust_hedge::execute(data.correlation_id, synth_usd_liability, okex, okex_orders)
                .await?;
            Ok::<_, HedgingError>(data)
        })
        .await?;
    Ok(())
}
