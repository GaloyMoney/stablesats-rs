use rust_decimal::Decimal;
use tracing::instrument;

use okex_client::*;
use shared::pubsub::CorrelationId;

use crate::{error::*, okex::*};

#[instrument(name = "hedging.job.adjust_hedge", skip_all, fields(correlation_id = %correlation_id,
        target_liability, current_position, action, placed_order, client_order_id, lag_ok), err)]
pub(super) async fn execute(
    correlation_id: CorrelationId,
    pool: &sqlx::PgPool,
    ledger: ledger::Ledger,
    okex: OkexClient,
    okex_orders: OkexOrders,
    hedging_adjustment: HedgingAdjustment,
) -> Result<(), HedgingError> {
    let span = tracing::Span::current();
    if !crate::hack_user_trades_lag::lag_ok(pool).await? {
        span.record("lag_ok", &tracing::field::display(false));
        return Ok(());
    }
    let target_liability = ledger.balances().target_liability_in_cents().await?;
    span.record(
        "target_liability",
        &tracing::field::display(target_liability),
    );
    let current_position = okex.get_position_in_signed_usd_cents().await?.usd_cents;
    span.record(
        "current_position",
        &tracing::field::display(current_position),
    );

    let action = hedging_adjustment.determine_action(target_liability, current_position.into());
    span.record("action", &tracing::field::display(&action));
    match action {
        OkexHedgeAdjustment::DoNothing => {}
        _ => {
            let reservation = OrderReservation {
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
                    OkexHedgeAdjustment::ClosePosition => {
                        okex.close_positions(order_id).await?;
                    }
                    OkexHedgeAdjustment::Sell(ref contracts) => {
                        okex.place_order(order_id, OkexOrderSide::Sell, contracts)
                            .await?;
                    }
                    OkexHedgeAdjustment::Buy(ref contracts) => {
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
