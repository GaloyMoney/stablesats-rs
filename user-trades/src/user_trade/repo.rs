use sqlx::PgPool;

use super::entity::*;
use crate::{error::UserTradesError, user_trade_unit::*};

#[derive(Clone)]
pub struct UserTrades {
    pool: PgPool,
    units: UserTradeUnits,
}

impl UserTrades {
    pub fn new(pool: PgPool, units: UserTradeUnits) -> Self {
        Self { pool, units }
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
            "INSERT INTO user_trades (uuid, buy_unit_id, buy_amount, sell_unit_id, sell_amount) VALUES ($1, $2, $3, $4, $5) RETURNING idx",
            uuid::Uuid::from(id),
            self.units.get_id(buy_unit),
            &buy_amount,
            self.units.get_id(sell_unit),
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
