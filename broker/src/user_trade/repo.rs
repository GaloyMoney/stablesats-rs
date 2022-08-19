use crate::RepositoryError;
use super::entity::*;

pub struct UserTrades {
}

impl UserTrades {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn persist_new(&self, user_trade: UserTrade) -> Result<UserTradeId,RepositoryError>{
        let res = sqlx::query!("INSERT INTO user_trades (uuid) VALUES ($1)", uuid::Uuid::from(user_trade.id));
            // .execute(&self.pool)
            // .await
            // .map_err(|e| RepositoryError::from(e))?;
        Ok(user_trade.id)
    }
}
