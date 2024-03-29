use rust_decimal::Decimal;

mod converter;
mod error;
mod fee;
mod mixer;
mod tick_converter;
mod traits;

use crate::currency::*;

pub use converter::*;
pub use error::*;
pub use fee::*;
pub use mixer::*;
pub use tick_converter::*;
pub use traits::*;

#[derive(Debug)]
pub struct ConversionResult {
    pub sats: Satoshis,
    pub cents: UsdCents,
    pub sats_spread: Satoshis,
    pub cents_spread: UsdCents,
}

pub struct PriceCalculator {
    fee_calculator: FeeCalculator,
    price_mixer: PriceMixer,
}

impl PriceCalculator {
    pub fn new(fee_cfg: QuotesFeeCalculatorConfig, price_mixer: PriceMixer) -> Self {
        Self {
            fee_calculator: FeeCalculator::new(fee_cfg),
            price_mixer,
        }
    }

    pub async fn cents_from_sats_for_buy(
        &self,
        sats: Satoshis,
        immediate_execution: bool,
    ) -> Result<ConversionResult, ExchangePriceCacheError> {
        let cents = self
            .price_mixer
            .apply(|p| p.buy_usd().cents_from_sats(sats))
            .await?
            .floor();
        let cents_after_fee = self
            .fee_calculator
            .decrease_by_fee(immediate_execution, cents)
            .floor();
        let cents_spread = cents_after_fee - cents;
        let sats_spread = sats_spread(sats, cents, cents_after_fee);
        Ok(ConversionResult {
            sats,
            cents: cents_after_fee,
            sats_spread,
            cents_spread,
        })
    }

    pub async fn sats_from_cents_for_buy(
        &self,
        cents: UsdCents,
        immediate_execution: bool,
    ) -> Result<ConversionResult, ExchangePriceCacheError> {
        let sats = self
            .price_mixer
            .apply(|p| p.buy_usd().sats_from_cents(cents))
            .await?
            .ceil();
        let sats_after_fee = self
            .fee_calculator
            .increase_by_fee(immediate_execution, sats)
            .ceil();
        let sats_spread = sats_after_fee - sats;
        let cents_spread = cents_spread(cents, sats, sats_after_fee);
        Ok(ConversionResult {
            sats: sats_after_fee,
            cents,
            sats_spread,
            cents_spread,
        })
    }

    pub async fn cents_from_sats_for_sell(
        &self,
        sats: Satoshis,
        immediate_execution: bool,
    ) -> Result<ConversionResult, ExchangePriceCacheError> {
        let cents = self
            .price_mixer
            .apply(|p| p.sell_usd().cents_from_sats(sats))
            .await?
            .ceil();
        let cents_after_fee = self
            .fee_calculator
            .increase_by_fee(immediate_execution, cents)
            .ceil();
        let cents_spread = cents_after_fee - cents;
        let sats_spread = sats_spread(sats, cents, cents_after_fee);
        Ok(ConversionResult {
            sats,
            cents: cents_after_fee,
            sats_spread,
            cents_spread,
        })
    }

    pub async fn sats_from_cents_for_sell(
        &self,
        cents: UsdCents,
        immediate_execution: bool,
    ) -> Result<ConversionResult, ExchangePriceCacheError> {
        let sats = self
            .price_mixer
            .apply(|p| p.sell_usd().sats_from_cents(cents))
            .await?
            .floor();
        let sats_after_fee = self
            .fee_calculator
            .decrease_by_fee(immediate_execution, sats)
            .floor();
        let sats_spread = sats_after_fee - sats;
        let cents_spread = cents_spread(cents, sats, sats_after_fee);
        Ok(ConversionResult {
            sats: sats_after_fee,
            cents,
            sats_spread,
            cents_spread,
        })
    }
}

fn sats_spread(sats: Satoshis, cents: UsdCents, cents_after_fee: UsdCents) -> Satoshis {
    if cents_after_fee == UsdCents::from(Decimal::ZERO) {
        return Satoshis::from(Decimal::ZERO);
    }
    Satoshis::from(
        (sats.amount() * ((cents.amount() - cents_after_fee.amount()) / cents_after_fee.amount()))
            .floor(),
    )
}

fn cents_spread(cents: UsdCents, sats: Satoshis, sats_after_fee: Satoshis) -> UsdCents {
    if sats_after_fee == Satoshis::from(Decimal::ZERO) {
        return UsdCents::from(Decimal::ZERO);
    }
    UsdCents::from(
        (cents.amount() * ((sats.amount() - sats_after_fee.amount()) / sats_after_fee.amount()))
            .floor(),
    )
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use super::*;

    fn mixer() -> PriceMixer {
        let mut price_mixer = PriceMixer::new();
        let dummy = DummyProvider::new(
            UsdCents::from(Decimal::new(1_000_000_000, 12)),
            UsdCents::from(Decimal::new(10_000_000_000, 12)),
        );
        price_mixer.add_provider("dummy", dummy, Decimal::ONE);
        price_mixer
    }

    fn fee_config() -> QuotesFeeCalculatorConfig {
        QuotesFeeCalculatorConfig {
            base_fee_rate: dec!(0.001),
            immediate_fee_rate: dec!(0.01),
            delayed_fee_rate: dec!(0.1),
        }
    }

    #[tokio::test]
    async fn usd_buy() -> anyhow::Result<()> {
        let calc = PriceCalculator::new(fee_config(), mixer());
        let res = calc
            .cents_from_sats_for_buy(Satoshis::from(dec!(100_000_000)), true)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(98_900)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(-1_100)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(1_112_234)));
        let res = calc
            .cents_from_sats_for_buy(Satoshis::from(Decimal::ONE), true)
            .await?;
        assert_eq!(res.cents, UsdCents::from(Decimal::ZERO));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(0)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(0)));

        let res = calc
            .cents_from_sats_for_buy(Satoshis::from(dec!(100_000_000)), false)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(89_900)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(-10_100)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(11_234_705)));
        let res = calc
            .cents_from_sats_for_buy(Satoshis::from(Decimal::ONE), false)
            .await?;
        assert_eq!(res.cents, UsdCents::from(Decimal::ZERO));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(0)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(0)));

        let res = calc
            .sats_from_cents_for_buy(UsdCents::from(dec!(1_000_000)), true)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(1_011_000_000)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(-10_881)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(11_000_000)));
        let res = calc
            .sats_from_cents_for_buy(UsdCents::from(Decimal::ONE), true)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(1_011)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(-1)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(11)));

        let res = calc
            .sats_from_cents_for_buy(UsdCents::from(dec!(1_000_000)), false)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(1_101_000_000)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(-91_735)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(101_000_000)));
        let res = calc
            .sats_from_cents_for_buy(UsdCents::from(Decimal::ONE), false)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(1_101)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(-1)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(101)));

        Ok(())
    }

    #[tokio::test]
    async fn usd_sell() -> anyhow::Result<()> {
        let calc = PriceCalculator::new(fee_config(), mixer());
        let res = calc
            .cents_from_sats_for_sell(Satoshis::from(dec!(100_000_000)), true)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(1_011_000)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(11_000)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(-1_088_032)));
        let res = calc
            .cents_from_sats_for_sell(Satoshis::from(Decimal::ONE), true)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(2)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(1)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(-1)));

        let res = calc
            .cents_from_sats_for_sell(Satoshis::from(dec!(100_000_000)), false)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(1_101_000)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(101_000)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(-9_173_479)));
        let res = calc
            .cents_from_sats_for_sell(Satoshis::from(Decimal::ONE), false)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(2)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(1)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(-1)));

        let res = calc
            .sats_from_cents_for_sell(UsdCents::from(dec!(1_000_000)), true)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(98_900_000)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(11_122)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(-1_100_000)));
        let res = calc
            .sats_from_cents_for_sell(UsdCents::from(Decimal::ONE), true)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(98)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(0)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(-2)));

        let res = calc
            .sats_from_cents_for_sell(UsdCents::from(dec!(1_000_000)), false)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(89_900_000)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(112_347)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(-10_100_000)));
        let res = calc
            .sats_from_cents_for_sell(UsdCents::from(Decimal::ONE), false)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(89)));
        assert_eq!(res.cents_spread, UsdCents::from(dec!(0)));
        assert_eq!(res.sats_spread, Satoshis::from(dec!(-11)));

        Ok(())
    }

    #[test]
    fn test_sats_spread() {
        let sats = Satoshis::from(dec!(1_000));
        let cents = UsdCents::from(dec!(50));
        let cents_after_fee = UsdCents::from(dec!(45));
        let res = sats_spread(sats, cents, cents_after_fee);
        assert_eq!(res, Satoshis::from(dec!(111)));

        let sats = Satoshis::from(dec!(10));
        let cents = UsdCents::from(Decimal::ONE);
        let cents_after_fee = UsdCents::from(Decimal::ZERO);
        let res = sats_spread(sats, cents, cents_after_fee);
        assert_eq!(res, Satoshis::from(dec!(0)));
    }

    #[test]
    fn test_cents_spread() {
        let cents = UsdCents::from(dec!(50));
        let sats = Satoshis::from(dec!(1_000));
        let sats_after_fee = Satoshis::from(dec!(1_111));
        let res = cents_spread(cents, sats, sats_after_fee);
        assert_eq!(res, UsdCents::from(dec!(-5)));

        let cents = UsdCents::from(Decimal::ONE);
        let sats = Satoshis::from(Decimal::ONE);
        let sats_after_fee = Satoshis::from(Decimal::ZERO);
        let res = cents_spread(cents, sats, sats_after_fee);
        assert_eq!(res, UsdCents::from(dec!(0)));
    }
}
