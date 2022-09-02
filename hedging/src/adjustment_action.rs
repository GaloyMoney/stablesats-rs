use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub use okex_client::BtcUsdSwapContracts;

const CONTRACT_SIZE: u32 = 100;
const MIN_LIABILITY_THRESHOLD: i32 = -95;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AdjustmentAction {
    DoNothing,
    ClosePosition,
    Sell(BtcUsdSwapContracts),
    Buy(BtcUsdSwapContracts),
}

pub fn calculate_adjustment(abs_liability: Decimal, signed_exposure: Decimal) -> AdjustmentAction {
    let target_exposure = abs_liability * Decimal::NEGATIVE_ONE;
    if target_exposure > Decimal::from(MIN_LIABILITY_THRESHOLD) && signed_exposure != dec!(0) {
        AdjustmentAction::ClosePosition
    } else if target_exposure > signed_exposure {
        let contracts = (signed_exposure - target_exposure) / Decimal::from(CONTRACT_SIZE);
        AdjustmentAction::Buy(BtcUsdSwapContracts::from(u32::try_from(contracts.abs()).expect("decimal to u32")))
    } else if target_exposure < signed_exposure {
        let contracts = (target_exposure - signed_exposure) / Decimal::from(CONTRACT_SIZE);
        AdjustmentAction::Sell(BtcUsdSwapContracts::from(u32::try_from(contracts.abs()).expect("decimal to u32")))
    } else {
        AdjustmentAction::DoNothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_adjustment() {
        let liability = Decimal::new(100, 0);
        let exposure = Decimal::new(-100, 0);
        let adjustment = calculate_adjustment(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }

    #[test]
    fn close_position() {
        let liability = Decimal::new(0, 0);
        let exposure = Decimal::new(-100, 0);
        let adjustment = calculate_adjustment(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::ClosePosition);
    }

    #[test]
    fn increase() {
        let liability = Decimal::new(200, 0);
        let exposure = Decimal::new(-100, 0);
        let adjustment = calculate_adjustment(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::Sell(BtcUsdSwapContracts::from(1)));
    }

    #[test]
    fn decrease() {
        let liability = Decimal::new(100, 0);
        let exposure = Decimal::new(-200, 0);
        let adjustment = calculate_adjustment(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::Buy(BtcUsdSwapContracts::from(1)));
    }
}
