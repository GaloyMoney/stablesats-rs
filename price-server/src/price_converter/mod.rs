use std::collections::BTreeMap;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    currency::{Sats, UsdCents},
    QuotePrice,
};

#[derive(Debug)]
pub struct VolumeBasedPriceConverter {
    side: BTreeMap<QuotePrice, Decimal>,
    reverse: bool,
}
impl VolumeBasedPriceConverter {
    pub fn new(side: BTreeMap<QuotePrice, Decimal>, reverse: bool) -> Self {
        Self { side, reverse }
    }

    pub fn cents_from_sats(&self, sats: Sats) -> UsdCents {
        UsdCents::from_decimal(sats.amount() * self.weighted_price_of_volume(*sats.amount()))
    }

    pub fn sats_from_cents(&self, cents: UsdCents) -> Sats {
        Sats::from_decimal(cents.amount() / self.weighted_price_of_volume(*cents.amount()))
    }

    fn weighted_price_of_volume(&self, mut volume: Decimal) -> Decimal {
        let mut price_volume_pair = Vec::new();
        // let mut volume = sats.amount().to_owned();

        let side_collection = if self.reverse {
            self.side.iter().rev().collect::<Vec<_>>()
        } else {
            self.side.iter().collect::<Vec<_>>()
        };

        for (price, qty) in side_collection {
            if qty < &volume {
                price_volume_pair.push((price, *qty));
                let new_volume = volume - qty;
                volume = new_volume;
                continue;
            } else {
                let new_qty = volume;
                price_volume_pair.push((price, new_qty));
                break;
            }
        }

        let acc_price_qty = price_volume_pair
            .iter()
            .fold(dec!(0), |acc, (price, qty)| acc + (price.inner() * *qty));

        let acc_size = price_volume_pair
            .iter()
            .fold(dec!(0), |acc, (_, qty)| acc + *qty);

        acc_price_qty / acc_size
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde::Deserialize;

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
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks, false);
        let volumes = vec![
            Sats::from_decimal(dec!(1)),
            Sats::from_decimal(dec!(2_016)),
            Sats::from_decimal(dec!(3_000)),
            Sats::from_decimal(dec!(5_000)),
            Sats::from_decimal(dec!(10_000)),
            Sats::from_decimal(dec!(100_000)),
            Sats::from_decimal(dec!(1_000_000)),
        ];
        let expected_prices = vec![
            dec!(0.0203778),
            dec!(0.0203778),
            dec!(0.0203781),
            dec!(0.0203789),
            dec!(0.0203810),
            dec!(0.0204431),
            dec!(0.0205762),
        ];

        for (idx, sats) in volumes.into_iter().enumerate() {
            let mut price = converter.weighted_price_of_volume(*sats.amount());
            price.rescale(7);
            dbg!(price);

            assert_eq!(price, expected_prices[idx]);
        }

        Ok(())
    }

    #[test]
    fn weighted_average_bid_price() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("real")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids, true);
        let volumes = vec![
            Sats::from_decimal(dec!(1)),
            Sats::from_decimal(dec!(200)),
            Sats::from_decimal(dec!(3_000)),
            Sats::from_decimal(dec!(5_000)),
            Sats::from_decimal(dec!(10_000)),
            Sats::from_decimal(dec!(100_000)),
            Sats::from_decimal(dec!(1_000_000)),
        ];
        let expected_prices = vec![
            dec!(0.0203777),
            dec!(0.0203773),
            dec!(0.0203748),
            dec!(0.0203738),
            dec!(0.0203722),
            dec!(0.0203166),
            dec!(0.0202003),
        ];

        for (idx, sats) in volumes.into_iter().enumerate() {
            let mut price = converter.weighted_price_of_volume(*sats.amount());
            price.rescale(7);
            dbg!(price);

            assert_eq!(price, expected_prices[idx]);
        }

        Ok(())
    }

    #[test]
    fn cents_from_sats_volume() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("real")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids, true);
        let sats_volume = Sats::from_decimal(dec!(100_000_000));

        let cents = converter.cents_from_sats(sats_volume);

        assert_eq!(cents.floor(), UsdCents::from_major(2020029));

        Ok(())
    }

    #[test]
    fn sats_from_cents_volume() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("real")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids, true);
        let cents_volume = UsdCents::from_decimal(dec!(1000));

        let sats = converter.sats_from_cents(cents_volume);

        assert_eq!(sats.floor(), Sats::from_major(49075));

        Ok(())
    }
}
