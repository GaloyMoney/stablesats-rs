use rust_decimal::Decimal;
use sqlx::{postgres::PgListener, PgPool};
use tracing::{info_span, instrument, Instrument};

use std::collections::HashMap;

use crate::{error::*, user_trade_unit::*};

pub struct UserTradeBalanceSummary {
    pub unit: UserTradeUnit,
    pub current_balance: Decimal,
    pub last_trade_id: Option<i32>,
}

#[derive(Debug)]
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
        let ret = Self {
            pool: pool.clone(),
            units,
        };
        let user_trade_balances = ret.clone();
        tokio::spawn(async move {
            loop {
                let _ = Self::listen_to_events(pool.clone(), user_trade_balances.clone()).await;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
        Ok(ret)
    }

    async fn listen_to_events(
        pool: PgPool,
        user_trade_balances: Self,
    ) -> Result<(), UserTradesError> {
        let span = info_span!(
            "spawn_listen_to_user_trades_notifications",
            error = tracing::field::Empty,
            error.level = tracing::field::Empty,
            error.message = tracing::field::Empty,
        );
        let (send, recv) = tokio::sync::oneshot::channel();
        shared::tracing::record_error::<(), UserTradesError, _, _>(
            tracing::Level::ERROR,
            || async move {
                let mut listener = PgListener::connect_with(&pool).await?;
                listener.listen("user_trades").await?;
                user_trade_balances.update_balances().await?;
                tokio::spawn(async move {
                    loop {
                        match listener.recv().await {
                            Ok(_) => {
                                let span = info_span!(
                                    "user_trades_notification_received",
                                    error = tracing::field::Empty,
                                    error.level = tracing::field::Empty,
                                    error.message = tracing::field::Empty,
                                );
                                let repo = user_trade_balances.clone();
                                if let Err(e) = shared::tracing::record_error(
                                    tracing::Level::WARN,
                                    || async move { repo.update_balances().await },
                                )
                                .instrument(span)
                                .await
                                {
                                    let _ = send.send(e);
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = send.send(e.into());
                                break;
                            }
                        }
                    }
                });
                Ok(())
            },
        )
        .instrument(span)
        .await?;
        let _ = recv.await;
        Ok(())
    }

    #[instrument(name = "update_user_trade_balances", skip(self),
      fields(error, error.level, error.message), err)]
    async fn update_balances(&self) -> Result<(), UserTradesError> {
        shared::tracing::record_error(tracing::Level::WARN, || async move {
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

            let mut updated = false;
            for NewBalanceResult {
                unit_id,
                new_balance,
                new_latest_id,
            } in new_balance_result
            {
                if let Some(new_latest_id) = new_latest_id {
                    sqlx::query!("UPDATE user_trade_balances SET current_balance = $1, last_trade_id = $2, updated_at = now() WHERE unit_id = $3",
                    new_balance, new_latest_id, unit_id)
                        .execute(&mut tx)
                        .await?;
                    updated = true;
                }
            }
            if updated {
                tx.commit().await?;
            }
            Ok(())
        }).await
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
