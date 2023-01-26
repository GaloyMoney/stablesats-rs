use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    #[serde(default)]
    pub pg_con: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "bool_true")]
    pub migrate_on_start: bool,
}

pub async fn init_pool(config: &DbConfig) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.pool_size)
        .connect(&config.pg_con)
        .await?;
    if config.migrate_on_start {
        sqlx::migrate!("../migrations").run(&pool).await?;
    }
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM mq_payloads WHERE id IN (SELECT id FROM mq_msgs WHERE attempts = 0 AND id = ANY($1))",
        &[
        hedging::job::POLL_OKEX_ID,
        user_trades::job::POLL_GALOY_TRANSACTIONS_ID,
        user_trades::job::PUBLISH_LIABILITY_ID,
        ]
    )
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM mq_msgs WHERE id IN (SELECT id FROM mq_msgs WHERE attempts = 0 AND id = ANY($1))",
        &[
        hedging::job::POLL_OKEX_ID,
        user_trades::job::POLL_GALOY_TRANSACTIONS_ID,
        user_trades::job::PUBLISH_LIABILITY_ID,
        ]
    )
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(pool)
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            pool_size: default_pool_size(),
            migrate_on_start: true,
        }
    }
}

fn default_pool_size() -> u32 {
    20
}

fn bool_true() -> bool {
    true
}
