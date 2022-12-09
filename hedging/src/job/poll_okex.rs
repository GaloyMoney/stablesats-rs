use okex_client::{OkexClient, OkexClientError, PositionSize};
use shared::{
    payload::{
        ExchangeIdRaw, InstrumentIdRaw, OkexBtcUsdSwapPositionPayload, SyntheticCentExposure,
        OKEX_EXCHANGE_ID,
    },
    pubsub::Publisher,
};

use crate::{app::FundingSectionConfig, error::HedgingError, okex_orders::*, okex_transfers::*};

pub async fn execute(
    okex_orders: OkexOrders,
    okex_transfers: OkexTransfers,
    okex: OkexClient,
    publisher: Publisher,
    funding_config: FundingSectionConfig,
) -> Result<(), HedgingError> {
    let PositionSize {
        usd_cents,
        instrument_id,
        ..
    } = okex.get_position_in_signed_usd_cents().await?;
    publisher
        .publish(OkexBtcUsdSwapPositionPayload {
            exchange: ExchangeIdRaw::from(OKEX_EXCHANGE_ID),
            instrument_id: InstrumentIdRaw::from(instrument_id.to_string()),
            signed_usd_exposure: SyntheticCentExposure::from(usd_cents),
        })
        .await?;

    let mut execute_sweep = false;
    for id in okex_orders.open_orders().await? {
        match okex.order_details(id.clone()).await {
            Ok(details) => {
                okex_orders.update_order(details).await?;
            }
            Err(OkexClientError::OrderDoesNotExist) => {
                okex_orders.mark_as_lost(id).await?;
                execute_sweep = true;
            }
            Err(res) => return Err(res.into()),
        }
    }

    if execute_sweep {
        okex_orders.sweep_lost_records().await?;
    }

    let mut execute_transfer_sweep = false;
    for id in okex_transfers.open_non_external_deposit().await? {
        match okex.transfer_state_by_client_id(id.clone()).await {
            Ok(details) => {
                okex_transfers.update_non_external_deposit(details).await?;
            }
            Err(OkexClientError::WithdrawalIdDoesNotExist)
            | Err(OkexClientError::ParameterClientIdError) => {
                okex_transfers.mark_as_lost(id).await?;
                execute_transfer_sweep = true;
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
                if chrono::Utc::now() - created_at > funding_config.deposit_lost_timeout_seconds {
                    okex_transfers.mark_as_lost(id).await?;
                    execute_transfer_sweep = true;
                }
            }
            Err(res) => return Err(res.into()),
        }
    }

    if execute_transfer_sweep {
        okex_transfers.sweep_lost_records().await?;
    }

    Ok(())
}
