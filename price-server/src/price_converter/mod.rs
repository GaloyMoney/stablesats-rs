use crate::{
    currency::{Sats, UsdCents, VolumePicker},
    QuotePrice,
};
use rust_decimal::Decimal;

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

impl<'a, I: Iterator<Item = (&'a QuotePrice, &'a Decimal)> + Clone> VolumePicker
    for VolumeBasedPriceConverter<'a, I>
{
    fn cents_from_sats(&self, sats: Sats) -> UsdCents {
        UsdCents::from_decimal(*sats.amount() * self.weighted_price_of_volume(*sats.amount()))
    }

    fn sats_from_cents(&self, cents: UsdCents) -> Sats {
        Sats::from_decimal(*cents.amount() / self.weighted_price_of_volume(*cents.amount()))
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use shared::payload::OrderBookPayload;

    use crate::OrderBookView;

    use super::*;

    fn get_payload() -> OrderBookView {
        let raw = r#"{
            "asks": {
                "0.1": "10",
                "0.2": "90",
                "0.3": "1000"
            },
            "bids": {
                "0.1": "500",
                "0.15": "90",
                "0.2": "10"
            },
            "timestamp": 1667454784,
            "exchange": "okex"
            }"#;
        let price_message_payload =
            serde_json::from_str::<OrderBookPayload>(raw).expect("Could not parse payload");
        price_message_payload.into()
    }
    #[test]
    fn weighted_average_ask_price() -> anyhow::Result<()> {
        let latest_snapshot = get_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter());
        let volumes = [
            Sats::from_decimal(dec!(1)),
            Sats::from_decimal(dec!(10)),
            Sats::from_decimal(dec!(20)),
            Sats::from_decimal(dec!(100)),
            Sats::from_decimal(dec!(110)),
        ];
        let expected_prices = [dec!(0.1), dec!(0.1), dec!(0.15), dec!(0.19), dec!(0.2)];

        for (idx, sats) in volumes.into_iter().enumerate() {
            let mut price = converter.weighted_price_of_volume(*sats.amount());
            price.rescale(7);

            assert_eq!(price, expected_prices[idx]);
        }

        Ok(())
    }

    #[test]
    fn weighted_average_bid_price() -> anyhow::Result<()> {
        let latest_snapshot = get_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());
        let volumes = [
            Sats::from_decimal(dec!(1)),
            Sats::from_decimal(dec!(10)),
            Sats::from_decimal(dec!(20)),
            Sats::from_decimal(dec!(100)),
            Sats::from_decimal(dec!(200)),
        ];
        let expected_prices = [dec!(0.2), dec!(0.2), dec!(0.175), dec!(0.155), dec!(0.1275)];

        for (idx, sats) in volumes.into_iter().enumerate() {
            let mut price = converter.weighted_price_of_volume(*sats.amount());
            price.rescale(7);

            assert_eq!(price, expected_prices[idx]);
        }

        Ok(())
    }

    #[test]
    fn cents_from_sats_volume() -> anyhow::Result<()> {
        let latest_snapshot = get_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());
        let sats_volume = Sats::from_decimal(dec!(100_000_000));

        let cents = converter.cents_from_sats(sats_volume);

        assert_eq!(cents.floor(), UsdCents::from_major(65));

        Ok(())
    }

    #[test]
    fn sats_from_cents_volume() -> anyhow::Result<()> {
        let latest_snapshot = get_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());
        let cents_volume = UsdCents::from_decimal(dec!(10));

        let sats = converter.sats_from_cents(cents_volume);

        assert_eq!(sats.floor(), Sats::from_major(50));

        Ok(())
    }
}
