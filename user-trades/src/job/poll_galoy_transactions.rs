use rust_decimal::Decimal;
use tracing::{instrument, warn};

use std::collections::BTreeMap;

use galoy_client::{GaloyClient, SettlementCurrency, TxCursor};

use crate::{error::UserTradesError, galoy_transactions::*, user_trades::*};

#[instrument(
    name = "user_trades.job.poll_galoy_transactions",
    skip_all,
    err,
    fields(n_galoy_txs, n_unpaired_txs, n_user_trades, has_more)
)]
pub(super) async fn execute(
    pool: &sqlx::PgPool,
    user_trades: &UserTrades,
    galoy_transactions: &GaloyTransactions,
    galoy: &GaloyClient,
    ledger: &ledger::Ledger,
) -> Result<bool, UserTradesError> {
    let has_more = import_galoy_transactions(galoy_transactions, galoy.clone()).await?;
    update_user_trades(galoy_transactions, user_trades).await?;
    update_ledger(pool, user_trades, ledger).await?;

    Ok(has_more)
}

async fn import_galoy_transactions(
    galoy_transactions: &GaloyTransactions,
    galoy: GaloyClient,
) -> Result<bool, UserTradesError> {
    let latest_cursor = galoy_transactions.get_latest_cursor().await?;
    let transactions = galoy
        .transactions_list(latest_cursor.map(|c| TxCursor::from(c.0)))
        .await?;
    tracing::Span::current().record(
        "n_galoy_txs",
        &tracing::field::display(transactions.list.len()),
    );
    tracing::Span::current().record("has_more", &tracing::field::display(transactions.has_more));
    if !transactions.list.is_empty() {
        galoy_transactions.persist_all(transactions.list).await?;
    }
    Ok(transactions.has_more)
}

async fn update_user_trades(
    galoy_transactions: &GaloyTransactions,
    user_trades: &UserTrades,
) -> Result<(), UserTradesError> {
    let UnpairedTransactions { list, mut tx } =
        galoy_transactions.list_unpaired_transactions().await?;
    if list.is_empty() {
        return Ok(());
    }
    let (trades, paired_ids) = unify(list);
    galoy_transactions
        .update_paired_ids(&mut tx, &paired_ids)
        .await?;
    tracing::Span::current().record("n_user_trades", &tracing::field::display(trades.len()));
    user_trades.persist_all(&mut tx, trades).await?;
    tx.commit().await?;
    Ok(())
}

async fn update_ledger(
    pool: &sqlx::PgPool,
    user_trades: &UserTrades,
    ledger: &ledger::Ledger,
) -> Result<(), UserTradesError> {
    loop {
        let mut tx = pool.begin().await?;
        if let Ok(Some(UnaccountedUserTrade {
            buy_unit,
            buy_amount,
            sell_amount,
            external_ref,
            ledger_tx_id,
            ..
        })) = user_trades.find_unaccounted_trade(&mut tx).await
        {
            if buy_unit == UserTradeUnit::UsdCent {
                ledger
                    .user_buys_usd(
                        tx,
                        ledger_tx_id,
                        ledger::UserBuysUsdParams {
                            satoshi_amount: sell_amount,
                            usd_cents_amount: buy_amount,
                            meta: ledger::UserBuysUsdMeta {
                                timestamp: external_ref.timestamp,
                                btc_tx_id: external_ref.btc_tx_id,
                                usd_tx_id: external_ref.usd_tx_id,
                            },
                        },
                    )
                    .await?;
            } else {
                ledger
                    .user_sells_usd(
                        tx,
                        ledger_tx_id,
                        ledger::UserSellsUsdParams {
                            satoshi_amount: buy_amount,
                            usd_cents_amount: sell_amount,
                            meta: ledger::UserSellsUsdMeta {
                                timestamp: external_ref.timestamp,
                                btc_tx_id: external_ref.btc_tx_id,
                                usd_tx_id: external_ref.usd_tx_id,
                            },
                        },
                    )
                    .await?;
            }
        } else {
            break;
        }
    }
    Ok(())
}

fn unify(unpaired_transactions: Vec<UnpairedTransaction>) -> (Vec<NewUserTrade>, Vec<String>) {
    let mut txs: BTreeMap<_, _> = unpaired_transactions.into_iter().enumerate().collect();
    let mut user_trades = Vec::new();
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
                    buy_unit: tx.settlement_currency.into(),
                    buy_amount: tx.settlement_amount.abs(),
                    sell_unit: other.settlement_currency.into(),
                    sell_amount: other.settlement_amount.abs(),
                    external_ref,
                });
            } else {
                user_trades.push(NewUserTrade {
                    buy_unit: other.settlement_currency.into(),
                    buy_amount: other.settlement_amount.abs(),
                    sell_unit: tx.settlement_currency.into(),
                    sell_amount: tx.settlement_amount.abs(),
                    external_ref,
                });
            }
        }
    }
    tracing::Span::current().record("n_unpaired_txs", &tracing::field::display(unpaired));
    (user_trades, paired_ids)
}

fn is_pair(tx1: &UnpairedTransaction, tx2: &UnpairedTransaction) -> bool {
    tx1.created_at == tx2.created_at
        && tx1.settlement_currency != tx2.settlement_currency
        && tx1.direction != tx2.direction
        && tx1.settlement_method == tx2.settlement_method
        && (tx1.amount_in_usd_cents.abs() - tx2.amount_in_usd_cents.abs()).abs() <= Decimal::ONE
}

impl From<SettlementCurrency> for UserTradeUnit {
    fn from(currency: SettlementCurrency) -> Self {
        match currency {
            SettlementCurrency::BTC => Self::Satoshi,
            SettlementCurrency::USD => Self::UsdCent,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use galoy_client::SettlementCurrency;
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn unify_transactions() {
        let created_at = chrono::Utc::now();
        let created_earlier = created_at - chrono::Duration::days(1);
        let tx1 = UnpairedTransaction {
            id: "id1".to_string(),
            created_at,
            settlement_amount: dec!(1000),
            settlement_currency: SettlementCurrency::BTC,
            settlement_method: format!("ln"),
            direction: format!("RECEIVE"),
            amount_in_usd_cents: dec!(10),
        };
        let tx2 = UnpairedTransaction {
            id: "id2".to_string(),
            created_at,
            settlement_amount: dec!(-10),
            settlement_currency: SettlementCurrency::USD,
            settlement_method: format!("ln"),
            direction: format!("SEND"),
            amount_in_usd_cents: dec!(10),
        };
        let tx3 = UnpairedTransaction {
            id: "id3".to_string(),
            created_at: created_earlier,
            settlement_amount: dec!(-1000),
            settlement_method: format!("ln"),
            settlement_currency: SettlementCurrency::BTC,
            direction: format!("SEND"),
            amount_in_usd_cents: dec!(10),
        };
        let tx4 = UnpairedTransaction {
            id: "id4".to_string(),
            created_at: created_earlier,
            settlement_amount: dec!(10),
            settlement_method: format!("ln"),
            settlement_currency: SettlementCurrency::USD,
            direction: format!("RECEIVE"),
            amount_in_usd_cents: dec!(10),
        };
        let unpaired = UnpairedTransaction {
            id: "unpaired".to_string(),
            created_at: created_earlier,
            settlement_amount: dec!(10),
            settlement_currency: SettlementCurrency::USD,
            settlement_method: format!("ln"),
            direction: format!("RECEIVE"),
            amount_in_usd_cents: dec!(10),
        };
        let unpaired_txs = vec![tx1, tx2, tx3, tx4, unpaired];
        let (trades, ids) = unify(unpaired_txs.clone());
        for tx in unpaired_txs[0..4].iter() {
            assert!(ids.contains(&tx.id));
        }
        assert!(ids.len() == 4);
        assert!(trades.len() == 2);
        let (trade1, trade2) = (trades.first().unwrap(), trades.last().unwrap());
        assert_eq!(
            trade1,
            &NewUserTrade {
                buy_unit: UserTradeUnit::UsdCent,
                buy_amount: dec!(10),
                sell_unit: UserTradeUnit::Satoshi,
                sell_amount: dec!(1000),
                external_ref: ExternalRef {
                    timestamp: created_at,
                    btc_tx_id: "id1".to_string(),
                    usd_tx_id: "id2".to_string(),
                },
            }
        );
        assert_eq!(
            trade2,
            &NewUserTrade {
                buy_unit: UserTradeUnit::Satoshi,
                buy_amount: dec!(1000),
                sell_unit: UserTradeUnit::UsdCent,
                sell_amount: dec!(10),
                external_ref: ExternalRef {
                    timestamp: created_earlier,
                    btc_tx_id: "id3".to_string(),
                    usd_tx_id: "id4".to_string(),
                },
            }
        );
    }
}
