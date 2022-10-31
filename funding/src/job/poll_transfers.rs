use okex_client::{OkexClient, OkexClientError};

use crate::{error::FundingError, okex_transfers::*};

pub async fn execute(okex_transfers: OkexTransfers, okex: OkexClient) -> Result<(), FundingError> {
    let mut execute_sweep = false;
    for id in okex_transfers.open_transfers().await? {
        match okex.transfer_state_by_client_id(id.clone()).await {
            Ok(details) => {
                okex_transfers.update_transfer(details).await?;
            }
            Err(OkexClientError::WithdrawalIdDoesNotExist)
            | Err(OkexClientError::ParameterClientIdError) => {
                okex_transfers.mark_as_lost(id).await?;
                execute_sweep = true;
            }
            Err(res) => return Err(res.into()),
        }
    }

    if execute_sweep {
        okex_transfers.sweep_lost_records().await?;
    }
    Ok(())
}
