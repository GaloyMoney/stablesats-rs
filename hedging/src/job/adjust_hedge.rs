use rust_decimal::Decimal;
use sqlxmq::CurrentJob;
use tracing::instrument;

use okex_client::*;
use shared::pubsub::CorrelationId;

use crate::{adjustment_action::*, error::*, hedging_adjustments::*, synth_usd_liability::*};

#[instrument(name = "adjust_hedge", skip_all, fields(
        target_liability, current_position, action, usd_value_after_adjustment) err)]
pub(super) async fn execute(
    mut current_job: CurrentJob,
    correlation_id: CorrelationId,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    hedging_adjustments: HedgingAdjustments,
) -> Result<(), HedgingError> {
    let span = tracing::Span::current();
    let target_liability = synth_usd_liability.get_latest_liability().await?;
    span.record(
        "target_liability",
        &tracing::field::display(target_liability),
    );
    let current_position = okex.get_position_in_signed_usd().await?.value;
    span.record(
        "current_position",
        &tracing::field::display(current_position),
    );

    let action = determine_action(target_liability, current_position);
    span.record("action", &tracing::field::display(&action));
    let mut exchange_ref = None;
    match action {
        AdjustmentAction::DoNothing => {}
        AdjustmentAction::ClosePosition => {
            okex.close_positions().await?;
        }
        AdjustmentAction::Sell(ref contracts) => {
            exchange_ref = Some(
                okex.place_order(OkexOrderSide::Sell, contracts)
                    .await?
                    .value,
            );
        }
        AdjustmentAction::Buy(ref contracts) => {
            exchange_ref = Some(okex.place_order(OkexOrderSide::Buy, contracts).await?.value);
        }
    };
    if action.action_required() {
        let usd_value_after_adjustment = okex.get_position_in_signed_usd().await?.value;
        span.record(
            "usd_value_after_adjustment",
            &tracing::field::display(usd_value_after_adjustment),
        );
        let _ = hedging_adjustments
            .persist(Adjustment {
                correlation_id,
                exchange_ref,
                action,
                target_usd_value: target_liability * Decimal::NEGATIVE_ONE,
                usd_value_before_adjustment: current_position,
                usd_value_after_adjustment,
            })
            .await;
    }
    current_job.complete().await?;
    Ok(())
}
