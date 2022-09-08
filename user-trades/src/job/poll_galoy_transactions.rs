use galoy_client::{
    GaloyClient, GaloyTransactionEdge, GaloyTransactionNode, LastTransactionCursor,
};
use sqlxmq::CurrentJob;

use crate::{error::UserTradesError, user_trades::*};

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

fn unify(galoy_transactions: Vec<GaloyTransactionEdge>) -> Vec<NewUserTrade> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unify_simple() {
        // let created_at = chrono::Utc::now();
        // let tx1 = GaloyTransactionEdge {
        //     cursor: "1".to_string(),
        //     node: GaloyTransactionNode {
        //         id: "1".to_string(),
        //         created_at,
        //     },
        // };
        // let tx2 = GaloyTransactionEdge {
        //     cursor: "2".to_string(),
        //     node: GaloyTransactionNode {
        //         id: "2".to_string(),
        //         created_at,
        //     },
        // };
        // let trades = unify(vec![tx1, tx2]);
        // assert_eq!(trades.len(), 2);
    }
}
