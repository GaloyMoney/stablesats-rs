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
            .apply(|p| p.buy_usd().cents_from_sats(sats.clone()))
            .await?;
        let cents_after_fee = self
            .fee_calculator
            .decrease_by_fee(immediate_execution, cents.clone())
            .floor();
        let cents_spread = UsdCents::from(cents_after_fee.amount() - cents.amount());
        let sats_spread = sats_spread(*sats.amount(), *cents.amount(), *cents_after_fee.amount());
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
            .apply(|p| p.buy_usd().sats_from_cents(cents.clone()))
            .await?;
        let sats_after_fee = self
            .fee_calculator
            .increase_by_fee(immediate_execution, sats.clone())
            .ceil();
        let sats_spread = Satoshis::from(sats_after_fee.amount() - sats.amount());
        let cents_spread = cents_spread(*cents.amount(), *sats.amount(), *sats_after_fee.amount());
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
            .apply(|p| p.sell_usd().cents_from_sats(sats.clone()))
            .await?;
        let cents_after_fee = self
            .fee_calculator
            .increase_by_fee(immediate_execution, cents.clone())
            .ceil();
        let cents_spread = UsdCents::from(cents_after_fee.amount() - cents.amount());
        let sats_spread = sats_spread(*sats.amount(), *cents.amount(), *cents_after_fee.amount());
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
            .apply(|p| p.sell_usd().sats_from_cents(cents.clone()))
            .await?;
        let sats_after_fee = self
            .fee_calculator
            .decrease_by_fee(immediate_execution, sats.clone())
            .floor();
        let sats_spread = Satoshis::from(sats_after_fee.amount() - sats.amount());
        let cents_spread = cents_spread(*cents.amount(), *sats.amount(), *sats_after_fee.amount());
        Ok(ConversionResult {
            sats: sats_after_fee,
            cents,
            sats_spread,
            cents_spread,
        })
    }
}

fn sats_spread(sats: Decimal, cents: Decimal, cents_after_fee: Decimal) -> Satoshis {
    if cents_after_fee == Decimal::ZERO {
        return Satoshis::from(Decimal::ZERO);
    }
    Satoshis::from((sats * ((cents - cents_after_fee) / cents_after_fee)).round())
}

fn cents_spread(cents: Decimal, sats: Decimal, sats_after_fee: Decimal) -> UsdCents {
    if sats_after_fee == Decimal::ZERO {
        return UsdCents::from(Decimal::ZERO);
    }
    UsdCents::from((cents * ((sats - sats_after_fee) / sats_after_fee)).round())
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
        let res = calc
            .cents_from_sats_for_buy(Satoshis::from(Decimal::ONE), true)
            .await?;
        assert_eq!(res.cents, UsdCents::from(Decimal::ZERO));

        let res = calc
            .cents_from_sats_for_buy(Satoshis::from(dec!(100_000_000)), false)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(89_900)));
        let res = calc
            .cents_from_sats_for_buy(Satoshis::from(Decimal::ONE), false)
            .await?;
        assert_eq!(res.cents, UsdCents::from(Decimal::ZERO));

        let res = calc
            .sats_from_cents_for_buy(UsdCents::from(dec!(1_000_000)), true)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(1_011_000_000)));
        let res = calc
            .sats_from_cents_for_buy(UsdCents::from(Decimal::ONE), true)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(1_011)));

        let res = calc
            .sats_from_cents_for_buy(UsdCents::from(dec!(1_000_000)), false)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(1_101_000_000)));
        let res = calc
            .sats_from_cents_for_buy(UsdCents::from(Decimal::ONE), false)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(1_101)));

        Ok(())
    }

    #[tokio::test]
    async fn usd_sell() -> anyhow::Result<()> {
        let calc = PriceCalculator::new(fee_config(), mixer());
        let res = calc
            .cents_from_sats_for_sell(Satoshis::from(dec!(100_000_000)), true)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(1_011_000)));
        let res = calc
            .cents_from_sats_for_sell(Satoshis::from(Decimal::ONE), true)
            .await?;
        assert_eq!(res.cents, UsdCents::from(Decimal::ONE));

        let res = calc
            .cents_from_sats_for_sell(Satoshis::from(dec!(100_000_000)), false)
            .await?;
        assert_eq!(res.cents, UsdCents::from(dec!(1_101_000)));
        let res = calc
            .cents_from_sats_for_sell(Satoshis::from(Decimal::ONE), false)
            .await?;
        assert_eq!(res.cents, UsdCents::from(Decimal::ONE));

        let res = calc
            .sats_from_cents_for_sell(UsdCents::from(dec!(1_000_000)), true)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(98_900_000)));
        let res = calc
            .sats_from_cents_for_sell(UsdCents::from(Decimal::ONE), true)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(98)));

        let res = calc
            .sats_from_cents_for_sell(UsdCents::from(dec!(1_000_000)), false)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(89_900_000)));
        let res = calc
            .sats_from_cents_for_sell(UsdCents::from(Decimal::ONE), false)
            .await?;
        assert_eq!(res.sats, Satoshis::from(dec!(89)));

        Ok(())
    }

    #[test]
    fn test_sats_spread() {
        let sats = dec!(1000);
        let cents = dec!(50);
        let cents_after_fee = dec!(45);
        let res = sats_spread(sats, cents, cents_after_fee);
        assert_eq!(res, Satoshis::from(dec!(111)));

        let sats = dec!(10);
        let cents = dec!(1);
        let cents_after_fee = dec!(0);
        let res = sats_spread(sats, cents, cents_after_fee);
        assert_eq!(res, Satoshis::from(dec!(0)));
    }

    #[test]
    fn test_cents_spread() {
        let cents = dec!(50);
        let sats = dec!(1000);
        let sats_after_fee = dec!(1111);
        let res = cents_spread(cents, sats, sats_after_fee);
        assert_eq!(res, UsdCents::from(dec!(-5)));

        let cents = dec!(1);
        let sats = dec!(1);
        let sats_after_fee = dec!(0);
        let res = cents_spread(cents, sats, sats_after_fee);
        assert_eq!(res, UsdCents::from(dec!(0)));
    }
}
