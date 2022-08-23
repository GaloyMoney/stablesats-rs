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

pub struct UserTradeSum {
    max: Option<i32>,
    unit: UserTradeUnit,
    sum: Option<Decimal>,
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
        let balance_result = sqlx::query_as!(UserTradeBalanceSummary, r#"SELECT unit as "unit: _", current_balance, last_trade_idx FROM user_trade_balances FOR UPDATE"#).fetch_all(&mut tx).await?;
        let mut last_tx_idx = 0;
        let mut balances: HashMap<UserTradeUnit, Decimal> = balance_result
            .into_iter()
            .map(|balance| {
                if let Some(last_idx) = balance.last_trade_idx {
                    last_tx_idx = last_idx
                }
                (balance.unit, balance.current_balance)
            })
            .collect();

        let buy_sum = sqlx::query_as!(UserTradeSum, r#"SELECT MAX(idx), buy_unit as "unit: _", SUM(buy_amount) FROM user_trades WHERE idx > $1 GROUP BY buy_unit"#, last_tx_idx)
            .fetch_all(&mut tx)
            .await?;
        let mut latest_tx_idx = last_tx_idx;
        buy_sum
            .into_iter()
            .for_each(|UserTradeSum { max, unit, sum }| {
                if let (Some(mut balance), Some(max), Some(sum)) =
                    (balances.get_mut(&unit), max, sum)
                {
                    balance -= sum;
                    if max > latest_tx_idx {
                        latest_tx_idx = max;
                    }
                }
            });
        let sell_sum = sqlx::query_as!(UserTradeSum, r#"SELECT MAX(idx), sell_unit as "unit: _", SUM(sell_amount) FROM user_trades WHERE idx > $1 GROUP BY sell_unit"#, last_tx_idx)
            .fetch_all(&mut tx)
            .await?;
        sell_sum
            .into_iter()
            .for_each(|UserTradeSum { max, unit, sum }| {
                if let (Some(mut balance), Some(max), Some(sum)) =
                    (balances.get_mut(&unit), max, sum)
                {
                    balance += sum;
                    if max > latest_tx_idx {
                        latest_tx_idx = max;
                    }
                }
            });

        if latest_tx_idx == last_tx_idx {
            tx.rollback().await?;
            return Ok(());
        }

        for (unit, balance) in balances {
            sqlx::query!("UPDATE user_trade_balances SET current_balance = $1, last_trade_idx = $2, updated_at = now() WHERE unit = $3", balance, latest_tx_idx, unit as UserTradeUnit)
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
