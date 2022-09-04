use rust_decimal::Decimal;

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
            .map(|size| Decimal::ONE_HUNDRED * Decimal::from(u32::from(size)))
    }
}

pub fn determin_action(abs_liability: Decimal, signed_exposure: Decimal) -> AdjustmentAction {
    let target_exposure = abs_liability * Decimal::NEGATIVE_ONE;
    if target_exposure > Decimal::from(MIN_LIABILITY_THRESHOLD) && signed_exposure != Decimal::ZERO
    {
        AdjustmentAction::ClosePosition
    } else if target_exposure > signed_exposure {
        let contracts = ((signed_exposure - target_exposure) / Decimal::from(CONTRACT_SIZE))
            .round()
            .abs();
        if contracts == Decimal::ZERO {
            AdjustmentAction::DoNothing
        } else {
            AdjustmentAction::Buy(BtcUsdSwapContracts::from(
                u32::try_from(contracts).expect("decimal to u32"),
            ))
        }
    } else if target_exposure < signed_exposure {
        let contracts = ((target_exposure - signed_exposure) / Decimal::from(CONTRACT_SIZE))
            .round()
            .abs();
        if contracts == Decimal::ZERO {
            AdjustmentAction::DoNothing
        } else {
            AdjustmentAction::Sell(BtcUsdSwapContracts::from(
                u32::try_from(contracts).expect("decimal to u32"),
            ))
        }
    } else {
        AdjustmentAction::DoNothing
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
        let adjustment = determin_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }

    #[test]
    fn close_position() {
        let liability = dec!(0);
        let exposure = dec!(-100);
        let adjustment = determin_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::ClosePosition);
    }

    #[test]
    fn increase() {
        let liability = dec!(200);
        let exposure = dec!(-100);
        let adjustment = determin_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Sell(BtcUsdSwapContracts::from(1))
        );
    }

    #[test]
    fn decrease() {
        let liability = dec!(1000);
        let exposure = dec!(-5998.824959074429);
        let adjustment = determin_action(liability, exposure);
        assert_eq!(
            adjustment,
            AdjustmentAction::Buy(BtcUsdSwapContracts::from(50))
        );
    }

    #[test]
    fn ignores_rounding() {
        let liability = dec!(100);
        let exposure = dec!(-99.8);
        let adjustment = determin_action(liability, exposure);
        assert_eq!(adjustment, AdjustmentAction::DoNothing);
    }
}
