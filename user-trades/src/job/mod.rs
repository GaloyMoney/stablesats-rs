mod poll_galoy_transactions;

use rust_decimal_macros::dec;
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::{info_span, instrument, Instrument, Span};
use uuid::{uuid, Uuid};

use galoy_client::GaloyClient;
use shared::{
    payload::{SynthUsdLiabilityPayload, SyntheticCentLiability},
    pubsub::*,
};

use crate::{
    error::UserTradesError, galoy_transactions::GaloyTransactions,
    user_trade_balances::UserTradeBalances, user_trade_unit::UserTradeUnit,
    user_trades::UserTrades,
};

const PUBLISH_LIABILITY_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");
const POLL_GALOY_TRANSACTIONS_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000002");

#[derive(Debug, Clone)]
struct LiabilityPublishDelay(std::time::Duration);
#[derive(Debug, Clone)]
struct PollGaloyTransactionsDelay(std::time::Duration);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AttemptTracker {
    pub attempts: u32,
}

pub async fn start_job_runner(
    pool: sqlx::PgPool,
    publisher: Publisher,
    liability_publish_delay: std::time::Duration,
    user_trade_balances: UserTradeBalances,
    user_trades: UserTrades,
    galoy_client: GaloyClient,
    galoy_poll_delay: std::time::Duration,
) -> Result<OwnedHandle, UserTradesError> {
    let mut registry = JobRegistry::new(&[publish_liability, poll_galoy_transactions]);
    registry.set_context(publisher);
    registry.set_context(user_trade_balances);
    registry.set_context(LiabilityPublishDelay(liability_publish_delay));
    registry.set_context(user_trades);
    registry.set_context(galoy_client);
    registry.set_context(PollGaloyTransactionsDelay(galoy_poll_delay));

    Ok(registry.runner(&pool).run().await?)
}

#[instrument(skip_all, err)]
pub async fn spawn_publish_liability(
    pool: &sqlx::PgPool,
    duration: std::time::Duration,
) -> Result<(), UserTradesError> {
    match JobBuilder::new_with_id(PUBLISH_LIABILITY_ID, "publish_liability")
        .set_delay(duration)
        .set_json(&AttemptTracker { attempts: 0 })
        .expect("couldn't set json")
        .spawn(pool)
        .await
    {
        Err(sqlx::Error::Database(err)) if err.message().contains("duplicate key") => Ok(()),
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
    }
}

#[instrument(skip_all,fields(error, error.message), err)]
pub async fn spawn_poll_galoy_transactions(
    pool: &sqlx::PgPool,
    duration: std::time::Duration,
) -> Result<(), UserTradesError> {
    match JobBuilder::new_with_id(POLL_GALOY_TRANSACTIONS_ID, "poll_galoy_transactions")
        .set_delay(duration)
        .set_json(&AttemptTracker { attempts: 0 })
        .expect("couldn't set json")
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

#[job(name = "publish_liability", channel_name = "user_trades", retries = 10)]
async fn publish_liability(
    mut current_job: CurrentJob,
    publisher: Publisher,
    user_trade_balances: UserTradeBalances,
    LiabilityPublishDelay(delay): LiabilityPublishDelay,
) -> Result<(), UserTradesError> {
    let span = info_span!(
        "publish_liability",
        job_id = %current_job.id(),
        job_name = %current_job.name(),
        attempt = tracing::field::Empty,
        last_attempt = false,
        error = tracing::field::Empty,
        error.message = tracing::field::Empty,
    );
    shared::tracing::record_error(|| async move {
        let mut job_completed = false;
        if let Ok(tracker) = update_tracker(&mut current_job).await {
            if tracker.attempts > 5 {
                Span::current().record("last_attempt", &true);
                current_job.complete().await?;
                job_completed = true;
            }
        }
        let balances = user_trade_balances.fetch_all().await?;
        publisher
            .publish(SynthUsdLiabilityPayload {
                liability: SyntheticCentLiability::try_from(
                    balances
                        .get(&UserTradeUnit::SynthCent)
                        .expect("SynthCents should always be present")
                        .current_balance
                        * dec!(-1),
                )
                .expect("SynthCents should be negative"),
            })
            .await?;
        if !job_completed {
            current_job.complete().await?;
        }
        spawn_publish_liability(current_job.pool(), delay).await?;
        Ok(())
    })
    .instrument(span)
    .await
}

#[job(
    name = "poll_galoy_transactions",
    channel_name = "user_trades",
    retries = 10
)]
async fn poll_galoy_transactions(
    mut current_job: CurrentJob,
    user_trades: UserTrades,
    galoy: GaloyClient,
    PollGaloyTransactionsDelay(delay): PollGaloyTransactionsDelay,
) -> Result<(), UserTradesError> {
    let mut job_completed = false;
    if let Ok(tracker) = update_tracker(&mut current_job).await {
        if tracker.attempts > 5 {
            Span::current().record("last_attempt", &true);
            current_job.complete().await?;
            job_completed = true;
        }
    }
    poll_galoy_transactions::execute(
        user_trades,
        GaloyTransactions::new(current_job.pool().clone()),
        galoy,
    )
    .await?;
    if !job_completed {
        current_job.complete().await?;
    }
    spawn_poll_galoy_transactions(current_job.pool(), delay).await?;
    Ok(())
}

async fn update_tracker(current_job: &mut CurrentJob) -> Result<AttemptTracker, UserTradesError> {
    let mut checkpoint = sqlxmq::Checkpoint::new();
    let tracker =
        if let Ok(Some(AttemptTracker { attempts })) = current_job.json::<AttemptTracker>() {
            AttemptTracker {
                attempts: attempts + 1,
            }
        } else {
            AttemptTracker { attempts: 1 }
        };
    Span::current().record("attempt", &tracing::field::display(tracker.attempts));
    checkpoint
        .set_json(&tracker)
        .expect("Couldn't update tracker");

    current_job.checkpoint(&checkpoint).await?;
    Ok(tracker)
}
