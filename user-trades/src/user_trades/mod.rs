mod unit;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use std::collections::HashMap;
use tracing::instrument;
use uuid::{uuid, Uuid};

use crate::error::UserTradesError;
use rust_decimal::Decimal;

pub use unit::*;

pub const BAD_TRADE_MARKER: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

pub struct UnaccountedUserTrade {
    pub buy_unit: UserTradeUnit,
    pub buy_amount: Decimal,
    pub sell_unit: UserTradeUnit,
    pub sell_amount: Decimal,
    pub external_ref: ExternalRef,
    pub ledger_tx_id: ledger::LedgerTxId,
}

pub struct UserTradeNeedingRevert {
    pub buy_unit: UserTradeUnit,
    pub buy_amount: Decimal,
    pub sell_unit: UserTradeUnit,
    pub sell_amount: Decimal,
    pub external_ref: ExternalRef,
    pub ledger_tx_id: ledger::LedgerTxId,
    pub correction_ledger_tx_id: ledger::LedgerTxId,
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

#[derive(Debug)]
pub struct PairedTradesLookup {
    pub usd_to_btc: HashMap<String, (i32, String)>,
    pub btc_to_usd: HashMap<String, (i32, String)>,
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

    #[instrument(name = "user_trades.mark_bad_trades", skip_all)]
    pub async fn mark_bad_trades<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        bad_ids: Vec<i32>,
    ) -> Result<(), UserTradesError> {
        if bad_ids.is_empty() {
            return Ok(());
        }
        sqlx::query!(
            "UPDATE user_trades SET correction_ledger_tx_id = $1 WHERE id = ANY($2) AND correction_ledger_tx_id IS NULL",
            BAD_TRADE_MARKER,
            &bad_ids[..]
        )
        .execute(tx)
        .await?;
        Ok(())
    }

    #[instrument(name = "user_trades.find_already_paired_trades", skip_all)]
    pub async fn find_already_paired_trades<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        ids: Vec<String>,
    ) -> Result<PairedTradesLookup, UserTradesError> {
        let rows = sqlx::query!(
            "SELECT id, external_ref->>'btc_tx_id' AS btc_id, external_ref->>'usd_tx_id' AS usd_id FROM user_trades WHERE external_ref->>'btc_tx_id' = ANY($1) AND correction_ledger_tx_id IS NULL
             UNION
             SELECT id, external_ref->>'btc_tx_id' AS btc_id, external_ref->>'usd_tx_id' AS usd_id FROM user_trades WHERE external_ref->>'usd_tx_id' = ANY($1) AND correction_ledger_tx_id IS NULL",
            &ids[..]
        ).fetch_all(&mut *tx)
            .await?;
        let usd_to_btc: HashMap<String, (i32, String)> = rows
            .into_iter()
            .filter_map(|row| {
                if let (Some(usd), Some(btc)) = (row.usd_id, row.btc_id) {
                    Some((usd, (row.id.expect("id is always present"), btc)))
                } else {
                    None
                }
            })
            .collect();
        let btc_to_usd = usd_to_btc
            .iter()
            .map(|(usd, (id, btc))| (btc.clone(), (*id, usd.clone())))
            .collect();
        Ok(PairedTradesLookup {
            usd_to_btc,
            btc_to_usd,
        })
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
                 SELECT id FROM user_trades WHERE ledger_tx_id IS NULL ORDER BY external_ref->>'timestamp' LIMIT 1
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

    #[instrument(name = "user_trades.find_trade_needing_revert", skip_all)]
    pub async fn find_trade_needing_revert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Option<UserTradeNeedingRevert>, UserTradesError> {
        let tx_id = Uuid::new_v4();
        let trade = sqlx::query!(
            r#"UPDATE user_trades
               SET correction_ledger_tx_id = $1
               WHERE id = (
                 SELECT id FROM user_trades WHERE ledger_tx_id IS NOT NULL AND correction_ledger_tx_id = $2 ORDER BY external_ref->>'timestamp' LIMIT 1
               ) RETURNING id, ledger_tx_id, buy_amount, buy_unit as "buy_unit: UserTradeUnit", sell_amount, sell_unit as "sell_unit: UserTradeUnit", external_ref"#,
            tx_id,
            BAD_TRADE_MARKER
        )
        .fetch_optional(&mut *tx)
        .await?;
        Ok(trade.map(|trade| UserTradeNeedingRevert {
            buy_unit: trade.buy_unit,
            buy_amount: trade.buy_amount,
            sell_unit: trade.sell_unit,
            sell_amount: trade.sell_amount,
            external_ref: serde_json::from_value(trade.external_ref)
                .expect("failed to deserialize external_ref"),
            ledger_tx_id: trade
                .ledger_tx_id
                .expect("ledger_tx_id is always present")
                .into(),
            correction_ledger_tx_id: ledger::LedgerTxId::from(tx_id),
        }))
    }
}
