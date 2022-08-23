use rust_decimal::Decimal;
use sqlx::{postgres::PgListener, Executor, PgPool};
use thiserror::Error;

use std::collections::HashMap;

use crate::user_trade::UserTradeUnit;

#[derive(Error, Debug)]
pub enum UserTradeBalancesError {
    #[error("UserTradeBalancesError: {0}")]
    Sqlx(#[from] sqlx::Error),
}

pub struct UserTradeBalanceSummary {
    pub unit: UserTradeUnit,
    pub current_balance: Decimal,
    pub last_trade_idx: Option<i32>,
}

pub struct NewBalanceResult {
    unit: UserTradeUnit,
    new_balance: Option<Decimal>,
    new_latest_idx: Option<i32>,
}

#[derive(Clone)]
pub struct UserTradeBalances {
    pool: PgPool,
}

impl UserTradeBalances {
    pub async fn new(pool: PgPool) -> Result<Self, UserTradeBalancesError> {
        let mut listener = PgListener::connect_with(&pool).await?;
        listener.listen("user_trades").await?;
        let ret = Self { pool };
        let user_trade_balances = ret.clone();
        tokio::spawn(async move {
            while listener.recv().await.is_ok() {
                let _ = user_trade_balances.update_balances().await;
            }
        });
        ret.update_balances().await?;
        Ok(ret)
    }

    pub async fn update_balances(&self) -> Result<(), UserTradeBalancesError> {
        let mut tx = self.pool.begin().await?;
        tx.execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;")
            .await?;
        let balance_result =
            sqlx::query!(r#"SELECT last_trade_idx FROM user_trade_balances FOR UPDATE"#)
                .fetch_all(&mut tx)
                .await?;

        let last_tx_idx = balance_result
            .into_iter()
            .map(|res| res.last_trade_idx.unwrap_or(0))
            .fold(0, |a, b| a.max(b));

        let new_balance_result = sqlx::query_file_as!(
            NewBalanceResult,
            "src/user_trade_balances/sql/new-balance.sql",
            last_tx_idx
        )
        .fetch_all(&mut tx)
        .await?;

        for NewBalanceResult {
            unit,
            new_balance,
            new_latest_idx,
        } in new_balance_result
        {
            sqlx::query!("UPDATE user_trade_balances SET current_balance = $1, last_trade_idx = $2, updated_at = now() WHERE unit = $3",
                         new_balance, new_latest_idx, unit as UserTradeUnit)
                .execute(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn fetch_all(
        &self,
    ) -> Result<HashMap<UserTradeUnit, UserTradeBalanceSummary>, UserTradeBalancesError> {
        let balance_result = sqlx::query_as!(
            UserTradeBalanceSummary,
            r#"SELECT unit as "unit: _", current_balance, last_trade_idx FROM user_trade_balances"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(balance_result
            .into_iter()
            .map(|balance| (balance.unit, balance))
            .collect())
    }
}
