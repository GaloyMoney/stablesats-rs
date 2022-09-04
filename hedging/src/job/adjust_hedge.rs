use okex_client::*;
use sqlxmq::CurrentJob;

use crate::{adjustment_action::*, error::*, synth_usd_liability::*};

pub(super) async fn execute(
    mut current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
) -> Result<(), HedgingError> {
    let target_liability = synth_usd_liability.get_latest_liability().await?;
    let current_position = okex.get_position_in_signed_usd().await?.value;

    match calculate_adjustment(target_liability, current_position) {
        AdjustmentAction::DoNothing => {}
        AdjustmentAction::ClosePosition => {
            okex.close_positions().await?;
        }
        AdjustmentAction::Sell(contracts) => {
            okex.place_order(OkexOrderSide::Sell, contracts).await?;
        }
        AdjustmentAction::Buy(contracts) => {
            okex.place_order(OkexOrderSide::Buy, contracts).await?;
        }
    };
    current_job.complete().await?;
    Ok(())
}
