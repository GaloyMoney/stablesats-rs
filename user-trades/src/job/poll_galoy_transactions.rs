use galoy_client::{GaloyClient, LastTransactionCursor};
use sqlxmq::CurrentJob;

use crate::{
    error::UserTradesError, galoy_transactions::GaloyTransactions, user_trades::UserTrades,
};

pub(super) async fn execute(
    mut current_job: CurrentJob,
    galoy_transactions: GaloyTransactions,
    user_trades: UserTrades,
    mut galoy: GaloyClient,
) -> Result<(), UserTradesError> {
    // 1. Get the last_tx_cursor from user_trades (the user_trades table is initially seeded)
    let latest_cursor = user_trades.latest_tx_cursor().await?;
    // 2. Get transactions from galoy_client
    let transactions = galoy
        .transactions_list(4, Some(LastTransactionCursor::from(latest_cursor)))
        .await?;
    // 3. Unify transaction pairs
    // 4. Persist vector of NewUserTrade to user_trades
    Ok(())
}
