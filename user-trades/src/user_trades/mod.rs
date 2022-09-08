use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres, QueryBuilder, Transaction};

use crate::{error::UserTradesError, user_trade_unit::*};
use rust_decimal::Decimal;

use crate::user_trade_unit::UserTradeUnit;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalRef {
    pub cursor: String,
    pub btc_tx_id: String,
    pub usd_tx_id: String,
}

pub struct LatestRef<'a> {
    id: Option<i32>,
    external_ref: Option<ExternalRef>,
    tx: Transaction<'a, Postgres>,
}
impl<'a> LatestRef<'a> {
    pub fn take(&mut self) -> Option<ExternalRef> {
        self.external_ref.take()
    }

    pub fn id(&self) -> Option<i32> {
        self.id
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewUserTrade {
    pub is_latest: bool,
    pub buy_unit: UserTradeUnit,
    pub buy_amount: Decimal,
    pub sell_unit: UserTradeUnit,
    pub sell_amount: Decimal,
    pub external_ref: Option<ExternalRef>,
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

    pub async fn persist_all<'a>(
        &self,
        LatestRef {
            id: latest_id,
            mut tx,
            ..
        }: LatestRef<'a>,
        new_user_trades: Vec<NewUserTrade>,
    ) -> Result<(), UserTradesError> {
        if let Some(latest_id) = latest_id {
            sqlx::query!(
                "UPDATE user_trades SET is_latest = NULL WHERE id = $1",
                latest_id
            )
            .execute(&mut tx)
            .await?;
        }
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO user_trades (is_latest, buy_unit_id, buy_amount, sell_unit_id, sell_amount, external_ref) "
        );
        query_builder.push_values(
            new_user_trades,
            |mut builder,
             NewUserTrade {
                 is_latest,
                 buy_unit,
                 buy_amount,
                 sell_unit,
                 sell_amount,
                 external_ref,
             }| {
                builder.push_bind(is_latest);
                builder.push_bind(self.units.get_id(buy_unit));
                builder.push_bind(buy_amount);
                builder.push_bind(self.units.get_id(sell_unit));
                builder.push_bind(sell_amount);
                builder.push_bind(external_ref.map(|external_ref| {
                    serde_json::to_value(external_ref).expect("failed to serialize external_ref")
                }));
            },
        );
        let query = query_builder.build();
        query.execute(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Uses optimistic locking as described
    /// https://stackoverflow.com/questions/71987836/postgresql-select-for-update-lock-new-rows/71988854#71988854
    pub async fn get_latest_ref(&self) -> Result<LatestRef, UserTradesError> {
        let mut tx = self.pool.begin().await?;
        tx.execute("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .await?;
        let res = sqlx::query!("SELECT id, external_ref FROM user_trades WHERE is_latest = 'true'")
            .fetch_all(&mut tx)
            .await?;

        if let Some(res) = res.into_iter().next() {
            Ok(LatestRef {
                id: Some(res.id),
                external_ref: res.external_ref.map(|res| {
                    serde_json::from_value(res).expect("Couldn't deserialize external ref")
                }),
                tx,
            })
        } else {
            Ok(LatestRef {
                id: None,
                external_ref: None,
                tx,
            })
        }
    }
}
