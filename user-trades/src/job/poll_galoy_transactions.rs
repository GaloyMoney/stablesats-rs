use galoy_client::GaloyClient;
use sqlxmq::CurrentJob;

use crate::{
    error::UserTradesError, galoy_transactions::GaloyTransactions, user_trades::UserTrades,
};

pub(super) async fn execute(
    mut current_job: CurrentJob,
    galoy_transactions: GaloyTransactions,
    user_trades: UserTrades,
    galoy: GaloyClient,
) -> Result<(), UserTradesError> {
    Ok(())
}
