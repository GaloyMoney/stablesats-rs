mod config;

use sqlxmq::OwnedHandle;

use crate::{error::*, job, user_trade::*, user_trade_balances::*, user_trade_unit::*};
pub use config::*;
use shared::pubsub::*;

pub struct UserTradesApp {
    _user_trades: UserTrades,
    _runner: OwnedHandle,
}

impl UserTradesApp {
    pub async fn run(
        UserTradesAppConfig {
            pg_con,
            migrate_on_start,
            publish_frequency,
        }: UserTradesAppConfig,
        pubsub_cfg: PubSubConfig,
    ) -> Result<Self, UserTradesError> {
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        if migrate_on_start {
            sqlx::migrate!().run(&pool).await?;
        }
        let units = UserTradeUnits::load(&pool).await?;
        let user_trade_balances = UserTradeBalances::new(pool.clone(), units.clone()).await?;
        let publisher = Publisher::new(pubsub_cfg).await?;
        let job_runner = job::start_job_runner(
            pool.clone(),
            publisher,
            publish_frequency,
            user_trade_balances,
        )
        .await?;
        Self::spawn_publish_liability(pool.clone(), publish_frequency).await?;
        Ok(Self {
            _user_trades: UserTrades::new(pool, units),
            _runner: job_runner,
        })
    }

    async fn spawn_publish_liability(
        pool: sqlx::PgPool,
        delay: std::time::Duration,
    ) -> Result<(), UserTradesError> {
        let _ = tokio::spawn(async move {
            loop {
                let _ =
                    job::spawn_publish_liability(&pool, std::time::Duration::from_secs(1)).await;
                tokio::time::sleep(delay).await;
            }
        });
        Ok(())
    }
}
