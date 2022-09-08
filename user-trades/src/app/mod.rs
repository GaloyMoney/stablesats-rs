mod config;

use sqlxmq::OwnedHandle;

use galoy_client::{GaloyClient, GaloyClientConfig};
use shared::pubsub::*;

use crate::{error::*, job, user_trade_balances::*, user_trade_unit::*, user_trades::*};
pub use config::*;

pub struct UserTradesApp {
    _runner: OwnedHandle,
}

impl UserTradesApp {
    pub async fn run(
        UserTradesAppConfig {
            pg_con,
            migrate_on_start,
            publish_frequency,
            galoy_poll_frequency,
        }: UserTradesAppConfig,
        pubsub_cfg: PubSubConfig,
        galoy_client_cfg: GaloyClientConfig,
    ) -> Result<Self, UserTradesError> {
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        if migrate_on_start {
            sqlx::migrate!().run(&pool).await?;
        }
        let units = UserTradeUnits::load(&pool).await?;
        let user_trade_balances = UserTradeBalances::new(pool.clone(), units.clone()).await?;
        let user_trades = UserTrades::new(pool.clone(), units);
        let publisher = Publisher::new(pubsub_cfg).await?;
        let job_runner = job::start_job_runner(
            pool.clone(),
            publisher,
            publish_frequency,
            user_trade_balances,
            user_trades,
            GaloyClient::connect(galoy_client_cfg).await?,
            galoy_poll_frequency,
        )
        .await?;
        Self::spawn_publish_liability(pool.clone(), publish_frequency).await?;
        Self::spawn_poll_galoy_transactions(pool, galoy_poll_frequency).await?;
        Ok(Self {
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

    async fn spawn_poll_galoy_transactions(
        pool: sqlx::PgPool,
        delay: std::time::Duration,
    ) -> Result<(), UserTradesError> {
        let _ = tokio::spawn(async move {
            loop {
                let _ =
                    job::spawn_poll_galoy_transactions(&pool, std::time::Duration::from_secs(1))
                        .await;
                tokio::time::sleep(delay).await;
            }
        });
        Ok(())
    }
}
