mod config;

use sqlxmq::JobRunnerHandle;
use tracing::instrument;

use galoy_client::{GaloyClient, GaloyClientConfig};

use crate::{error::*, job, user_trades::*};
pub use config::*;

pub struct UserTradesApp {
    _runner: JobRunnerHandle,
}

impl UserTradesApp {
    #[instrument(name = "UserTradesApp.run", skip_all, fields(error, error.level, error.message))]
    pub async fn run(
        pool: sqlx::PgPool,
        UserTradesConfig {
            galoy_poll_frequency,
        }: UserTradesConfig,
        galoy_client_cfg: GaloyClientConfig,
        ledger: ledger::Ledger,
    ) -> Result<Self, UserTradesError> {
        let user_trades = UserTrades::new(pool.clone());
        let job_runner = job::start_job_runner(
            pool.clone(),
            ledger,
            user_trades,
            shared::tracing::record_error(tracing::Level::ERROR, || async move {
                GaloyClient::connect(galoy_client_cfg).await
            })
            .await?,
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
