use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};

use crate::{error::UserTradesError, user_trade_unit::*};
use rust_decimal::Decimal;

use crate::user_trade_unit::UserTradeUnit;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalRef {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub btc_tx_id: String,
    pub usd_tx_id: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewUserTrade {
    pub is_latest: Option<bool>,
    pub buy_unit: UserTradeUnit,
    pub buy_amount: Decimal,
    pub sell_unit: UserTradeUnit,
    pub sell_amount: Decimal,
    pub external_ref: Option<ExternalRef>,
}

#[derive(Clone)]
pub struct UserTrades {
    units: UserTradeUnits,
}

impl UserTrades {
    pub fn new(_pool: PgPool, units: UserTradeUnits) -> Self {
        Self { units }
    }

    pub async fn persist_all<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        new_user_trades: Vec<NewUserTrade>,
    ) -> Result<(), UserTradesError> {
        if new_user_trades.is_empty() {
            return Ok(());
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO user_trades (buy_unit_id, buy_amount, sell_unit_id, sell_amount, external_ref)"
        );
        query_builder.push_values(
            new_user_trades,
            |mut builder,
             NewUserTrade {
                 is_latest: _,
                 buy_unit,
                 buy_amount,
                 sell_unit,
                 sell_amount,
                 external_ref,
             }| {
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
        query.execute(tx).await?;
        Ok(())
    }
}
