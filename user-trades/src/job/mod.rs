use rust_decimal_macros::dec;
use sqlxmq::{job, CurrentJob, JobBuilder, JobRegistry, OwnedHandle};
use uuid::{uuid, Uuid};

use shared::{payload::SynthUsdLiabilityPayload, pubsub::*};

use crate::{
    error::UserTradesError, user_trade_balances::UserTradeBalances, user_trade_unit::UserTradeUnit,
};

const PUBLISH_LIABILITY_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

#[derive(Debug, Clone)]
struct LiabilityPublishDelay(std::time::Duration);

pub async fn start_job_runner(
    pool: sqlx::PgPool,
    publisher: Publisher,
    liability_publish_delay: std::time::Duration,
    user_trade_balances: UserTradeBalances,
) -> Result<OwnedHandle, UserTradesError> {
    let mut registry = JobRegistry::new(&[publish_liability]);
    registry.set_context(publisher);
    registry.set_context(user_trade_balances);
    registry.set_context(LiabilityPublishDelay(liability_publish_delay));

    Ok(registry.runner(&pool).run().await?)
}

pub async fn spawn_publish_liability(
    pool: &sqlx::PgPool,
    duration: std::time::Duration,
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

#[job(name = "publish_liability", channel_name = "user_trades")]
async fn publish_liability(
    mut current_job: CurrentJob,
    publisher: Publisher,
    user_trade_balances: UserTradeBalances,
    LiabilityPublishDelay(delay): LiabilityPublishDelay,
) -> Result<(), UserTradesError> {
    let balances = user_trade_balances.fetch_all().await?;
    publisher
        .publish(SynthUsdLiabilityPayload {
            liability: balances
                .get(&UserTradeUnit::SynthCent)
                .expect("SynthCents should always be present")
                .current_balance
                * dec!(-1),
        })
        .await?;
    current_job.complete().await?;
    spawn_publish_liability(current_job.pool(), delay).await?;
    Ok(())
}
