use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub use okex_client::BtcUsdSwapContracts;
pub use shared::payload::{SyntheticCentExposure, SyntheticCentLiability};

const CONTRACT_SIZE_CENTS: Decimal = dec!(10_000);
const MIN_LIABILITY_THRESHOLD_CENTS: Decimal = dec!(5_000);
const MINIMUM_TRANSFER_AMOUNT_CENTS: Decimal = CONTRACT_SIZE_CENTS;

const LOW_BOUND_RATIO_LEVERAGE: Decimal = dec!(2);
const LOW_SAFEBOUND_RATIO_LEVERAGE: Decimal = dec!(3);
const HIGH_SAFEBOUND_RATIO_LEVERAGE: Decimal = dec!(3);
const HIGH_BOUND_RATIO_LEVERAGE: Decimal = dec!(4);

const HIGH_BOUND_BUFFER: Decimal = dec!(0.9);

const SATS_PER_BTC: Decimal = dec!(100_000_000);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RebalanceAction {
    DoNothing,
    WithdrawAll(Decimal),
    Deposit(Decimal),
    Withdraw(Decimal),
}
impl std::fmt::Display for RebalanceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RebalanceAction::DoNothing => write!(f, "DoNothing"),
            RebalanceAction::WithdrawAll(amount_in_btc) => {
                write!(f, "WithdrawAll({})", amount_in_btc)
            }
            RebalanceAction::Deposit(amount_in_btc) => {
                write!(f, "Deposit({})", amount_in_btc)
            }
            RebalanceAction::Withdraw(amount_in_btc) => {
                write!(f, "Withdraw({})", amount_in_btc)
            }
        }
    }
}
impl RebalanceAction {
    pub fn action_required(&self) -> bool {
        !matches!(*self, Self::DoNothing)
    }

    pub fn action_type(&self) -> &'static str {
        match *self {
            Self::DoNothing => "do-nothing",
            Self::WithdrawAll(_) => "withdraw-all",
            Self::Deposit(_) => "deposit",
            Self::Withdraw(_) => "withdraw",
        }
    }

    pub fn size(&self) -> Option<Decimal> {
        match *self {
            Self::WithdrawAll(size) | Self::Deposit(size) | Self::Withdraw(size) => Some(size),
            _ => None,
        }
    }

    pub fn unit(&self) -> &'static str {
        "btc"
    }
}

fn round_btc(amount_in_btc: Decimal) -> Decimal {
    let amount_in_sats = amount_in_btc * SATS_PER_BTC;
    amount_in_sats.round() / SATS_PER_BTC
}

fn floor_btc(amount_in_btc: Decimal) -> Decimal {
    let amount_in_sats = amount_in_btc * SATS_PER_BTC;
    amount_in_sats.floor() / SATS_PER_BTC
}

pub fn determine_action(
    abs_liability_in_cents: SyntheticCentLiability,
    signed_exposure_in_cents: SyntheticCentExposure,
    total_collateral_in_btc: Decimal,
    btc_price_in_cents: Decimal,
) -> RebalanceAction {
    let abs_liability_in_btc = abs_liability_in_cents / btc_price_in_cents;
    let abs_exposure_in_btc = Decimal::from(signed_exposure_in_cents).abs() / btc_price_in_cents;
    if abs_exposure_in_btc.is_zero()
        && total_collateral_in_btc.is_zero()
        && abs_liability_in_cents > MIN_LIABILITY_THRESHOLD_CENTS
    {
        let new_collateral_in_btc = abs_liability_in_btc / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

        RebalanceAction::Deposit(transfer_size_in_btc)
    } else if abs_exposure_in_btc.is_zero()
        && total_collateral_in_btc > Decimal::ZERO
        && abs_liability_in_cents >= Decimal::ZERO
        && abs_liability_in_cents < MINIMUM_TRANSFER_AMOUNT_CENTS
    {
        RebalanceAction::WithdrawAll(floor_btc(total_collateral_in_btc))
    } else if abs_liability_in_btc > total_collateral_in_btc * HIGH_BOUND_RATIO_LEVERAGE {
        let new_collateral_in_btc = abs_liability_in_btc / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

        RebalanceAction::Deposit(transfer_size_in_btc)
    } else if abs_exposure_in_btc < total_collateral_in_btc * LOW_BOUND_RATIO_LEVERAGE {
        let new_collateral_in_btc = abs_exposure_in_btc / LOW_SAFEBOUND_RATIO_LEVERAGE;
        let transfer_size_in_btc = floor_btc(total_collateral_in_btc - new_collateral_in_btc);

        RebalanceAction::Withdraw(transfer_size_in_btc)
    } else if abs_exposure_in_btc
        > total_collateral_in_btc * HIGH_BOUND_BUFFER * HIGH_BOUND_RATIO_LEVERAGE
    {
        let new_collateral_in_btc = abs_exposure_in_btc / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

        RebalanceAction::Deposit(transfer_size_in_btc)
    } else {
        RebalanceAction::DoNothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn do_nothing_conditions() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(-10_000));
        let total_collateral: Decimal = liability / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let btc_price: Decimal = dec!(1);
        let adjustment = determine_action(liability, exposure, total_collateral, btc_price);
        assert_eq!(adjustment, RebalanceAction::DoNothing);
    }

    #[test]
    fn initial_conditions() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(0));
        let total_collateral: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_transfer: Decimal = round_btc(liability / HIGH_SAFEBOUND_RATIO_LEVERAGE);
        let adjustment = determine_action(liability, exposure, total_collateral, btc_price);
        assert_eq!(adjustment, RebalanceAction::Deposit(expected_transfer));
    }

    #[test]
    fn terminal_conditions() {
        let liability =
            SyntheticCentLiability::try_from(MINIMUM_TRANSFER_AMOUNT_CENTS / dec!(2)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(0));
        let total_collateral: Decimal = liability / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let btc_price: Decimal = dec!(1);
        let expected_transfer: Decimal = floor_btc(total_collateral);
        let adjustment = determine_action(liability, exposure, total_collateral, btc_price);
        assert_eq!(adjustment, RebalanceAction::WithdrawAll(expected_transfer));
    }

    #[test]
    fn user_activity_tracking() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(-3_000));
        let total_collateral: Decimal = exposure / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let btc_price: Decimal = dec!(1);
        let expected_transfer: Decimal =
            round_btc(liability / HIGH_SAFEBOUND_RATIO_LEVERAGE - total_collateral);
        let adjustment = determine_action(liability, exposure, total_collateral, btc_price);
        assert_eq!(adjustment, RebalanceAction::Deposit(expected_transfer));
    }

    #[test]
    fn counterparty_risk_avoidance() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = dec!(10_000);
        let signed_exposure = SyntheticCentExposure::from(-exposure);
        let total_collateral: Decimal = dec!(10_000);
        let btc_price: Decimal = dec!(1);
        let expected_transfer: Decimal =
            floor_btc(total_collateral - exposure / LOW_SAFEBOUND_RATIO_LEVERAGE);
        let adjustment = determine_action(liability, signed_exposure, total_collateral, btc_price);
        assert_eq!(adjustment, RebalanceAction::Withdraw(expected_transfer));
    }

    #[test]
    fn liquidation_risk_avoidance() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = dec!(10_100);
        let signed_exposure = SyntheticCentExposure::from(-exposure);
        let total_collateral: Decimal = exposure / HIGH_BOUND_RATIO_LEVERAGE;
        let btc_price: Decimal = dec!(1);
        let expected_transfer: Decimal =
            round_btc(exposure / HIGH_SAFEBOUND_RATIO_LEVERAGE - total_collateral);
        let adjustment = determine_action(liability, signed_exposure, total_collateral, btc_price);
        assert_eq!(adjustment, RebalanceAction::Deposit(expected_transfer));
    }

    #[test]
    fn btc_round_down() {
        let expected_btc = dec!(100_000_000.0) / dec!(100_000_000);
        let unrounded_btc = dec!(100_000_000.4) / dec!(100_000_000);
        let rounded_btc = round_btc(unrounded_btc);
        assert_eq!(rounded_btc, expected_btc);
    }

    #[test]
    fn btc_round_up() {
        let expected_btc = dec!(100_000_001.0) / dec!(100_000_000);
        let unrounded_btc = dec!(100_000_000.6) / dec!(100_000_000);
        let rounded_btc = round_btc(unrounded_btc);
        assert_eq!(rounded_btc, expected_btc);
    }

    #[test]
    fn btc_floor_down() {
        let expected_btc = dec!(100_000_000.0) / dec!(100_000_000);
        let unfloored_btc = dec!(100_000_000.4) / dec!(100_000_000);
        let floored_btc = floor_btc(unfloored_btc);
        assert_eq!(floored_btc, expected_btc);
    }

    #[test]
    fn btc_floor_up() {
        let expected_btc = dec!(100_000_000.0) / dec!(100_000_000);
        let unfloored_btc = dec!(100_000_000.6) / dec!(100_000_000);
        let floored_btc = floor_btc(unfloored_btc);
        assert_eq!(floored_btc, expected_btc);
    }
}
