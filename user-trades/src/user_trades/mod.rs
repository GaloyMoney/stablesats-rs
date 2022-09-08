use sqlx::PgPool;

use crate::{error::UserTradesError, user_trade_unit::*};
use rust_decimal::Decimal;

use crate::user_trade_unit::UserTradeUnit;

pub struct NewUserTrade {
    pub buy_unit: UserTradeUnit,
    pub buy_amount: Decimal,
    pub sell_unit: UserTradeUnit,
    pub sell_amount: Decimal,
    pub tx_cursor: String, // TODO: refactor to domain type
}

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
            buy_unit,
            buy_amount,
            sell_unit,
            sell_amount,
            tx_cursor,
        }: NewUserTrade,
    ) -> Result<i32, UserTradesError> {
        let res = sqlx::query!(
            "INSERT INTO user_trades (buy_unit_id, buy_amount, sell_unit_id, sell_amount, tx_cursor) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            self.units.get_id(buy_unit),
            &buy_amount,
            self.units.get_id(sell_unit),
            &sell_amount,
            tx_cursor,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(res.id)
    }

    pub async fn latest_tx_cursor(&self) -> Result<String, UserTradesError> {
        let latest_cursor = sqlx::query!(
            r#"
                SELECT tx_cursor
                FROM user_trades
                ORDER BY tx_cursor DESC
                LIMIT 1 FOR UPDATE
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .tx_cursor;

        Ok(latest_cursor)
    }
}
