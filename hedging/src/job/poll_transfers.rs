use okex_client::{OkexClient, OkexClientError};

use crate::{error::HedgingError, okex_transfers::*};

const DEPOSIT_TIMEOUT: i64 = 10;

pub async fn execute(okex_transfers: OkexTransfers, okex: OkexClient) -> Result<(), HedgingError> {
    let mut execute_sweep = false;
    for id in okex_transfers.open_non_external_deposit().await? {
        match okex.transfer_state_by_client_id(id.clone()).await {
            Ok(details) => {
                okex_transfers.update_non_external_deposit(details).await?;
            }
            Err(OkexClientError::WithdrawalIdDoesNotExist)
            | Err(OkexClientError::ParameterClientIdError) => {
                okex_transfers.mark_as_lost(id).await?;
                execute_sweep = true;
            }
            Err(res) => return Err(res.into()),
        }
    }

    for (id, address, amount, created_at) in okex_transfers.open_external_deposit().await? {
        match okex.fetch_deposit(address, amount).await {
            Ok(details) => {
                okex_transfers
                    .update_external_deposit(id, details.state, details.transaction_id)
                    .await?;
            }
            Err(OkexClientError::UnexpectedResponse { .. }) => {
                if chrono::Utc::now() - created_at > chrono::Duration::minutes(DEPOSIT_TIMEOUT) {
                    okex_transfers.mark_as_lost(id).await?;
                    execute_sweep = true;
                }
            }
            Err(res) => return Err(res.into()),
        }
    }

    if execute_sweep {
        okex_transfers.sweep_lost_records().await?;
    }
    Ok(())
}
