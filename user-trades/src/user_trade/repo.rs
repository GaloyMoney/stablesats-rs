use sqlx::PgPool;

use super::entity::*;
use super::error::UserTradesError;

pub struct UserTrades {
    pool: PgPool,
}

impl UserTrades {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn persist_new(
        &self,
        NewUserTrade {
            id,
            buy_unit,
            buy_amount,
            sell_unit,
            sell_amount,
        }: NewUserTrade,
    ) -> Result<UserTrade, UserTradesError> {
        let res = sqlx::query!(
            "INSERT INTO user_trades (uuid, buy_unit, buy_amount, sell_unit, sell_amount) VALUES ($1, $2, $3, $4, $5) RETURNING idx",
            uuid::Uuid::from(id),
            buy_unit as UserTradeUnit,
            &buy_amount,
            sell_unit as UserTradeUnit,
            &sell_amount,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(UserTrade {
            id,
            idx: res.idx,
            buy_unit,
            buy_amount,
            sell_unit,
            sell_amount,
        })
    }
}
