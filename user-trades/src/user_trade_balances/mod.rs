use rust_decimal::Decimal;
use sqlx::{postgres::PgListener, PgPool};

use std::collections::HashMap;

use crate::{error::*, user_trade_unit::*};

pub struct UserTradeBalanceSummary {
    pub unit: UserTradeUnit,
    pub current_balance: Decimal,
    pub last_trade_id: Option<i32>,
}

struct NewBalanceResult {
    unit_id: i32,
    new_balance: Option<Decimal>,
    new_latest_id: Option<i32>,
}

#[derive(Clone)]
pub struct UserTradeBalances {
    pool: PgPool,
    units: UserTradeUnits,
}

impl UserTradeBalances {
    pub async fn new(pool: PgPool, units: UserTradeUnits) -> Result<Self, UserTradesError> {
        let mut listener = PgListener::connect_with(&pool).await?;
        listener.listen("user_trades").await?;
        let ret = Self { pool, units };
        let user_trade_balances = ret.clone();
        tokio::spawn(async move {
            while listener.recv().await.is_ok() {
                let _ = user_trade_balances.update_balances().await;
            }
        });
        ret.update_balances().await?;
        Ok(ret)
    }

    async fn update_balances(&self) -> Result<(), UserTradesError> {
        let mut tx = self.pool.begin().await?;
        let balance_result =
            sqlx::query!(r#"SELECT last_trade_id FROM user_trade_balances FOR UPDATE"#)
                .fetch_all(&mut tx)
                .await?;

        let last_tx_id = balance_result
            .into_iter()
            .map(|res| res.last_trade_id.unwrap_or(0))
            .fold(0, |a, b| a.max(b));

        let new_balance_result = sqlx::query_file_as!(
            NewBalanceResult,
            "src/user_trade_balances/sql/new-balance.sql",
            last_tx_id
        )
        .fetch_all(&mut tx)
        .await?;

        for NewBalanceResult {
            unit_id,
            new_balance,
            new_latest_id,
        } in new_balance_result
        {
            sqlx::query!("UPDATE user_trade_balances SET current_balance = $1, last_trade_id = $2, updated_at = now() WHERE unit_id = $3",
                         new_balance, new_latest_id, unit_id)
                .execute(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn fetch_all(
        &self,
    ) -> Result<HashMap<UserTradeUnit, UserTradeBalanceSummary>, UserTradesError> {
        let balance_result = sqlx::query!(
            r#"SELECT unit_id, current_balance, last_trade_id FROM user_trade_balances"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(balance_result
            .into_iter()
            .map(|balance| {
                let unit = self.units.from_id(balance.unit_id);
                (
                    unit,
                    UserTradeBalanceSummary {
                        unit,
                        current_balance: balance.current_balance,
                        last_trade_id: balance.last_trade_id,
                    },
                )
            })
            .collect())
    }
}
