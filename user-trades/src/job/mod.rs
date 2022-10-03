mod poll_galoy_transactions;

use rust_decimal_macros::dec;
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use tracing::instrument;
use uuid::{uuid, Uuid};

use galoy_client::GaloyClient;
use shared::{
    payload::{SynthUsdLiabilityPayload, SyntheticCentLiability},
    pubsub::*,
    sqlxmq::JobExecutor,
};
use std::time::Duration;

use crate::{
    error::UserTradesError, galoy_transactions::GaloyTransactions,
    user_trade_balances::UserTradeBalances, user_trade_unit::UserTradeUnit,
    user_trades::UserTrades,
};

const PUBLISH_LIABILITY_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");
const POLL_GALOY_TRANSACTIONS_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000002");

#[derive(Debug, Clone)]
struct LiabilityPublishDelay(Duration);
#[derive(Debug, Clone)]
struct PollGaloyTransactionsDelay(Duration);

pub async fn start_job_runner(
    pool: sqlx::PgPool,
    publisher: Publisher,
    liability_publish_delay: Duration,
    user_trade_balances: UserTradeBalances,
    user_trades: UserTrades,
    galoy_client: GaloyClient,
    galoy_poll_delay: Duration,
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
    duration: Duration,
) -> Result<(), UserTradesError> {
    match JobBuilder::new_with_id(PUBLISH_LIABILITY_ID, "publish_liability")
        .set_delay(duration)
        .spawn(pool)
        .await
    {
        Err(sqlx::Error::Database(err)) if err.message().contains("duplicate key") => Ok(()),
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
    }
}

#[instrument(skip_all,fields(error, error.level, error.message), err)]
pub async fn spawn_poll_galoy_transactions(
    pool: &sqlx::PgPool,
    duration: Duration,
) -> Result<(), UserTradesError> {
    match JobBuilder::new_with_id(POLL_GALOY_TRANSACTIONS_ID, "poll_galoy_transactions")
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

#[job(name = "publish_liability", channel_name = "user_trades")]
async fn publish_liability(
    mut current_job: CurrentJob,
    publisher: Publisher,
    user_trade_balances: UserTradeBalances,
    LiabilityPublishDelay(delay): LiabilityPublishDelay,
) -> Result<(), UserTradesError> {
    JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|_| async move {
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
            Ok::<_, UserTradesError>(())
        })
        .await?;
    spawn_publish_liability(current_job.pool(), delay).await?;
    Ok(())
}

#[job(name = "poll_galoy_transactions", channel_name = "user_trades")]
async fn poll_galoy_transactions(
    mut current_job: CurrentJob,
    user_trades: UserTrades,
    galoy: GaloyClient,
    PollGaloyTransactionsDelay(delay): PollGaloyTransactionsDelay,
) -> Result<(), UserTradesError> {
    let pool = current_job.pool().clone();
    let has_more = JobExecutor::builder(&mut current_job)
        .build()
        .expect("couldn't build JobExecutor")
        .execute(|_| async move {
            let galoy_transactions = GaloyTransactions::new(pool);
            poll_galoy_transactions::execute(&user_trades, &galoy_transactions, &galoy).await
        })
        .await?;
    if has_more {
        spawn_poll_galoy_transactions(current_job.pool(), Duration::from_secs(0)).await?;
    } else {
        spawn_poll_galoy_transactions(current_job.pool(), delay).await?;
    }
    Ok(())
}
