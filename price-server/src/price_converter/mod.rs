use rust_decimal::Decimal;

use crate::{
    currency::{Sats, UsdCents},
    QuotePrice,
};

pub struct VolumeBasedPriceConverter<'a, I: Iterator<Item = (&'a QuotePrice, &'a Decimal)> + Clone>
{
    pairs: I,
}
impl<'a, I: Iterator<Item = (&'a QuotePrice, &'a Decimal)> + Clone>
    VolumeBasedPriceConverter<'a, I>
{
    pub fn new(pairs: I) -> Self {
        Self { pairs }
    }

    pub fn cents_from_sats(&self, sats: Sats) -> UsdCents {
        UsdCents::from_decimal(sats.amount() * self.weighted_price_of_volume(*sats.amount()))
    }

    pub fn sats_from_cents(&self, cents: UsdCents) -> Sats {
        Sats::from_decimal(cents.amount() / self.weighted_price_of_volume(*cents.amount()))
    }

    fn weighted_price_of_volume(&self, total_volume: Decimal) -> Decimal {
        let mut price_acc = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;

        let pairs = self.pairs.clone();
        for (price, qty) in pairs {
            if (volume_acc + qty) < total_volume {
                volume_acc += qty;
                price_acc += price.inner() * qty;
                continue;
            } else {
                let remaining_volume = total_volume - volume_acc;
                price_acc += price.inner() * remaining_volume;
                break;
            }
        }

        price_acc / total_volume
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use serde::Deserialize;
    use std::fs;

    use crate::OrderBookView;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct SnapshotFixture {
        payload: OrderBookView,
    }

    fn load_order_book(filename: &str) -> anyhow::Result<SnapshotFixture> {
        let contents = fs::read_to_string(format!(
            "./tests/fixtures/order-book-payload-{}.json",
            filename
        ))
        .expect(&format!("Couldn't load fixture {}", filename));

        let res = serde_json::from_str::<SnapshotFixture>(&contents)?;
        Ok(res)
    }

    #[test]
    fn weighted_average_ask_price() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("real")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter());
        let volumes = vec![
            Sats::from_decimal(dec!(1)),
            Sats::from_decimal(dec!(10)),
            Sats::from_decimal(dec!(20)),
            Sats::from_decimal(dec!(100)),
            Sats::from_decimal(dec!(110)),
        ];
        let expected_prices = vec![dec!(0.1), dec!(0.1), dec!(0.15), dec!(0.19), dec!(0.2)];

        for (idx, sats) in volumes.into_iter().enumerate() {
            let mut price = converter.weighted_price_of_volume(*sats.amount());
            price.rescale(7);

            assert_eq!(price, expected_prices[idx]);
        }

        Ok(())
    }

    #[test]
    fn weighted_average_bid_price() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("real")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());
        let volumes = vec![
            Sats::from_decimal(dec!(1)),
            Sats::from_decimal(dec!(10)),
            Sats::from_decimal(dec!(20)),
            Sats::from_decimal(dec!(100)),
            Sats::from_decimal(dec!(200)),
        ];
        let expected_prices = vec![dec!(0.2), dec!(0.2), dec!(0.175), dec!(0.155), dec!(0.1275)];

        for (idx, sats) in volumes.into_iter().enumerate() {
            let mut price = converter.weighted_price_of_volume(*sats.amount());
            price.rescale(7);

            assert_eq!(price, expected_prices[idx]);
        }

        Ok(())
    }

    #[test]
    fn cents_from_sats_volume() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("real")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());
        let sats_volume = Sats::from_decimal(dec!(100_000_000));

        let cents = converter.cents_from_sats(sats_volume);

        assert_eq!(cents.floor(), UsdCents::from_major(65));

        Ok(())
    }

    #[test]
    fn sats_from_cents_volume() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("real")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());
        let cents_volume = UsdCents::from_decimal(dec!(10));

        let sats = converter.sats_from_cents(cents_volume);

        assert_eq!(sats.floor(), Sats::from_major(50));

        Ok(())
    }
}
