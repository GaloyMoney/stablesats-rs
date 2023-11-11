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

pub struct PriceCalculator {
    fee_calculator: FeeCalculator,
    price_mixer: PriceMixer,
}

pub struct ConversionResult {
    pub sats: Satoshis,
    pub cents: UsdCents,
    pub sat_spread: Satoshis,
    pub cent_spread: UsdCents,
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
        let cent_spread = &cents_after_fee - &cents;
        let sat_spread = self.calculate_sat_spread(&sats, &cents, &cents_after_fee);
        Ok(ConversionResult {
            sats,
            cents: cents_after_fee,
            sat_spread,
            cent_spread,
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
        let sat_spread = &sats_after_fee - &sats;
        let cent_spread = self.calculate_cent_spread(&cents, &sats, &sats_after_fee);
        Ok(ConversionResult {
            sats: sats_after_fee,
            cents,
            sat_spread,
            cent_spread,
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
        let cent_spread = &cents_after_fee - &cents;
        let sat_spread = self.calculate_sat_spread(&sats, &cents, &cents_after_fee);
        Ok(ConversionResult {
            sats,
            cents: cents_after_fee,
            sat_spread,
            cent_spread,
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
        let sat_spread = &sats_after_fee - &sats;
        let cent_spread = self.calculate_cent_spread(&cents, &sats, &sats_after_fee);
        Ok(ConversionResult {
            sats: sats_after_fee,
            cents,
            sat_spread,
            cent_spread,
        })
    }

    fn calculate_cent_spread(
        &self,
        cents: &UsdCents,
        sats: &Satoshis,
        sats_after_fee: &Satoshis,
    ) -> UsdCents {
        UsdCents::from(
            (cents.amount()
                * ((sats.amount() - sats_after_fee.amount()) / sats_after_fee.amount()))
            .round(),
        )
    }

    fn calculate_sat_spread(
        &self,
        sats: &Satoshis,
        cents: &UsdCents,
        cents_after_fee: &UsdCents,
    ) -> Satoshis {
        use rust_decimal::Decimal;
        if cents_after_fee.amount() == &Decimal::ZERO {
            return Satoshis::from(Decimal::ZERO);
        }
        Satoshis::from(
            (sats.amount()
                * ((cents.amount() - cents_after_fee.amount()) / cents_after_fee.amount()))
            .round(),
        )
    }
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
}
