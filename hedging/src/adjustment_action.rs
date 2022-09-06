use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub use okex_client::BtcUsdSwapContracts;

const CONTRACT_SIZE: Decimal = dec!(100);
const MIN_LIABILITY_THRESHOLD: Decimal = dec!(100); // CONTRACT_SIZE / 2

const LOW_BOUND_RATIO_SHORTING: Decimal = dec!(0.95);
const LOW_SAFEBOUND_RATIO_SHORTING: Decimal = dec!(0.98);
const HIGH_SAFEBOUND_RATIO_SHORTING: Decimal = dec!(1.);
const HIGH_BOUND_RATIO_SHORTING: Decimal = dec!(1.03);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AdjustmentAction {
    DoNothing,
    ClosePosition,
    Sell(BtcUsdSwapContracts),
    Buy(BtcUsdSwapContracts),
}
impl std::fmt::Display for AdjustmentAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdjustmentAction::DoNothing => write!(f, "DoNothing"),
            AdjustmentAction::ClosePosition => write!(f, "ClosePosition"),
            AdjustmentAction::Sell(contracts) => write!(f, "Sell({})", contracts),
            AdjustmentAction::Buy(contracts) => write!(f, "Buy({})", contracts),
        }
    }
}
impl AdjustmentAction {
    pub fn action_required(&self) -> bool {
        !matches!(*self, Self::DoNothing)
    }

    pub fn action_type(&self) -> &'static str {
        match *self {
            Self::DoNothing => "do-nothing",
            Self::ClosePosition => "close-position",
            Self::Sell(_) => "sell",
            Self::Buy(_) => "buy",
        }
    }

    pub fn size(&self) -> Option<u32> {
        match *self {
            Self::Sell(ref size) | Self::Buy(ref size) => Some(size.into()),
            _ => None,
        }
    }

    pub fn unit(&self) -> &'static str {
        "swap-contract"
    }

    pub fn size_in_usd(&self) -> Option<Decimal> {
        self.size()
            .map(|size| Decimal::ONE_HUNDRED * Decimal::from(size))
    }
}

pub fn determine_action(abs_liability: Decimal, abs_exposure: Decimal) -> AdjustmentAction {
    if abs_liability >= Decimal::ZERO && abs_liability < MIN_LIABILITY_THRESHOLD {
        AdjustmentAction::ClosePosition
    } else {
        let exposure_ratio = abs_exposure / abs_liability;
        if exposure_ratio < LOW_BOUND_RATIO_SHORTING {
            let target_exposure = abs_liability * LOW_SAFEBOUND_RATIO_SHORTING;
            let contracts = ((target_exposure - abs_exposure) / CONTRACT_SIZE).round();
            if contracts == Decimal::ZERO {
                AdjustmentAction::DoNothing
            } else {
                AdjustmentAction::Sell(BtcUsdSwapContracts::from(
                    u32::try_from(contracts).expect("decimal to u32"),
                ))
            }
        } else if exposure_ratio > HIGH_BOUND_RATIO_SHORTING {
            let target_exposure = abs_liability * HIGH_SAFEBOUND_RATIO_SHORTING;
            let contracts = ((abs_exposure - target_exposure) / CONTRACT_SIZE).round();
            if contracts == Decimal::ZERO {
                AdjustmentAction::DoNothing
            } else {
                AdjustmentAction::Buy(BtcUsdSwapContracts::from(
                    u32::try_from(contracts).expect("decimal to u32"),
                ))
            }
        } else {
            AdjustmentAction::DoNothing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn no_adjustment() {
        let liability = dec!(100);
        let exposure = dec!(100);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }

    #[test]
    fn close_position() {
        let liability = dec!(0);
        let exposure = dec!(100);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::ClosePosition);
    }

    #[test]
    fn increase() {
        let liability = dec!(200);
        let exposure = dec!(100);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Sell(BtcUsdSwapContracts::from(1))
        );
    }

    #[test]
    fn decrease() {
        let liability = dec!(1000);
        let exposure = dec!(5998.824959074429);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Buy(BtcUsdSwapContracts::from(50))
        );
    }

    #[test]
    fn ignores_rounding() {
        let liability = dec!(100);
        let exposure = dec!(99.8);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }
}
