use okex_client::{OkexClient, OkexClientError, PositionSize};
use shared::{
    payload::{
        ExchangeIdRaw, InstrumentIdRaw, OkexBtcUsdSwapPositionPayload, SyntheticCentExposure,
        OKEX_EXCHANGE_ID,
    },
    pubsub::Publisher,
};

use crate::{error::HedgingError, okex_orders::*};

pub async fn execute(
    okex_orders: OkexOrders,
    okex: OkexClient,
    publisher: Publisher,
) -> Result<(), HedgingError> {
    let PositionSize {
        usd_cents,
        instrument_id,
    } = okex.get_position_in_signed_usd_cents().await?;
    publisher
        .publish(OkexBtcUsdSwapPositionPayload {
            exchange: ExchangeIdRaw::from(OKEX_EXCHANGE_ID),
            instrument_id: InstrumentIdRaw::from(instrument_id.to_string()),
            signed_usd_exposure: SyntheticCentExposure::from(usd_cents),
        })
        .await?;

    for id in okex_orders.open_orders().await? {
        match okex.order_details(id.clone()).await {
            Ok(details) => {
                okex_orders.update_order(details).await?;
            }
            Err(OkexClientError::OrderDoesNotExist) => {
                okex_orders.mark_as_lost(id).await?;
            }
            Err(res) => return Err(res.into()),
        }
    }
    Ok(())
}
