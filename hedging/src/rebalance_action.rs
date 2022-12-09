use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub use okex_client::BtcUsdSwapContracts;
pub use shared::payload::{SyntheticCentExposure, SyntheticCentLiability};

use crate::{FundingSectionConfig, HedgingSectionConfig};

const SATS_PER_BTC: Decimal = dec!(100_000_000);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RebalanceAction {
    DoNothing,
    TransferTradingToFunding(Decimal),
    TransferFundingToTrading(Decimal),
    OnchainDeposit(Decimal),
    OnchainWithdraw(Decimal),
}
impl std::fmt::Display for RebalanceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RebalanceAction::DoNothing => write!(f, "DoNothing"),
            RebalanceAction::TransferTradingToFunding(amount_in_btc) => {
                write!(f, "TransferTradingToFunding({})", amount_in_btc)
            }
            RebalanceAction::TransferFundingToTrading(amount_in_btc) => {
                write!(f, "TransferFundingToTrading({})", amount_in_btc)
            }
            RebalanceAction::OnchainDeposit(amount_in_btc) => {
                write!(f, "OnchainDeposit({})", amount_in_btc)
            }
            RebalanceAction::OnchainWithdraw(amount_in_btc) => {
                write!(f, "OnchainWithdraw({})", amount_in_btc)
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
            Self::TransferTradingToFunding(_) => "transfer-trading-to-funding",
            Self::TransferFundingToTrading(_) => "transfer-funding-to-trading",
            Self::OnchainDeposit(_) => "deposit",
            Self::OnchainWithdraw(_) => "withdraw",
        }
    }

    pub fn size(&self) -> Option<Decimal> {
        match *self {
            Self::TransferTradingToFunding(size)
            | Self::TransferFundingToTrading(size)
            | Self::OnchainDeposit(size)
            | Self::OnchainWithdraw(size) => Some(size),
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

#[derive(Debug, Clone)]
pub struct FundingAdjustment {
    config: FundingSectionConfig,
    hedging_config: HedgingSectionConfig,
}

impl FundingAdjustment {
    pub fn new(config: FundingSectionConfig, hedging_config: HedgingSectionConfig) -> Self {
        Self {
            config,
            hedging_config,
        }
    }

    pub fn determine_action(
        &self,
        abs_liability_in_cents: SyntheticCentLiability,
        signed_exposure_in_cents: SyntheticCentExposure,
        total_collateral_in_btc: Decimal,
        btc_price_in_cents: Decimal,
        funding_btc_total_balance: Decimal,
    ) -> RebalanceAction {
        let abs_liability_in_btc = abs_liability_in_cents / btc_price_in_cents;
        let abs_exposure_in_btc =
            Decimal::from(signed_exposure_in_cents).abs() / btc_price_in_cents;
        if abs_exposure_in_btc.is_zero()
            && total_collateral_in_btc.is_zero()
            && abs_liability_in_cents > self.hedging_config.minimum_liability_threshold_cents
        {
            let new_collateral_in_btc =
                abs_liability_in_btc / self.config.high_safebound_ratio_leverage;
            let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

            let (internal_transfer_amount, external_transfer_amount) = split_deposit(
                funding_btc_total_balance,
                transfer_size_in_btc,
                self.config.minimum_funding_balance_btc,
            );

            if !internal_transfer_amount.is_zero() {
                RebalanceAction::TransferFundingToTrading(internal_transfer_amount)
            } else if !external_transfer_amount.is_zero() {
                RebalanceAction::OnchainDeposit(external_transfer_amount)
            } else {
                RebalanceAction::DoNothing
            }
        } else if abs_exposure_in_btc.is_zero()
            && total_collateral_in_btc > Decimal::ZERO
            && abs_liability_in_cents >= Decimal::ZERO
            && abs_liability_in_cents < self.config.minimum_transfer_amount_cents
        {
            let transfer_size_in_btc = floor_btc(total_collateral_in_btc);

            let (internal_transfer_amount, external_transfer_amount) = split_withdraw(
                funding_btc_total_balance,
                transfer_size_in_btc,
                self.config.minimum_funding_balance_btc,
            );

            if !internal_transfer_amount.is_zero() {
                RebalanceAction::TransferTradingToFunding(internal_transfer_amount)
            } else if !external_transfer_amount.is_zero() {
                RebalanceAction::OnchainWithdraw(external_transfer_amount)
            } else {
                RebalanceAction::DoNothing
            }
        } else if abs_liability_in_btc
            > total_collateral_in_btc * self.config.high_bound_ratio_leverage
        {
            let new_collateral_in_btc =
                abs_liability_in_btc / self.config.high_safebound_ratio_leverage;
            let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

            let (internal_transfer_amount, external_transfer_amount) = split_deposit(
                funding_btc_total_balance,
                transfer_size_in_btc,
                self.config.minimum_funding_balance_btc,
            );

            if !internal_transfer_amount.is_zero() {
                RebalanceAction::TransferFundingToTrading(internal_transfer_amount)
            } else if !external_transfer_amount.is_zero() {
                RebalanceAction::OnchainDeposit(external_transfer_amount)
            } else {
                RebalanceAction::DoNothing
            }
        } else if abs_exposure_in_btc
            < total_collateral_in_btc * self.config.low_bound_ratio_leverage
        {
            let new_collateral_in_btc =
                abs_exposure_in_btc / self.config.low_safebound_ratio_leverage;
            let transfer_size_in_btc = floor_btc(total_collateral_in_btc - new_collateral_in_btc);

            let (internal_transfer_amount, external_transfer_amount) = split_withdraw(
                funding_btc_total_balance,
                transfer_size_in_btc,
                self.config.minimum_funding_balance_btc,
            );

            if !internal_transfer_amount.is_zero() {
                RebalanceAction::TransferTradingToFunding(internal_transfer_amount)
            } else if !external_transfer_amount.is_zero() {
                RebalanceAction::OnchainWithdraw(external_transfer_amount)
            } else {
                RebalanceAction::DoNothing
            }
        } else if abs_exposure_in_btc
            > total_collateral_in_btc
                * self.config.high_bound_buffer_percentage
                * self.config.high_bound_ratio_leverage
        {
            let new_collateral_in_btc =
                abs_exposure_in_btc / self.config.high_safebound_ratio_leverage;
            let transfer_size_in_btc = round_btc(new_collateral_in_btc - total_collateral_in_btc);

            let (internal_transfer_amount, external_transfer_amount) = split_deposit(
                funding_btc_total_balance,
                transfer_size_in_btc,
                self.config.minimum_funding_balance_btc,
            );

            if !internal_transfer_amount.is_zero() {
                RebalanceAction::TransferFundingToTrading(internal_transfer_amount)
            } else if !external_transfer_amount.is_zero() {
                RebalanceAction::OnchainDeposit(external_transfer_amount)
            } else {
                RebalanceAction::DoNothing
            }
        } else {
            RebalanceAction::DoNothing
        }
    }
}

fn split_deposit(
    funding_btc_total_balance: Decimal,
    amount_in_btc: Decimal,
    minimum_funding_balance_btc: Decimal,
) -> (Decimal, Decimal) {
    let internal_transfer_amount = std::cmp::min(funding_btc_total_balance, amount_in_btc);
    let new_funding_balance = funding_btc_total_balance - internal_transfer_amount;
    let funding_refill = std::cmp::max(
        Decimal::ZERO,
        minimum_funding_balance_btc - new_funding_balance,
    );
    let missing_amount = amount_in_btc - internal_transfer_amount;
    let external_transfer_amount = missing_amount + funding_refill;

    (internal_transfer_amount, external_transfer_amount)
}

fn split_withdraw(
    funding_btc_total_balance: Decimal,
    amount_in_btc: Decimal,
    minimum_funding_balance_btc: Decimal,
) -> (Decimal, Decimal) {
    let internal_transfer_amount = amount_in_btc;
    let external_transfer_amount = std::cmp::max(
        Decimal::ZERO,
        amount_in_btc + funding_btc_total_balance - minimum_funding_balance_btc,
    );

    (internal_transfer_amount, external_transfer_amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FundingSectionConfig, HedgingSectionConfig};
    use rust_decimal_macros::dec;

    #[test]
    fn do_nothing_conditions() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(-10_000));
        let total_collateral: Decimal =
            liability / funding_adjustment.config.high_safebound_ratio_leverage;
        let btc_price: Decimal = dec!(1);
        let adjustment = funding_adjustment.determine_action(
            liability,
            exposure,
            total_collateral,
            btc_price,
            funding_adjustment.config.minimum_funding_balance_btc,
        );
        assert_eq!(adjustment, RebalanceAction::DoNothing);
    }

    #[test]
    fn initial_conditions() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(0));
        let total_collateral: Decimal = dec!(0);
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal =
            round_btc(liability / funding_adjustment.config.high_safebound_ratio_leverage);
        let (_, expected_external) = split_deposit(
            funding_btc_total_balance,
            expected_total,
            funding_adjustment.config.minimum_funding_balance_btc,
        );
        let adjustment = funding_adjustment.determine_action(
            liability,
            exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::OnchainDeposit(expected_external)
        );
    }

    #[test]
    fn terminal_conditions() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let liability = SyntheticCentLiability::try_from(
            funding_adjustment.config.minimum_transfer_amount_cents / dec!(2),
        )
        .unwrap();
        let exposure = SyntheticCentExposure::from(dec!(0));
        let total_collateral: Decimal =
            liability / funding_adjustment.config.high_safebound_ratio_leverage;
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal = floor_btc(total_collateral);
        let (expected_internal, _) = split_withdraw(
            funding_btc_total_balance,
            expected_total,
            funding_adjustment.config.minimum_funding_balance_btc,
        );
        let adjustment = funding_adjustment.determine_action(
            liability,
            exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::TransferTradingToFunding(expected_internal)
        );
    }

    #[test]
    fn user_activity_tracking() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = SyntheticCentExposure::from(dec!(-3_000));
        let total_collateral: Decimal =
            exposure / funding_adjustment.config.high_safebound_ratio_leverage;
        let funding_btc_total_balance: Decimal = dec!(2_000);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal = round_btc(
            liability / funding_adjustment.config.high_safebound_ratio_leverage - total_collateral,
        );
        let (expected_internal, _) = split_deposit(
            funding_btc_total_balance,
            expected_total,
            funding_adjustment.config.minimum_funding_balance_btc,
        );
        let adjustment = funding_adjustment.determine_action(
            liability,
            exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::TransferFundingToTrading(expected_internal)
        );
    }

    #[test]
    fn counterparty_risk_avoidance() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = dec!(10_000);
        let signed_exposure = SyntheticCentExposure::from(-exposure);
        let total_collateral: Decimal = dec!(10_000);
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal = floor_btc(
            total_collateral - exposure / funding_adjustment.config.low_safebound_ratio_leverage,
        );
        let (expected_internal, _) = split_withdraw(
            funding_btc_total_balance,
            expected_total,
            funding_adjustment.config.minimum_funding_balance_btc,
        );
        let adjustment = funding_adjustment.determine_action(
            liability,
            signed_exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::TransferTradingToFunding(expected_internal)
        );
    }

    #[test]
    fn liquidation_risk_avoidance() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let liability = SyntheticCentLiability::try_from(dec!(10_000)).unwrap();
        let exposure = dec!(10_100);
        let signed_exposure = SyntheticCentExposure::from(-exposure);
        let total_collateral: Decimal =
            exposure / funding_adjustment.config.high_bound_ratio_leverage;
        let funding_btc_total_balance: Decimal = dec!(0);
        let btc_price: Decimal = dec!(1);
        let expected_total: Decimal = round_btc(
            exposure / funding_adjustment.config.high_safebound_ratio_leverage - total_collateral,
        );
        let (_, expected_external) = split_deposit(
            funding_btc_total_balance,
            expected_total,
            funding_adjustment.config.minimum_funding_balance_btc,
        );
        let adjustment = funding_adjustment.determine_action(
            liability,
            signed_exposure,
            total_collateral,
            btc_price,
            funding_btc_total_balance,
        );
        assert_eq!(
            adjustment,
            RebalanceAction::OnchainDeposit(expected_external)
        );
    }

    #[test]
    fn split_deposit_no_funding() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let funding_btc_total_balance: Decimal = dec!(0);
        let amount_in_btc: Decimal = dec!(1);
        let expected_internal = dec!(0);
        let expected_external =
            amount_in_btc + funding_adjustment.config.minimum_funding_balance_btc;
        let (internal, external) = split_deposit(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_equal_funding_amount_under() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let funding_btc_total_balance: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc;
        let amount_in_btc: Decimal = funding_btc_total_balance / dec!(5) * dec!(3);
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal;
        let (internal, external) = split_deposit(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_equal_funding_amount_equal() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let funding_btc_total_balance: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc;
        let amount_in_btc: Decimal = funding_btc_total_balance;
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal;
        let (internal, external) = split_deposit(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_equal_funding_amount_over() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let funding_btc_total_balance: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc;
        let amount_in_btc: Decimal = funding_btc_total_balance * dec!(2);
        let expected_internal = funding_btc_total_balance;
        let expected_external = amount_in_btc;
        let (internal, external) = split_deposit(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_more_funding_amount_under() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let extra_funding = dec!(0.3);
        let funding_btc_total_balance: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc + extra_funding;
        let amount_in_btc: Decimal = funding_adjustment.config.minimum_funding_balance_btc;
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal - extra_funding;
        let (internal, external) = split_deposit(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_more_funding_amount_equal() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let extra_funding = dec!(0.3);
        let funding_btc_total_balance: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc + extra_funding;
        let amount_in_btc: Decimal = funding_btc_total_balance;
        let expected_internal = amount_in_btc;
        let expected_external = funding_adjustment.config.minimum_funding_balance_btc;
        let (internal, external) = split_deposit(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_deposit_more_funding_amount_over() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let extra_funding = dec!(0.3);
        let funding_btc_total_balance: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc + extra_funding;
        let amount_in_btc: Decimal = funding_btc_total_balance * dec!(2);
        let expected_internal = funding_btc_total_balance;
        let expected_external = amount_in_btc - extra_funding;
        let (internal, external) = split_deposit(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_no_funding_amount_under() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let funding_btc_total_balance: Decimal = dec!(0);
        let amount_in_btc: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc / dec!(5) * dec!(3);
        let expected_internal = amount_in_btc;
        let expected_external = dec!(0);
        let (internal, external) = split_withdraw(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_no_funding_amount_equal() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let funding_btc_total_balance: Decimal = dec!(0);
        let amount_in_btc: Decimal = funding_adjustment.config.minimum_funding_balance_btc;
        let expected_internal = amount_in_btc;
        let expected_external = dec!(0);
        let (internal, external) = split_withdraw(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_no_funding_amount_over() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let funding_btc_total_balance: Decimal = dec!(0);
        let amount_in_btc: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc * dec!(2);
        let expected_internal = amount_in_btc;
        let expected_external =
            amount_in_btc - funding_adjustment.config.minimum_funding_balance_btc;
        let (internal, external) = split_withdraw(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_equal_funding() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let funding_btc_total_balance: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc;
        let amount_in_btc: Decimal = dec!(1);
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal;
        let (internal, external) = split_withdraw(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

        assert_eq!(internal, expected_internal);
        assert_eq!(external, expected_external);
    }

    #[test]
    fn split_withdraw_more_funding() {
        let funding_adjustment = FundingAdjustment {
            config: FundingSectionConfig::default(),
            hedging_config: HedgingSectionConfig::default(),
        };
        let extra_funding = dec!(0.3);
        let funding_btc_total_balance: Decimal =
            funding_adjustment.config.minimum_funding_balance_btc + extra_funding;
        let amount_in_btc: Decimal = funding_adjustment.config.minimum_funding_balance_btc;
        let expected_internal = amount_in_btc;
        let expected_external = expected_internal + extra_funding;
        let (internal, external) = split_withdraw(
            funding_btc_total_balance,
            amount_in_btc,
            funding_adjustment.config.minimum_funding_balance_btc,
        );

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
