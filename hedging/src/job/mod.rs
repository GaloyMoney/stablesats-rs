mod adjust_hedge;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::{info_span, instrument, Instrument, Span};
use uuid::{uuid, Uuid};

use std::collections::HashMap;

use okex_client::{OkexClient, OkexClientError, PositionSize};
use shared::{
    payload::{
        ExchangeIdRaw, InstrumentIdRaw, OkexBtcUsdSwapPositionPayload, SyntheticCentExposure,
        OKEX_EXCHANGE_ID,
    },
    pubsub::{CorrelationId, Publisher},
};

use crate::{error::*, okex_orders::OkexOrders, synth_usd_liability::*};

const POLL_OKEX_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

#[derive(Debug, Clone)]
struct OkexPollDelay(std::time::Duration);

#[derive(Serialize, Deserialize)]
pub struct AttemptTracker {
    pub attempts: u32,
}

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
        .set_json(&AttemptTracker { attempts: 0 })
        .expect("Couldn't set json")
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
    #[serde(flatten)]
    tracing_data: HashMap<String, String>,
    correlation_id: CorrelationId,
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

#[job(name = "poll_okex", channel_name = "hedging", retries = 10)]
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
        attempt = tracing::field::Empty,
        last_attempt = false,
        error = tracing::field::Empty,
        error.level = tracing::field::Empty,
        error.message = tracing::field::Empty,
    );
    let tracker = current_tracker(&current_job);
    shared::tracing::record_error(
        if tracker.attempts >= 4 {
            tracing::Level::ERROR
        } else {
            tracing::Level::WARN
        },
        || async move {
            let mut job_completed = false;
            if let Ok(tracker) = update_tracker(&mut current_job).await {
                if tracker.attempts > 5 {
                    Span::current().record("last_attempt", &true);
                    current_job.complete().await?;
                    job_completed = true;
                }
            }
            let PositionSize {
                usd_cents,
                instrument_id,
            } = okex.get_position_in_signed_usd_cents().await?;
            publisher
                .publish(OkexBtcUsdSwapPositionPayload {
                    exchange: ExchangeIdRaw::from(OKEX_EXCHANGE_ID),
                    instrument_id: InstrumentIdRaw::from(instrument_id.to_string()),
                    signed_usd_exposure: SyntheticCentExposure::from(usd_cents),
                })
                .await?;
            if !job_completed {
                current_job.complete().await?;
            }
            spawn_poll_okex(current_job.pool(), delay).await?;
            Ok(())
        },
    )
    .instrument(span)
    .await
}

#[job(name = "adjust_hedge", channel_name = "hedging", retries = 20)]
async fn adjust_hedge(
    mut current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_orders: OkexOrders,
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
        error.level = tracing::field::Empty,
        error.message = tracing::field::Empty,
    );
    shared::tracing::inject_tracing_data(&span, &tracing_data);
    shared::tracing::record_error(tracing::Level::ERROR, || async move {
        let result =
            match adjust_hedge::execute(correlation_id, synth_usd_liability, okex, okex_orders)
                .await
            {
                Err(HedgingError::OkexClient(OkexClientError::ServiceUnavailable { .. })) => Ok(()),
                res => res,
            };
        current_job.complete().await?;
        result
    })
    .instrument(span)
    .await?;
    Ok(())
}

fn current_tracker(current_job: &CurrentJob) -> AttemptTracker {
    if let Ok(Some(AttemptTracker { attempts })) = current_job.json::<AttemptTracker>() {
        AttemptTracker {
            attempts: attempts + 1,
        }
    } else {
        AttemptTracker { attempts: 1 }
    }
}

async fn update_tracker(current_job: &mut CurrentJob) -> Result<AttemptTracker, HedgingError> {
    let mut checkpoint = sqlxmq::Checkpoint::new();
    let tracker = current_tracker(current_job);
    Span::current().record("attempt", &tracing::field::display(tracker.attempts));
    checkpoint
        .set_json(&tracker)
        .expect("Couldn't update tracker");

    current_job.checkpoint(&checkpoint).await?;
    Ok(tracker)
}
