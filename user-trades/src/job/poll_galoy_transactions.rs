use rust_decimal::Decimal;
use tracing::{instrument, warn};

use std::collections::BTreeMap;

use galoy_client::{GaloyClient, GaloyTransaction, SettlementCurrency, TxCursor};

use crate::{
    error::UserTradesError, galoy_transactions::*, user_trade_unit::UserTradeUnit, user_trades::*,
};

#[instrument(
    name = "poll_galoy_transactions",
    skip_all,
    err,
    fields(n_galoy_txs, n_user_trades, error, error.message)
)]
pub(super) async fn execute(
    user_trades: UserTrades,
    galoy_transactions: GaloyTransactions,
    galoy: GaloyClient,
) -> Result<(), UserTradesError> {
    shared::tracing::record_error(|| async move {
        let span = tracing::Span::current();

        import_galoy_transactions(&galoy_transactions, galoy.clone()).await?;
        update_user_trades(galoy_transactions, &user_trades).await?;

        let mut latest_ref = user_trades.get_latest_ref().await?;
        let external_ref = latest_ref.take();
        let cursor = None; //external_ref.map(|ExternalRef { cursor, .. }| TxCursor::from(cursor));
        let transactions = galoy.transactions_list(cursor).await?;

        if !transactions.list.is_empty() {
            let trades = unify_galoy_transactions(transactions.list);
            span.record("n_user_trades", &tracing::field::display(trades.len()));
            user_trades
                .persist_all(latest_ref, trades.into_iter().rev())
                .await?;
        }
        Ok(())
    })
    .await
}

#[instrument(skip_all, err, fields(n_galoy_txs))]
async fn import_galoy_transactions(
    galoy_transactions: &GaloyTransactions,
    galoy: GaloyClient,
) -> Result<(), UserTradesError> {
    let mut latest_cursor = galoy_transactions.get_latest_cursor().await?;
    let transactions = galoy
        .transactions_list(latest_cursor.take().map(TxCursor::from))
        .await?;
    tracing::Span::current().record(
        "n_galoy_txs",
        &tracing::field::display(transactions.list.len()),
    );
    if !transactions.list.is_empty() {
        galoy_transactions
            .persist_all(latest_cursor, transactions.list)
            .await?;
    }
    Ok(())
}

#[instrument(skip_all, err, fields(n_unpaired_txs))]
async fn update_user_trades(
    galoy_transactions: GaloyTransactions,
    user_trades: &UserTrades,
) -> Result<(), UserTradesError> {
    let UnpairedTransactions { list, mut tx } =
        galoy_transactions.list_unpaired_transactions().await?;
    if list.is_empty() {
        return Ok(());
    }
    let (trades, paired_ids) = unify(list);
    galoy_transactions
        .update_paired_ids(&mut tx, paired_ids)
        .await?;
    tx.commit().await?;
    Ok(())
}

fn unify(unpaired_transactions: Vec<UnpairedTransaction>) -> (Vec<NewUserTrade>, Vec<String>) {
    let mut txs: BTreeMap<_, _> = unpaired_transactions.into_iter().enumerate().collect();
    let mut user_trades = Vec::new();
    let mut is_latest = Some(true);
    let mut unpaired = 0;
    let mut paired_ids = Vec::new();
    for idx in 0..txs.len() {
        if txs.is_empty() {
            break;
        }
        if let Some(tx) = txs.remove(&idx) {
            let idx = if let Some((idx, _)) = txs.iter().find(|(_, other)| is_pair(&tx, other)) {
                *idx
            } else {
                warn!({ transaction = ?tx, tx_idx = idx }, "no pair for galoy transaction");
                unpaired += 1;
                continue;
            };
            let other = txs.remove(&idx).unwrap();
            let external_ref = if tx.settlement_currency == SettlementCurrency::BTC {
                ExternalRef {
                    timestamp: tx.created_at,
                    btc_tx_id: tx.id,
                    usd_tx_id: other.id,
                }
            } else {
                ExternalRef {
                    timestamp: tx.created_at,
                    btc_tx_id: other.id,
                    usd_tx_id: tx.id,
                }
            };
            paired_ids.push(external_ref.btc_tx_id.clone());
            paired_ids.push(external_ref.usd_tx_id.clone());
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
    tracing::Span::current().record("n_unpaired_txs", &tracing::field::display(unpaired));
    (user_trades, paired_ids)
}
fn unify_galoy_transactions(galoy_transactions: Vec<GaloyTransaction>) -> Vec<NewUserTrade> {
    let mut txs: BTreeMap<usize, GaloyTransaction> =
        galoy_transactions.into_iter().enumerate().collect();
    let mut user_trades = Vec::new();
    let mut is_latest = Some(true);
    for idx in 0..txs.len() {
        if txs.is_empty() {
            break;
        }
        if let Some(tx) = txs.remove(&idx) {
            let idx =
                if let Some((idx, _)) = txs.iter().find(|(_, other)| is_pair_galoy(&tx, other)) {
                    *idx
                } else {
                    unimplemented!();
                };
            let other = txs.remove(&idx).unwrap();
            let external_ref = if tx.settlement_currency == SettlementCurrency::BTC {
                ExternalRef {
                    timestamp: tx.created_at,
                    btc_tx_id: tx.id,
                    usd_tx_id: other.id,
                }
            } else {
                ExternalRef {
                    timestamp: tx.created_at,
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

fn is_pair(tx1: &UnpairedTransaction, tx2: &UnpairedTransaction) -> bool {
    if tx1.created_at != tx2.created_at || tx1.settlement_currency == tx2.settlement_currency {
        return false;
    }

    if tx1.settlement_currency == SettlementCurrency::BTC {
        tx1.amount_in_usd_cents.abs() == tx2.settlement_amount.abs()
    } else {
        tx2.amount_in_usd_cents.abs() == tx1.settlement_amount.abs()
    }
}

fn is_pair_galoy(tx1: &GaloyTransaction, tx2: &GaloyTransaction) -> bool {
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
    use galoy_client::{SettlementCurrency, SettlementMethod, TxStatus};
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
            settlement_method: SettlementMethod::SettlementViaIntraLedger,
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
            settlement_method: SettlementMethod::SettlementViaIntraLedger,
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
            settlement_method: SettlementMethod::SettlementViaIntraLedger,
            amount_in_usd_cents: dec!(-10),
            cents_per_unit: dec!(0.01),
            status: TxStatus::SUCCESS,
        };
        let tx4 = GaloyTransaction {
            cursor: TxCursor::from("4".to_string()),
            id: "id4".to_string(),
            created_at: created_earlier,
            settlement_amount: dec!(10),
            settlement_method: SettlementMethod::SettlementViaIntraLedger,
            settlement_currency: SettlementCurrency::USD,
            amount_in_usd_cents: dec!(10),
            cents_per_unit: dec!(1),
            status: TxStatus::SUCCESS,
        };
        let trades = unify_galoy_transactions(vec![tx1, tx2, tx3, tx4]);
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
                    btc_tx_id: "id3".to_string(),
                    usd_tx_id: "id4".to_string(),
                }),
            }
        );
    }
}
