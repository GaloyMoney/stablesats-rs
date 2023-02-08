mod unit;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use tracing::instrument;
use uuid::Uuid;

use crate::error::UserTradesError;
use rust_decimal::Decimal;

pub use unit::*;

pub struct UnaccountedUserTrade {
    pub buy_unit: UserTradeUnit,
    pub buy_amount: Decimal,
    pub sell_unit: UserTradeUnit,
    pub sell_amount: Decimal,
    pub external_ref: ExternalRef,
    pub ledger_tx_id: ledger::LedgerTxId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalRef {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub btc_tx_id: String,
    pub usd_tx_id: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewUserTrade {
    pub buy_unit: UserTradeUnit,
    pub buy_amount: Decimal,
    pub sell_unit: UserTradeUnit,
    pub sell_amount: Decimal,
    pub external_ref: ExternalRef,
}

#[derive(Clone)]
pub struct UserTrades {}

impl UserTrades {
    pub fn new(_pool: PgPool) -> Self {
        Self {}
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
            "INSERT INTO user_trades (buy_unit, buy_amount, sell_unit, sell_amount, external_ref)",
        );
        query_builder.push_values(
            new_user_trades,
            |mut builder,
             NewUserTrade {
                 buy_unit,
                 buy_amount,
                 sell_unit,
                 sell_amount,
                 external_ref,
             }| {
                builder.push_bind(buy_unit);
                builder.push_bind(buy_amount);
                builder.push_bind(sell_unit);
                builder.push_bind(sell_amount);
                builder.push_bind(
                    serde_json::to_value(external_ref).expect("failed to serialize external_ref"),
                );
            },
        );
        let query = query_builder.build();
        query.execute(tx).await?;
        Ok(())
    }

    #[instrument(name = "user_trades.find_unaccounted_trade", skip_all)]
    pub async fn find_unaccounted_trade(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Option<UnaccountedUserTrade>, UserTradesError> {
        let tx_id = Uuid::new_v4();
        let trade = sqlx::query!(
            r#"UPDATE user_trades
               SET ledger_tx_id = $1
               WHERE id = (
                 SELECT id FROM user_trades WHERE ledger_tx_id IS NULL ORDER BY id LIMIT 1
               ) RETURNING id, buy_amount, buy_unit as "buy_unit: UserTradeUnit", sell_amount, sell_unit as "sell_unit: UserTradeUnit", external_ref"#,
            tx_id
        )
        .fetch_optional(&mut *tx)
        .await?;
        Ok(trade.map(|trade| UnaccountedUserTrade {
            buy_unit: trade.buy_unit,
            buy_amount: trade.buy_amount,
            sell_unit: trade.sell_unit,
            sell_amount: trade.sell_amount,
            external_ref: serde_json::from_value(trade.external_ref)
                .expect("failed to deserialize external_ref"),
            ledger_tx_id: ledger::LedgerTxId::from(tx_id),
        }))
    }
}
