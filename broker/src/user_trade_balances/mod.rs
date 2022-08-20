use sqlx::{postgres::PgListener, PgPool};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserTradeBalancesError {
    #[error("UserTradeBlancesError: {0}")]
    SqlxError(#[from] sqlx::Error),
}

pub struct UserTradeBalances {
    pool: PgPool,
}

impl UserTradeBalances {
    pub async fn new(pool: PgPool) -> Result<Self, UserTradeBalancesError> {
        let mut listener = PgListener::connect_with(&pool).await?;
        listener.listen("user_trades").await?;
        let ret = Self { pool };
        tokio::spawn(async move {
            while let Ok(notification) = listener.recv().await {
                println!("{:?}", notification);
            }
        });

        Ok(ret)
    }
}
