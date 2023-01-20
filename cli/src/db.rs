use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    #[serde(default)]
    pub pg_con: String,
    #[serde(default = "bool_true")]
    pub migrate_on_start: bool,
}

pub async fn init_pool(config: DbConfig) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::PgPool::connect(&config.pg_con).await?;
    if config.migrate_on_start {
        sqlx::migrate!("../migrations").run(&pool).await?;
    }
    Ok(pool)
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            migrate_on_start: true,
        }
    }
}

fn bool_true() -> bool {
    true
}
