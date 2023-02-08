mod config;

use sqlxmq::OwnedHandle;

use galoy_client::{GaloyClient, GaloyClientConfig};

use crate::{error::*, job, user_trades::*};
pub use config::*;

pub struct UserTradesApp {
    _runner: OwnedHandle,
}

impl UserTradesApp {
    pub async fn run(
        pool: sqlx::PgPool,
        UserTradesConfig {
            galoy_poll_frequency,
        }: UserTradesConfig,
        galoy_client_cfg: GaloyClientConfig,
    ) -> Result<Self, UserTradesError> {
        let ledger = ledger::Ledger::init(&pool).await?;
        let user_trades = UserTrades::new(pool.clone());
        let job_runner = job::start_job_runner(
            pool.clone(),
            ledger,
            user_trades,
            GaloyClient::connect(galoy_client_cfg).await?,
            galoy_poll_frequency,
        )
        .await?;
        Self::spawn_poll_galoy_transactions(pool, galoy_poll_frequency).await?;
        Ok(Self {
            _runner: job_runner,
        })
    }

    async fn spawn_poll_galoy_transactions(
        pool: sqlx::PgPool,
        delay: std::time::Duration,
    ) -> Result<(), UserTradesError> {
        loop {
            let _ =
                job::spawn_poll_galoy_transactions(&pool, std::time::Duration::from_secs(1)).await;
            tokio::time::sleep(delay).await;
        }
    }
}
