use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub use okex_client::BtcUsdSwapContracts;

const CONTRACT_SIZE: Decimal = dec!(100);
const MIN_LIABILITY_THRESHOLD: Decimal = dec!(50); // CONTRACT_SIZE / 2

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

pub fn determine_action(abs_liability: Decimal, signed_exposure: Decimal) -> AdjustmentAction {
    if abs_liability >= Decimal::ZERO && abs_liability < MIN_LIABILITY_THRESHOLD {
        AdjustmentAction::ClosePosition
    } else {
        let signed_liability = abs_liability * Decimal::NEGATIVE_ONE;
        let abs_exposure = signed_exposure.abs();
        let exposure_ratio = signed_exposure / signed_liability;
        if exposure_ratio.is_sign_negative() {
            let target_exposure = abs_liability * LOW_SAFEBOUND_RATIO_SHORTING;
            let contracts = ((target_exposure + abs_exposure) / CONTRACT_SIZE)
                .round()
                .abs();
            if contracts.is_zero() {
                AdjustmentAction::DoNothing
            } else {
                AdjustmentAction::Sell(BtcUsdSwapContracts::from(
                    u32::try_from(contracts).expect("decimal to u32"),
                ))
            }
        } else if exposure_ratio < LOW_BOUND_RATIO_SHORTING {
            let target_exposure = abs_liability * LOW_SAFEBOUND_RATIO_SHORTING;
            let contracts = ((target_exposure - abs_exposure) / CONTRACT_SIZE)
                .round()
                .abs();
            if contracts.is_zero() {
                AdjustmentAction::DoNothing
            } else {
                AdjustmentAction::Sell(BtcUsdSwapContracts::from(
                    u32::try_from(contracts).expect("decimal to u32"),
                ))
            }
        } else if exposure_ratio > HIGH_BOUND_RATIO_SHORTING {
            let target_exposure = abs_liability * HIGH_SAFEBOUND_RATIO_SHORTING;
            let contracts = ((abs_exposure - target_exposure) / CONTRACT_SIZE)
                .round()
                .abs();
            if contracts.is_zero() {
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
        let exposure = dec!(-100);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }

    #[test]
    fn close_position() {
        let liability = dec!(0);
        let exposure = dec!(-100);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::ClosePosition);
    }

    #[test]
    fn increase() {
        let liability = dec!(200);
        let exposure = dec!(-100);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Sell(BtcUsdSwapContracts::from(1))
        );
    }

    #[test]
    fn decrease() {
        let liability = dec!(1000);
        let exposure = dec!(-5998.824959074429);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Buy(BtcUsdSwapContracts::from(50))
        );
    }

    #[test]
    fn ignores_rounding() {
        let liability = dec!(100);
        let exposure = dec!(-99.8);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }

    #[test]
    fn positive_exposure() {
        let liability = dec!(100);
        let exposure = dec!(100);
        let adjustment = determine_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Sell(BtcUsdSwapContracts::from(2))
        );
    }

    #[test]
    fn low_bound_limit() {
        let nominal_liability = 10000;
        let liability = Decimal::from(nominal_liability);
        let exposure = Decimal::from(-nominal_liability) * LOW_BOUND_RATIO_SHORTING;

        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }

    #[test]
    fn low_bound_below() {
        let nominal_liability = 10000;
        let liability = Decimal::from(nominal_liability);
        let exposure = Decimal::from(-(nominal_liability - 1)) * LOW_BOUND_RATIO_SHORTING;

        let expected = liability * LOW_SAFEBOUND_RATIO_SHORTING;
        let expected_ct = ((expected - exposure.abs()) / CONTRACT_SIZE).round().abs();

        let adjustment = determine_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Sell(BtcUsdSwapContracts::from(
                u32::try_from(expected_ct).expect("decimal to u32")
            ))
        );
    }

    #[test]
    fn high_bound_limit() {
        let nominal_liability = 10000;
        let liability = Decimal::from(nominal_liability);
        let exposure = Decimal::from(-nominal_liability) * HIGH_BOUND_RATIO_SHORTING;

        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }

    #[test]
    fn high_bound_above() {
        let nominal_liability = 10000;
        let liability = Decimal::from(nominal_liability);
        let exposure = Decimal::from(-(nominal_liability + 1)) * HIGH_BOUND_RATIO_SHORTING;

        let expected = liability * HIGH_SAFEBOUND_RATIO_SHORTING;
        let expected_ct = ((exposure.abs() - expected) / CONTRACT_SIZE).round().abs();

        let adjustment = determine_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Buy(BtcUsdSwapContracts::from(
                u32::try_from(expected_ct).expect("decimal to u32")
            ))
        );
    }

    #[test]
    fn min_liability_threshold_below() {
        let liability = dec!(49);
        let exposure = dec!(-199.98);

        let adjustment = determine_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::ClosePosition);
    }

    #[test]
    fn min_liability_threshold_above() {
        let liability = dec!(55);
        let exposure = dec!(-199.98);
        let expected_ct = 1;

        let adjustment = determine_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Buy(BtcUsdSwapContracts::from(
                u32::try_from(expected_ct).expect("decimal to u32")
            ))
        );
    }
}
