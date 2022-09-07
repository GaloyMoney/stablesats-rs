use galoy_client::GaloyClient;
use sqlxmq::CurrentJob;

use crate::{error::UserTradesError, user_trades::UserTrades};

pub(super) async fn execute(
    mut current_job: CurrentJob,
    user_trades: UserTrades,
    galoy: GaloyClient,
) -> Result<(), UserTradesError> {
    // get latest cursor from GaloyTransactions
    // call galoy client passing cursor
    // unify pairs
    // persist each new transaction in GaloyTransactions
    // aggregate all new GaloyTransactions into 1 UserTrade
    Ok(())
}
