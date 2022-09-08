use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use sqlxmq::CurrentJob;
use std::{collections::HashMap, convert::TryInto};

use crate::{error::UserTradesError, user_trades::*};
use galoy_client::{
    queries::{SettlementPrice, TxDirection},
    GaloyClient, GaloyTransactionEdge, LastTransactionCursor, SettlementCurrency,
};

pub(super) async fn execute(
    mut current_job: CurrentJob,
    user_trades: UserTrades,
    galoy: GaloyClient,
) -> Result<(), UserTradesError> {
    let mut latest_ref = user_trades.get_latest_ref().await?;
    let external_ref = latest_ref.take();
    let cursor = external_ref.map(|ExternalRef { cursor, .. }| LastTransactionCursor::from(cursor));
    let transactions = galoy.transactions_list(cursor).await?;

    let user_trades = unify(transactions.list);

    // unify pairs

    // persist each new transaction in GaloyTransactions
    // aggregate all new GaloyTransactions into 1 UserTrade
    Ok(())
}

fn coincident_transactions<'a>(
    galoy_transactions: &'a Vec<GaloyTransactionEdge>,
) -> HashMap<i64, Vec<&'a GaloyTransactionEdge>> {
    let mut transactions: HashMap<i64, _> = HashMap::new();

    // 1. Collect all transactions created at the same time
    for tx in galoy_transactions.iter() {
        let timestamp = tx.node.created_at;
        let key_exists = transactions.contains_key(&timestamp);
        if !key_exists {
            transactions.insert(timestamp, Vec::new());
            transactions.get_mut(&timestamp).unwrap().push(tx);
        } else {
            transactions.get_mut(&timestamp).unwrap().push(tx);
        }
    }

    transactions
}

fn is_matching_pair(btc_tx: &GaloyTransactionEdge, usd_tx: &GaloyTransactionEdge) -> bool {
    let SettlementPrice { base, offset, .. } = btc_tx.node.settlement_price.clone();
    let base = Decimal::new(base, 0);
    let offset = dec!(10).powi(offset);
    let amount_in_usd = (btc_tx.node.settlement_amount * base) / offset;

    if amount_in_usd == usd_tx.node.settlement_amount.abs() {
        return true;
    }

    false
}

fn unify(
    galoy_transactions: Vec<GaloyTransactionEdge>,
) -> Result<Vec<NewUserTrade>, UserTradesError> {
    if galoy_transactions.len() == 0 {
        return Err(UserTradesError::Unify(
            "Can't unify empty transactions vector".to_string(),
        ));
    }

    let same_time_txs = coincident_transactions(&galoy_transactions);

    // 2. Check the equality of settlement price
    let mut btc_transactions = Vec::new();
    let mut usd_transactions = Vec::new();

    for (_, value) in same_time_txs.into_iter() {
        for tx in value {
            let node = tx.node.clone();

            match node.settlement_currency {
                SettlementCurrency::BTC => btc_transactions.push(tx),
                SettlementCurrency::USD => usd_transactions.push(tx),
                _ => (),
            }
        }
    }

    let mut unified_pairs = Vec::new();
    for tx_b in btc_transactions.iter() {
        for tx_u in usd_transactions.iter() {
            if is_matching_pair(tx_b, tx_u) {
                // create NewUserTrade
                if tx_u.node.direction == TxDirection::RECEIVE {
                    let new_user_trade = NewUserTrade {
                        is_latest: true,
                        buy_unit: tx_u.node.settlement_currency.clone().try_into()?,
                        buy_amount: tx_u.node.settlement_amount,
                        sell_unit: tx_b.node.settlement_currency.clone().try_into()?,
                        sell_amount: tx_b.node.settlement_amount,
                        external_ref: Some(ExternalRef {
                            cursor: tx_b.cursor.clone(), // revisit
                            btc_tx_id: tx_b.node.id.clone(),
                            usd_tx_id: tx_u.node.id.clone(),
                        }),
                    };

                    unified_pairs.push(new_user_trade)
                } else if tx_u.node.direction == TxDirection::SEND {
                    let new_user_trade = NewUserTrade {
                        is_latest: true,
                        buy_unit: tx_b.node.settlement_currency.clone().try_into()?,
                        buy_amount: tx_b.node.settlement_amount,
                        sell_unit: tx_u.node.settlement_currency.clone().try_into()?,
                        sell_amount: tx_u.node.settlement_amount,
                        external_ref: Some(ExternalRef {
                            cursor: tx_b.cursor.clone(),
                            btc_tx_id: tx_b.node.id.clone(),
                            usd_tx_id: tx_u.node.id.clone(),
                        }),
                    };

                    unified_pairs.push(new_user_trade)
                } else {
                    // ignore any transaction direction that isn't RECEIVE or SEND
                    continue;
                }
            }
        }
    }

    Ok(unified_pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use galoy_client::{
        queries::{
            ExchangeCurrencyUnit, InitiationVia, SettlementPrice, SettlementVia, TxDirection,
            TxStatus,
        },
        GaloyTransactionNode, SettlementCurrency,
    };
    use rust_decimal_macros::dec;

    #[test]
    fn unify_simple() {
        let created_at = chrono::Utc::now().timestamp();
        let tx1 = GaloyTransactionEdge {
            cursor: "1".to_string(),
            node: GaloyTransactionNode {
                id: "1".to_string(),
                created_at,
                direction: TxDirection::RECEIVE,
                initiation_via: InitiationVia::InitiationViaIntraLedger,
                memo: None,
                settlement_amount: dec!(190253),
                settlement_fee: dec!(0),
                settlement_price: SettlementPrice {
                    base: 19999684630,
                    currency_unit: ExchangeCurrencyUnit::USDCENT,
                    formatted_amount: "0.019999684630465746".to_string(),
                    offset: 12,
                },
                settlement_currency: SettlementCurrency::BTC,
                settlement_via: SettlementVia::SettlementViaIntraLedger,
                status: TxStatus::SUCCESS,
            },
        };

        let tx2 = GaloyTransactionEdge {
            cursor: "1".to_string(),
            node: GaloyTransactionNode {
                id: "2".to_string(),
                created_at,
                direction: TxDirection::SEND,
                initiation_via: InitiationVia::InitiationViaIntraLedger,
                memo: None,
                settlement_amount: dec!(-3805),
                settlement_fee: dec!(0),
                settlement_price: SettlementPrice {
                    base: 1000000000000,
                    currency_unit: ExchangeCurrencyUnit::USDCENT,
                    formatted_amount: "0.9999999999999999".to_string(),
                    offset: 12,
                },
                settlement_currency: SettlementCurrency::USD,
                settlement_via: SettlementVia::SettlementViaIntraLedger,
                status: TxStatus::SUCCESS,
            },
        };

        let trades = unify(vec![tx1, tx2]).unwrap();
        println!("{:?}", trades);
        assert_eq!(trades.len(), 1);
    }
}
