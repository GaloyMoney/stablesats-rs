mod poll_galoy_transactions;

use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::instrument;
use uuid::{uuid, Uuid};

use galoy_client::GaloyClient;
use shared::sqlxmq::JobExecutor;
use std::time::Duration;

use crate::{
    error::UserTradesError, galoy_transactions::GaloyTransactions, user_trades::UserTrades,
};

// retired: uuid!("10000000-0000-0000-0000-000000000001");
pub const POLL_GALOY_TRANSACTIONS_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000002");

#[derive(Debug, Clone)]
struct PollGaloyTransactionsDelay(Duration);

#[allow(clippy::too_many_arguments)]
pub async fn start_job_runner(
    pool: sqlx::PgPool,
    ledger: ledger::Ledger,
    user_trades: UserTrades,
    galoy_client: GaloyClient,
    galoy_poll_delay: Duration,
) -> Result<OwnedHandle, UserTradesError> {
    let mut registry = JobRegistry::new(&[poll_galoy_transactions]);
    registry.set_context(ledger);
    registry.set_context(user_trades);
    registry.set_context(galoy_client);
    registry.set_context(PollGaloyTransactionsDelay(galoy_poll_delay));

    Ok(registry
        .runner(&pool)
        .set_channel_names(&["user_trades"])
        .run()
        .await?)
}

#[instrument(name = "user_trades.job.spawn_poll_galoy_transactions", skip_all,fields(error, error.level, error.message), err)]
pub async fn spawn_poll_galoy_transactions(
    pool: &sqlx::PgPool,
    duration: Duration,
) -> Result<(), UserTradesError> {
    match JobBuilder::new_with_id(POLL_GALOY_TRANSACTIONS_ID, "poll_galoy_transactions")
        .set_channel_name("user_trades")
        .set_channel_args("poll_galoy_transactions")
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

#[job(name = "poll_galoy_transactions")]
async fn poll_galoy_transactions(
    mut current_job: CurrentJob,
    user_trades: UserTrades,
    galoy: GaloyClient,
    PollGaloyTransactionsDelay(delay): PollGaloyTransactionsDelay,
    ledger: ledger::Ledger,
) -> Result<(), UserTradesError> {
    let pool = current_job.pool().clone();
    let has_more = JobExecutor::builder(&mut current_job)
        .initial_retry_delay(Duration::from_secs(5))
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|_| async move {
            let galoy_transactions = GaloyTransactions::new(pool.clone());
            poll_galoy_transactions::execute(
                &pool,
                &user_trades,
                &galoy_transactions,
                &galoy,
                &ledger,
            )
            .await
        })
        .await?;
    if has_more {
        spawn_poll_galoy_transactions(current_job.pool(), Duration::from_secs(0)).await?;
    } else {
        spawn_poll_galoy_transactions(current_job.pool(), delay).await?;
    }
    Ok(())
}
