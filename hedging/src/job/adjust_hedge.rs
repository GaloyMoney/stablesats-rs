use rust_decimal::Decimal;
use tracing::instrument;

use okex_client::*;
use shared::pubsub::CorrelationId;

use crate::{adjustment_action::*, error::*, okex_orders::*, synth_usd_liability::*};

use super::HedgingFundingConfig;

#[instrument(name = "adjust_hedge", skip_all, fields(correlation_id = %correlation_id,
        target_liability, current_position, action, placed_order, client_order_id) err)]
pub(super) async fn execute(
    correlation_id: CorrelationId,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    okex_orders: OkexOrders,
    hedging_funding_config: HedgingFundingConfig,
) -> Result<(), HedgingError> {
    let span = tracing::Span::current();
    let target_liability = synth_usd_liability.get_latest_liability().await?;
    span.record(
        "target_liability",
        &tracing::field::display(target_liability),
    );
    let current_position = okex.get_position_in_signed_usd_cents().await?.usd_cents;
    span.record(
        "current_position",
        &tracing::field::display(current_position),
    );

    let action = determine_action(
        target_liability,
        current_position.into(),
        hedging_funding_config,
    );
    span.record("action", &tracing::field::display(&action));
    match action {
        AdjustmentAction::DoNothing => {}
        _ => {
            let reservation = Reservation {
                correlation_id,
                action: &action,
                target_usd_value: target_liability * Decimal::NEGATIVE_ONE,
                usd_value_before_order: current_position,
            };
            if let Some(order_id) = okex_orders.reserve_order_slot(reservation).await? {
                span.record(
                    "client_order_id",
                    &tracing::field::display(String::from(order_id.clone())),
                );
                match action {
                    AdjustmentAction::ClosePosition => {
                        okex.close_positions(order_id).await?;
                    }
                    AdjustmentAction::Sell(ref contracts) => {
                        okex.place_order(order_id, OkexOrderSide::Sell, contracts)
                            .await?;
                    }
                    AdjustmentAction::Buy(ref contracts) => {
                        okex.place_order(order_id, OkexOrderSide::Buy, contracts)
                            .await?;
                    }
                    _ => unreachable!(),
                }
                span.record("placed_order", &tracing::field::display(true));
            } else {
                span.record("placed_order", &tracing::field::display(false));
            }
        }
    };
    Ok(())
}
