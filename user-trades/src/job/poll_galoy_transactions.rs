use galoy_client::{GaloyClient, GaloyTransaction, SettlementCurrency, TxCursor};
use rust_decimal::Decimal;
use sqlxmq::CurrentJob;
use tracing::instrument;

use std::collections::BTreeMap;

use crate::{error::UserTradesError, user_trade_unit::UserTradeUnit, user_trades::*};

#[instrument(
    name = "poll_galoy_transactions",
    skip_all,
    err,
    fields(n_galoy_txs, n_user_trades, error, error.message)
)]
pub(super) async fn execute(
    current_job: &mut CurrentJob,
    user_trades: UserTrades,
    galoy: GaloyClient,
) -> Result<(), UserTradesError> {
    shared::tracing::record_error(|| async move {
        let span = tracing::Span::current();
        let mut latest_ref = user_trades.get_latest_ref().await?;
        let external_ref = latest_ref.take();
        let cursor = external_ref.map(|ExternalRef { cursor, .. }| TxCursor::from(cursor));
        let transactions = galoy.transactions_list(cursor).await?;

        span.record(
            "n_galoy_txs",
            &tracing::field::display(transactions.list.len()),
        );

        if !transactions.list.is_empty() {
            let trades = unify(transactions.list);
            span.record("n_user_trades", &tracing::field::display(trades.len()));
            user_trades
                .persist_all(latest_ref, trades.into_iter().rev())
                .await?;
        }

        current_job.complete().await?;
        Ok(())
    })
    .await
}

fn unify(galoy_transactions: Vec<GaloyTransaction>) -> Vec<NewUserTrade> {
    let mut txs: BTreeMap<usize, GaloyTransaction> =
        galoy_transactions.into_iter().enumerate().collect();
    let mut user_trades = Vec::new();
    let mut is_latest = Some(true);
    for idx in 0..txs.len() {
        if txs.is_empty() {
            break;
        }
        if let Some(tx) = txs.remove(&idx) {
            let idx = if let Some((idx, _)) = txs.iter().find(|(_, other)| is_pair(&tx, other)) {
                *idx
            } else {
                unimplemented!()
            };
            let other = txs.remove(&idx).unwrap();
            let external_ref = if tx.settlement_currency == SettlementCurrency::BTC {
                ExternalRef {
                    timestamp: tx.created_at,
                    cursor: tx.cursor.into(),
                    btc_tx_id: tx.id,
                    usd_tx_id: other.id,
                }
            } else {
                ExternalRef {
                    timestamp: tx.created_at,
                    cursor: tx.cursor.into(),
                    btc_tx_id: other.id,
                    usd_tx_id: tx.id,
                }
            };
            if tx.settlement_amount < Decimal::ZERO {
                user_trades.push(NewUserTrade {
                    is_latest,
                    buy_unit: tx.settlement_currency.into(),
                    buy_amount: tx.settlement_amount.abs(),
                    sell_unit: other.settlement_currency.into(),
                    sell_amount: other.settlement_amount.abs(),
                    external_ref: Some(external_ref),
                });
            } else {
                user_trades.push(NewUserTrade {
                    is_latest,
                    buy_unit: other.settlement_currency.into(),
                    buy_amount: other.settlement_amount.abs(),
                    sell_unit: tx.settlement_currency.into(),
                    sell_amount: tx.settlement_amount.abs(),
                    external_ref: Some(external_ref),
                });
            }
            is_latest = None;
        }
    }
    user_trades
}

fn is_pair(tx1: &GaloyTransaction, tx2: &GaloyTransaction) -> bool {
    if tx1.created_at != tx2.created_at || tx1.settlement_currency == tx2.settlement_currency {
        return false;
    }

    if tx1.settlement_currency == SettlementCurrency::BTC {
        tx1.amount_in_usd_cents.abs() == tx2.settlement_amount.abs()
    } else {
        tx2.amount_in_usd_cents.abs() == tx1.settlement_amount.abs()
    }
}

impl From<SettlementCurrency> for UserTradeUnit {
    fn from(currency: SettlementCurrency) -> Self {
        match currency {
            SettlementCurrency::BTC => Self::Satoshi,
            SettlementCurrency::USD => Self::SynthCent,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use galoy_client::{SettlementCurrency, TxStatus};
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn unify_transactions() {
        let created_at = chrono::Utc::now();
        let created_earlier = created_at - chrono::Duration::days(1);
        let tx1 = GaloyTransaction {
            cursor: TxCursor::from("1".to_string()),
            id: "id1".to_string(),
            created_at,
            settlement_amount: dec!(1000),
            settlement_currency: SettlementCurrency::BTC,
            amount_in_usd_cents: dec!(10),
            cents_per_unit: dec!(0.01),
            status: TxStatus::SUCCESS,
        };
        let tx2 = GaloyTransaction {
            cursor: TxCursor::from("2".to_string()),
            id: "id2".to_string(),
            created_at,
            settlement_amount: dec!(-10),
            settlement_currency: SettlementCurrency::USD,
            amount_in_usd_cents: dec!(-10),
            cents_per_unit: dec!(1),
            status: TxStatus::SUCCESS,
        };
        let tx3 = GaloyTransaction {
            cursor: TxCursor::from("3".to_string()),
            id: "id3".to_string(),
            created_at: created_earlier,
            settlement_amount: dec!(-1000),
            settlement_currency: SettlementCurrency::BTC,
            amount_in_usd_cents: dec!(-10),
            cents_per_unit: dec!(0.01),
            status: TxStatus::SUCCESS,
        };
        let tx4 = GaloyTransaction {
            cursor: TxCursor::from("4".to_string()),
            id: "id4".to_string(),
            created_at: created_earlier,
            settlement_amount: dec!(10),
            settlement_currency: SettlementCurrency::USD,
            amount_in_usd_cents: dec!(10),
            cents_per_unit: dec!(1),
            status: TxStatus::SUCCESS,
        };
        let trades = unify(vec![tx1, tx2, tx3, tx4]);
        assert!(trades.len() == 2);
        let (trade1, trade2) = (trades.first().unwrap(), trades.last().unwrap());
        assert_eq!(
            trade1,
            &NewUserTrade {
                is_latest: Some(true),
                buy_unit: UserTradeUnit::SynthCent,
                buy_amount: dec!(10),
                sell_unit: UserTradeUnit::Satoshi,
                sell_amount: dec!(1000),
                external_ref: Some(ExternalRef {
                    timestamp: created_at,
                    cursor: "1".to_string(),
                    btc_tx_id: "id1".to_string(),
                    usd_tx_id: "id2".to_string(),
                }),
            }
        );
        assert_eq!(
            trade2,
            &NewUserTrade {
                is_latest: None,
                buy_unit: UserTradeUnit::Satoshi,
                buy_amount: dec!(1000),
                sell_unit: UserTradeUnit::SynthCent,
                sell_amount: dec!(10),
                external_ref: Some(ExternalRef {
                    timestamp: created_earlier,
                    cursor: "3".to_string(),
                    btc_tx_id: "id3".to_string(),
                    usd_tx_id: "id4".to_string(),
                }),
            }
        );
    }
}
