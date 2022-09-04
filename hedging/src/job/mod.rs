mod adjust_hedge;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::{info_span, Instrument};
use uuid::{uuid, Uuid};

use std::collections::HashMap;

use okex_client::{OkexClient, PositionSize};
use shared::{
    payload::{ExchangeIdRaw, InstrumentIdRaw, OkexBtcUsdSwapPositionPayload, OKEX_EXCHANGE_ID},
    pubsub::{CorrelationId, Publisher},
};

use crate::{error::*, synth_usd_liability::*};

const POLL_OKEX_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

#[derive(Debug, Clone)]
struct OkexPollDelay(std::time::Duration);

pub async fn start_job_runner(
    pool: sqlx::PgPool,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    publisher: Publisher,
    delay: std::time::Duration,
) -> Result<OwnedHandle, HedgingError> {
    let mut registry = JobRegistry::new(&[adjust_hedge, poll_okex]);
    registry.set_context(synth_usd_liability);
    registry.set_context(okex);
    registry.set_context(publisher);
    registry.set_context(OkexPollDelay(delay));

    Ok(registry.runner(&pool).run().await?)
}

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
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
    }
}

#[derive(Serialize, Deserialize)]
struct AdjustHedgeData {
    #[serde(flatten)]
    tracing_data: HashMap<String, String>,
    correlation_id: CorrelationId,
}

pub async fn spawn_adjust_hedge<'a>(
    tx: impl Executor<'a, Database = Postgres>,
    correlation_id: CorrelationId,
) -> Result<(), HedgingError> {
    adjust_hedge
        .builder()
        .set_json(&AdjustHedgeData {
            tracing_data: shared::tracing::extract_tracing_data(),
            correlation_id,
        })?
        .spawn(tx)
        .await?;
    Ok(())
}

#[job(name = "poll_okex", channel_name = "hedging")]
async fn poll_okex(
    mut current_job: CurrentJob,
    OkexPollDelay(delay): OkexPollDelay,
    okex: OkexClient,
    publisher: Publisher,
) -> Result<(), HedgingError> {
    let PositionSize {
        value,
        instrument_id,
    } = okex.get_position_in_signed_usd().await?;
    publisher
        .publish(OkexBtcUsdSwapPositionPayload {
            exchange: ExchangeIdRaw::from(OKEX_EXCHANGE_ID),
            instrument_id: InstrumentIdRaw::from(instrument_id.to_string()),
            signed_usd_exposure: value,
        })
        .await?;
    current_job.complete().await?;
    spawn_poll_okex(current_job.pool(), delay).await?;
    Ok(())
}

#[job(name = "adjust_hedge", channel_name = "hedging")]
async fn adjust_hedge(
    current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
) -> Result<(), HedgingError> {
    let AdjustHedgeData {
        tracing_data,
        correlation_id,
    } = current_job.json()?.expect("adjust_hedge missing data");
    let span = info_span!(
        "execute_job",
        correlation_id = %correlation_id,
        job_id = %current_job.id(),
        job_name = %current_job.name(),
    );
    shared::tracing::inject_tracing_data(&span, &tracing_data);
    adjust_hedge::execute(current_job, synth_usd_liability, okex)
        .instrument(span)
        .await?;
    Ok(())
}
