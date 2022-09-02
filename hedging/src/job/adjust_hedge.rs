use okex_client::*;

use crate::{adjustment_action::*, error::*, synth_usd_liability::*};

pub(super) async fn execute(
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
) -> Result<(), HedgingError> {
    let target_liability = synth_usd_liability.get_latest_liability().await?;
    let current_position = okex.get_position_in_usd().await?.value;

    match calculate_adjustment(target_liability, current_position) {
        AdjustmentAction::DoNothing => {}
        AdjustmentAction::ClosePosition => {
            let _ = okex.close_positions().await?;
        }
        AdjustmentAction::Sell(contracts) => {
            let _ = okex.place_order(OkexOrderSide::Sell, contracts).await?;
        }
        AdjustmentAction::Buy(contracts) => {
            let _ = okex.place_order(OkexOrderSide::Buy, contracts).await?;
        }
    };
    Ok(())
}
