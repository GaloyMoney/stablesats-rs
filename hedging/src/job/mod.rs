mod adjust_hedge;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::instrument;
use tracing::{info_span, Instrument};
use uuid::{uuid, Uuid};

use std::collections::HashMap;

use okex_client::{OkexClient, PositionSize};
use shared::{
    payload::{ExchangeIdRaw, InstrumentIdRaw, OkexBtcUsdSwapPositionPayload, OKEX_EXCHANGE_ID},
    pubsub::{CorrelationId, Publisher},
};

use crate::{error::*, hedging_adjustments::HedgingAdjustments, synth_usd_liability::*};

const POLL_OKEX_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

#[derive(Debug, Clone)]
struct OkexPollDelay(std::time::Duration);

pub async fn start_job_runner(
    pool: sqlx::PgPool,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    hedging_adjustments: HedgingAdjustments,
    publisher: Publisher,
    delay: std::time::Duration,
) -> Result<OwnedHandle, HedgingError> {
    let mut registry = JobRegistry::new(&[adjust_hedge, poll_okex]);
    registry.set_context(synth_usd_liability);
    registry.set_context(okex);
    registry.set_context(publisher);
    registry.set_context(OkexPollDelay(delay));
    registry.set_context(hedging_adjustments);

    Ok(registry.runner(&pool).run().await?)
}

#[instrument(skip_all, fields(error, error.message), err)]
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
            shared::tracing::insert_error_fields(&e);
            Err(e.into())
        }
        Ok(_) => Ok(()),
    }
}

#[derive(Serialize, Deserialize)]
struct AdjustHedgeData {
    #[serde(flatten)]
    tracing_data: HashMap<String, String>,
    correlation_id: CorrelationId,
}

#[instrument(skip_all, err)]
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

#[job(name = "poll_okex", channel_name = "hedging", retries = 1000)]
async fn poll_okex(
    mut current_job: CurrentJob,
    OkexPollDelay(delay): OkexPollDelay,
    okex: OkexClient,
    publisher: Publisher,
) -> Result<(), HedgingError> {
    let span = info_span!(
        "poll_okex",
        job_id = %current_job.id(),
        job_name = %current_job.name(),
        error = tracing::field::Empty,
        error.message = tracing::field::Empty,
    );
    shared::tracing::record_error(|| async move {
        checkpoint_job(&mut current_job).await?;
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
    })
    .instrument(span)
    .await
}

#[job(name = "adjust_hedge", channel_name = "hedging")]
async fn adjust_hedge(
    current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    hedging_adjustments: HedgingAdjustments,
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
        error = tracing::field::Empty,
        error.message = tracing::field::Empty,
    );
    shared::tracing::inject_tracing_data(&span, &tracing_data);
    shared::tracing::record_error(|| async move {
        adjust_hedge::execute(
            current_job,
            correlation_id,
            synth_usd_liability,
            okex,
            hedging_adjustments,
        )
        .await
    })
    .instrument(span)
    .await?;
    Ok(())
}

async fn checkpoint_job(current_job: &mut CurrentJob) -> Result<(), HedgingError> {
    let mut checkpoint = sqlxmq::Checkpoint::new();
    checkpoint.set_extra_retries(1);

    let raw_json = current_job.raw_json().map(String::from);
    if let Some(json) = raw_json.as_ref() {
        checkpoint.set_raw_json(json);
    }
    current_job.checkpoint(&checkpoint).await?;
    Ok(())
}
