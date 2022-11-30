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
        UsdCents::from_decimal(*sats.amount() * self.volume_weighted_price_of_one_sat(sats))
    }

    pub fn sats_from_cents(&self, cents: UsdCents) -> Sats {
        if *cents.amount() == dec!(0) {
            return Sats::from_major(0);
        }

        Sats::from_decimal(*cents.amount() / self.volume_weighted_price_of_one_cent(cents))
    }

    fn volume_weighted_price_of_one_sat(&self, volume: Sats) -> Decimal {
        let mut price_acc = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;

        let pairs = self.pairs.clone();
        for (price, qty) in pairs {
            if (volume_acc + qty) < *volume.amount() {
                volume_acc += qty;
                price_acc += price.inner() * qty;
                continue;
            } else {
                let remaining_volume = volume.amount() - volume_acc;
                volume_acc += remaining_volume;
                price_acc += price.inner() * remaining_volume;
                break;
            }
        }

        price_acc / volume_acc
    }

    fn volume_weighted_price_of_one_cent(&self, volume: UsdCents) -> Decimal {
        let mut price_acc = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;

        let pairs = self.pairs.clone();
        for (price, qty) in pairs {
            if (volume_acc + qty) < *volume.amount() {
                volume_acc += qty;
                let price_of_one_cent = Decimal::ONE / price.inner();
                price_acc += price_of_one_cent * qty;
                continue;
            } else {
                let remaining_volume = volume.amount() - volume_acc;
                volume_acc += remaining_volume;
                let price_of_one_cent = Decimal::ONE / price.inner();
                price_acc += price_of_one_cent * remaining_volume;
                break;
            }
        }

        (Decimal::ONE / price_acc) * volume_acc
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
    fn sats_to_cents() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("contrived")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter());
        let volumes = vec![
            Sats::from_decimal(dec!(1)),
            Sats::from_decimal(dec!(10)),
            Sats::from_decimal(dec!(100)),
            Sats::from_decimal(dec!(1_000)),
            Sats::from_decimal(dec!(10_000)),
        ];

        let expected_cents = vec![
            UsdCents::from_decimal(dec!(0.10)),
            UsdCents::from_decimal(dec!(1.00)),
            UsdCents::from_decimal(dec!(19.00)),
            UsdCents::from_decimal(dec!(289.00)),
            UsdCents::from_decimal(dec!(2950.00)),
        ];

        for (idx, sats) in volumes.iter().enumerate() {
            let cents = converter.cents_from_sats(sats.clone());
            assert!(cents.amount() >= expected_cents[idx].amount());
        }

        Ok(())
    }

    #[test]
    fn cents_to_sats() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("contrived")?.payload;
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());
        let volumes = vec![
            UsdCents::from_major(1),
            UsdCents::from_major(10),
            UsdCents::from_major(100),
            UsdCents::from_major(1_000),
            UsdCents::from_major(10_000),
        ];

        let expected_sats = vec![
            Sats::from_decimal(dec!(4)),
            Sats::from_decimal(dec!(40)),
            Sats::from_decimal(dec!(400)),
            Sats::from_decimal(dec!(4100)),
            Sats::from_decimal(dec!(53625)),
        ];

        for (idx, cents) in volumes.iter().enumerate() {
            let sats = converter.sats_from_cents(cents.clone());
            assert_eq!(sats.amount().round(), *expected_sats[idx].amount());
        }

        Ok(())
    }
}
