use rust_decimal::Decimal;
use sqlxmq::CurrentJob;

use okex_client::*;
use shared::pubsub::CorrelationId;

use crate::{adjustment_action::*, error::*, hedging_adjustments::*, synth_usd_liability::*};

pub(super) async fn execute(
    mut current_job: CurrentJob,
    correlation_id: CorrelationId,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
    hedging_adjustments: HedgingAdjustments,
) -> Result<(), HedgingError> {
    let target_liability = synth_usd_liability.get_latest_liability().await?;
    let current_position = okex.get_position_in_signed_usd().await?.value;

    let action = determine_action(target_liability, current_position);
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
