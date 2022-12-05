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
const MINIMUM_FUNDING_BALANCE_BTC: Decimal = dec!(0.5);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RebalanceAction {
    DoNothing,
    Deposit(Decimal, Decimal, Decimal),
    Withdraw(Decimal, Decimal, Decimal),
}
impl std::fmt::Display for RebalanceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RebalanceAction::DoNothing => write!(f, "DoNothing"),
            RebalanceAction::Deposit(
                amount_in_btc,
                internal_amount_in_btc,
                external_amount_in_btc,
            ) => {
                write!(
                    f,
                    "Deposit(total: {}, internal:{}, external:{})",
                    amount_in_btc, internal_amount_in_btc, external_amount_in_btc
                )
            }
            RebalanceAction::Withdraw(
                amount_in_btc,
                internal_amount_in_btc,
                external_amount_in_btc,
            ) => {
                write!(
                    f,
                    "Withdraw(total: {}, internal:{}, external:{})",
                    amount_in_btc, internal_amount_in_btc, external_amount_in_btc
                )
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
            Self::Deposit(_, _, _) => "deposit",
            Self::Withdraw(_, _, _) => "withdraw",
        }
    }

    pub fn size(&self) -> Option<Decimal> {
        match *self {
            Self::Deposit(size, _, _) | Self::Withdraw(size, _, _) => Some(size),
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
    funding_btc_total_balance: Decimal,
) -> RebalanceAction {
    let abs_liability_in_btc = abs_liability_in_cents / btc_price_in_cents;
    let abs_exposure_in_btc = Decimal::from(signed_exposure_in_cents).abs() / btc_price_in_cents;
    if abs_exposure_in_btc.is_zero()
        && total_collateral_in_btc.is_zero()
        && abs_liability_in_cents > MIN_LIABILITY_THRESHOLD_CENTS
    {
        let new_collateral_in_btc = abs_liability_in_btc / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

        let (internal_transfer_amount, external_transfer_amount) =
            split_deposit(funding_btc_total_balance, transfer_size_in_btc);

        RebalanceAction::Deposit(
            transfer_size_in_btc,
            internal_transfer_amount,
            external_transfer_amount,
        )
    } else if abs_exposure_in_btc.is_zero()
        && total_collateral_in_btc > Decimal::ZERO
        && abs_liability_in_cents >= Decimal::ZERO
        && abs_liability_in_cents < MINIMUM_TRANSFER_AMOUNT_CENTS
    {
        let transfer_size_in_btc = floor_btc(total_collateral_in_btc);

        let (internal_transfer_amount, external_transfer_amount) =
            split_withdraw(funding_btc_total_balance, transfer_size_in_btc);

        RebalanceAction::Withdraw(
            transfer_size_in_btc,
            internal_transfer_amount,
            external_transfer_amount,
        )
    } else if abs_liability_in_btc > total_collateral_in_btc * HIGH_BOUND_RATIO_LEVERAGE {
        let new_collateral_in_btc = abs_liability_in_btc / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

        let (internal_transfer_amount, external_transfer_amount) =
            split_deposit(funding_btc_total_balance, transfer_size_in_btc);

        RebalanceAction::Deposit(
            transfer_size_in_btc,
            internal_transfer_amount,
            external_transfer_amount,
        )
    } else if abs_exposure_in_btc < total_collateral_in_btc * LOW_BOUND_RATIO_LEVERAGE {
        let new_collateral_in_btc = abs_exposure_in_btc / LOW_SAFEBOUND_RATIO_LEVERAGE;
        let transfer_size_in_btc = floor_btc(total_collateral_in_btc - new_collateral_in_btc);

        let (internal_transfer_amount, external_transfer_amount) =
            split_withdraw(funding_btc_total_balance, transfer_size_in_btc);

        RebalanceAction::Withdraw(
            transfer_size_in_btc,
            internal_transfer_amount,
            external_transfer_amount,
        )
    } else if abs_exposure_in_btc
        > total_collateral_in_btc * HIGH_BOUND_BUFFER * HIGH_BOUND_RATIO_LEVERAGE
    {
        let new_collateral_in_btc = abs_exposure_in_btc / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

        let (internal_transfer_amount, external_transfer_amount) =
            split_deposit(funding_btc_total_balance, transfer_size_in_btc);

        RebalanceAction::Deposit(
            transfer_size_in_btc,
            internal_transfer_amount,
            external_transfer_amount,
        )
    } else {
        RebalanceAction::DoNothing
    }
}

fn split_deposit(funding_btc_total_balance: Decimal, amount_in_btc: Decimal) -> (Decimal, Decimal) {
    let internal_transfer_amount = std::cmp::min(funding_btc_total_balance, amount_in_btc);
    let new_funding_balance = funding_btc_total_balance - internal_transfer_amount;
    let funding_refill = std::cmp::max(
        Decimal::ZERO,
        MINIMUM_FUNDING_BALANCE_BTC - new_funding_balance,
    );
    let missing_amount = amount_in_btc - internal_transfer_amount;
    let external_transfer_amount = missing_amount + funding_refill;

    (internal_transfer_amount, external_transfer_amount)
}

fn split_withdraw(
    funding_btc_total_balance: Decimal,
    amount_in_btc: Decimal,
) -> (Decimal, Decimal) {
    let internal_transfer_amount = amount_in_btc;
    let external_transfer_amount = std::cmp::max(
        Decimal::ZERO,
        amount_in_btc + funding_btc_total_balance - MINIMUM_FUNDING_BALANCE_BTC,
    );

    (internal_transfer_amount, external_transfer_amount)
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
        let adjustment = determine_action(
            liability,
            exposure,
            total_collateral,
            btc_price,
            MINIMUM_FUNDING_BALANCE_BTC,
        );
        assert_eq!(adjustment, RebalanceAction::DoNothing);
    }

    #[test]
    fn initial_conditions() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(0));
        let total_collateral: Decimal = dec!(0);
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal = round_btc(liability / HIGH_SAFEBOUND_RATIO_LEVERAGE);
        let (expected_internal, expected_external) =
            split_deposit(funding_btc_total_balance, expected_total);
        let adjustment = determine_action(
            liability,
            exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::Deposit(expected_total, expected_internal, expected_external)
        );
    }

    #[test]
    fn terminal_conditions() {
        let liability =
            SyntheticCentLiability::try_from(MINIMUM_TRANSFER_AMOUNT_CENTS / dec!(2)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(0));
        let total_collateral: Decimal = liability / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal = floor_btc(total_collateral);
        let (expected_internal, expected_external) =
            split_withdraw(funding_btc_total_balance, expected_total);
        let adjustment = determine_action(
            liability,
            exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::Withdraw(expected_total, expected_internal, expected_external)
        );
    }

    #[test]
    fn user_activity_tracking() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(-3_000));
        let total_collateral: Decimal = exposure / HIGH_SAFEBOUND_RATIO_LEVERAGE;
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal =
            round_btc(liability / HIGH_SAFEBOUND_RATIO_LEVERAGE - total_collateral);
        let (expected_internal, expected_external) =
            split_deposit(funding_btc_total_balance, expected_total);
        let adjustment = determine_action(
            liability,
            exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::Deposit(expected_total, expected_internal, expected_external)
        );
    }

    #[test]
    fn counterparty_risk_avoidance() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = dec!(10_000);
        let signed_exposure = SyntheticCentExposure::from(-exposure);
        let total_collateral: Decimal = dec!(10_000);
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal =
            floor_btc(total_collateral - exposure / LOW_SAFEBOUND_RATIO_LEVERAGE);
        let (expected_internal, expected_external) =
            split_withdraw(funding_btc_total_balance, expected_total);
        let adjustment = determine_action(
            liability,
            signed_exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::Withdraw(expected_total, expected_internal, expected_external)
        );
    }

    #[test]
    fn liquidation_risk_avoidance() {
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = dec!(10_100);
        let signed_exposure = SyntheticCentExposure::from(-exposure);
        let total_collateral: Decimal = exposure / HIGH_BOUND_RATIO_LEVERAGE;
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal =
            round_btc(exposure / HIGH_SAFEBOUND_RATIO_LEVERAGE - total_collateral);
        let (expected_internal, expected_external) =
            split_deposit(funding_btc_total_balance, expected_total);
        let adjustment = determine_action(
            liability,
            signed_exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::Deposit(expected_total, expected_internal, expected_external)
        );
    }

    #[test]
    fn split_deposit_no_funding() {
        let funding_btc_total_balance: Decimal = dec!(0);
        let amount_in_btc: Decimal = dec!(1);
        let expected_internal = dec!(0);
        let expected_external = amount_in_btc + MINIMUM_FUNDING_BALANCE_BTC;
        let (internal, external) = split_deposit(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_equal_funding_amount_under() {
        let funding_btc_total_balance: Decimal = MINIMUM_FUNDING_BALANCE_BTC;
        let amount_in_btc: Decimal = funding_btc_total_balance / dec!(5) * dec!(3);
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal;
        let (internal, external) = split_deposit(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_equal_funding_amount_equal() {
        let funding_btc_total_balance: Decimal = MINIMUM_FUNDING_BALANCE_BTC;
        let amount_in_btc: Decimal = funding_btc_total_balance;
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal;
        let (internal, external) = split_deposit(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_equal_funding_amount_over() {
        let funding_btc_total_balance: Decimal = MINIMUM_FUNDING_BALANCE_BTC;
        let amount_in_btc: Decimal = funding_btc_total_balance * dec!(2);
        let expected_internal = funding_btc_total_balance;
        let expected_external = amount_in_btc;
        let (internal, external) = split_deposit(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_more_funding_amount_under() {
        let extra_funding = dec!(0.3);
        let funding_btc_total_balance: Decimal = MINIMUM_FUNDING_BALANCE_BTC + extra_funding;
        let amount_in_btc: Decimal = MINIMUM_FUNDING_BALANCE_BTC;
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal - extra_funding;
        let (internal, external) = split_deposit(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_more_funding_amount_equal() {
        let extra_funding = dec!(0.3);
        let funding_btc_total_balance: Decimal = MINIMUM_FUNDING_BALANCE_BTC + extra_funding;
        let amount_in_btc: Decimal = funding_btc_total_balance;
        let expected_internal = amount_in_btc;
        let expected_external = MINIMUM_FUNDING_BALANCE_BTC;
        let (internal, external) = split_deposit(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_more_funding_amount_over() {
        let extra_funding = dec!(0.3);
        let funding_btc_total_balance: Decimal = MINIMUM_FUNDING_BALANCE_BTC + extra_funding;
        let amount_in_btc: Decimal = funding_btc_total_balance * dec!(2);
        let expected_internal = funding_btc_total_balance;
        let expected_external = amount_in_btc - extra_funding;
        let (internal, external) = split_deposit(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_no_funding_amount_under() {
        let funding_btc_total_balance: Decimal = dec!(0);
        let amount_in_btc: Decimal = MINIMUM_FUNDING_BALANCE_BTC / dec!(5) * dec!(3);
        let expected_internal = amount_in_btc;
        let expected_external = dec!(0);
        let (internal, external) = split_withdraw(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_no_funding_amount_equal() {
        let funding_btc_total_balance: Decimal = dec!(0);
        let amount_in_btc: Decimal = MINIMUM_FUNDING_BALANCE_BTC;
        let expected_internal = amount_in_btc;
        let expected_external = dec!(0);
        let (internal, external) = split_withdraw(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_no_funding_amount_over() {
        let funding_btc_total_balance: Decimal = dec!(0);
        let amount_in_btc: Decimal = MINIMUM_FUNDING_BALANCE_BTC * dec!(2);
        let expected_internal = amount_in_btc;
        let expected_external = amount_in_btc - MINIMUM_FUNDING_BALANCE_BTC;
        let (internal, external) = split_withdraw(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_equal_funding() {
        let funding_btc_total_balance: Decimal = MINIMUM_FUNDING_BALANCE_BTC;
        let amount_in_btc: Decimal = dec!(1);
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal;
        let (internal, external) = split_withdraw(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_more_funding() {
        let extra_funding = dec!(0.3);
        let funding_btc_total_balance: Decimal = MINIMUM_FUNDING_BALANCE_BTC + extra_funding;
        let amount_in_btc: Decimal = MINIMUM_FUNDING_BALANCE_BTC;
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal + extra_funding;
        let (internal, external) = split_withdraw(funding_btc_total_balance, amount_in_btc);

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
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
