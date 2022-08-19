#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
pub mod user_trade;
mod app;
mod repository_error;

pub mod migrations {
    use sqlx::*;

    pub async fn run(pg_con: &str) -> anyhow::Result<()> {
        let mut con = sqlx::PgConnection::connect(&pg_con).await?;
        sqlx::migrate!()
            .run(&mut con)
            .await?;
        Ok(())
    }
}

pub use config::*;
pub use repository_error::RepositoryError;
pub use app::BrokerApp;
