use rust_decimal::Decimal;
use rust_decimal_macros::dec;

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
        let wap_sats = self.weighted_price_of_volume(*sats.amount());
        let cents = UsdCents::from_decimal(sats.amount() * wap_sats);
        cents
    }

    pub fn sats_from_cents(&self, cents: UsdCents) -> Sats {
        if cents.amount() == &dec!(0) {
            return Sats::from_major(0);
        }
        Sats::from_decimal(cents.amount() / self.weighted_price_of_volume(*cents.amount()))
    }

    fn weighted_price_of_volume(&self, total_volume: Decimal) -> Decimal {
        let mut price_acc = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;

        if total_volume == dec!(0) {
            return Decimal::ZERO;
        }

        let pairs = self.pairs.clone();
        for (price, qty) in pairs {
            if (volume_acc + qty) < total_volume {
                volume_acc += qty;
                price_acc += price.inner() * qty;
                continue;
            } else {
                let remaining_volume = total_volume - volume_acc;
                volume_acc += remaining_volume;
                price_acc += price.inner() * remaining_volume;
                break;
            }
        }
        price_acc / volume_acc
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
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter());
        let sats_volume = vec![
            Sats::from_decimal(dec!(0)),
            Sats::from_decimal(dec!(110)),
            Sats::from_decimal(dec!(1000)),
            Sats::from_decimal(dec!(100_000_000)),
        ];

        let expected = vec![
            UsdCents::from_major(0),
            UsdCents::from_major(22),
            UsdCents::from_major(289),
            UsdCents::from_major(29_000_000),
        ];

        for (idx, sats) in sats_volume.into_iter().enumerate() {
            let cents = converter.cents_from_sats(sats);

            assert_eq!(cents.floor(), expected[idx]);
        }

        Ok(())
    }

    #[test]
    fn sats_from_cents_volume() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("real")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter());
        let cents_volume = UsdCents::from_decimal(dec!(29_000_000));

        let sats = converter.sats_from_cents(cents_volume);

        assert_eq!(sats.floor(), Sats::from_major(100_000_000));

        Ok(())
    }

    #[test]
    fn weighted_price_of_sat_or_cent_volume() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("actual")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter());
        let sats_volumes = vec![
            Sats::from_decimal(dec!(0)),
            Sats::from_decimal(dec!(1)),
            Sats::from_decimal(dec!(10)),
            Sats::from_decimal(dec!(100)),
            Sats::from_decimal(dec!(1000)),
            Sats::from_decimal(dec!(100_000_000)),
        ];

        for (idx, sats) in sats_volumes.iter().enumerate() {
            let cents = converter.cents_from_sats(sats.clone());
            let sats = converter.sats_from_cents(cents);
            assert_eq!(sats_volumes[idx], sats);
        }

        Ok(())
    }
}
