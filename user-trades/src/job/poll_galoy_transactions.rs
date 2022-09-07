use galoy_client::{GaloyClient, LastTransactionCursor};
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
    // get latest cursor from GaloyTransactions
    // call galoy client passing cursor
    // unify pairs
    // persist each new transaction in GaloyTransactions
    // aggregate all new GaloyTransactions into 1 UserTrade
    Ok(())
}
