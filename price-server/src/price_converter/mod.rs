use crate::{
    currency::{Sats, UsdCents, VolumePicker},
    QuotePriceCentsForOneSat, VolumeInCents,
};
use rust_decimal::Decimal;

pub struct VolumeBasedPriceConverter<
    'a,
    I: Iterator<Item = (&'a QuotePriceCentsForOneSat, &'a VolumeInCents)> + Clone,
> {
    pairs: I,
}
impl<'a, I: Iterator<Item = (&'a QuotePriceCentsForOneSat, &'a VolumeInCents)> + Clone>
    VolumeBasedPriceConverter<'a, I>
{
    pub fn new(pairs: I) -> Self {
        Self { pairs }
    }

    fn weighted_price_of_volume(&self, total_volume: Decimal) -> Decimal {
        let mut price_acc = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;

        let pairs = self.pairs.clone();
        for (price, qty) in pairs {
            if (volume_acc + qty.inner()) < total_volume {
                volume_acc += qty.inner();
                price_acc += price.inner() * qty.inner();
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

impl<'a, I: Iterator<Item = (&'a QuotePriceCentsForOneSat, &'a VolumeInCents)> + Clone> VolumePicker
    for VolumeBasedPriceConverter<'a, I>
{
    fn cents_from_sats(&self, sats: Sats) -> UsdCents {
        UsdCents::from_decimal(*sats.amount() * self.weighted_price_of_volume(*sats.amount()))
    }

    fn sats_from_cents(&self, cents: UsdCents) -> Sats {
        let mut vec = self
            .pairs
            .clone()
            .map(|(quote_price, volume)| {
                (
                    quote_price.clone(),
                    volume.clone(),
                    volume.inner() / quote_price.inner(),
                )
            })
            .collect::<Vec<(QuotePriceCentsForOneSat, VolumeInCents, Decimal)>>();
        vec.sort_by(|a, b| a.0.cmp(&b.0));

        let mut sats = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;
        for (quote_price, volume, total_sats_at_volume) in vec.iter() {
            if volume_acc + volume.inner() <= *cents.amount() {
                volume_acc += volume.inner();
                sats += total_sats_at_volume;
            } else {
                let remaining_volume = *cents.amount() - volume_acc;
                volume_acc += remaining_volume;
                sats += remaining_volume / quote_price.inner();
                break;
            }
        }

        // to account for when the order book depth is not enough to fill the volume
        if *cents.amount() > volume_acc {
            sats += (*cents.amount() - volume_acc) / vec[vec.len() - 1].0.inner();
        }
        return Sats::from_decimal(sats.floor());
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use shared::payload::OrderBookPayload;

    use crate::OrderBookView;

    use super::*;

    fn get_trivial_payload() -> OrderBookView {
        let raw = r#"{
            "asks": {
                "0.1": "1000"
            },
            "bids": {
                "5.0" : "1000",
                "0.1" : "100" 
            },
            "timestamp": 1667454784,
            "exchange": "okex"
            }"#;
        let price_message_payload =
            serde_json::from_str::<OrderBookPayload>(raw).expect("Could not parse payload");
        price_message_payload.into()
    }

    fn get_complex_payload() -> OrderBookView {
        let raw = r#"{
            "asks": {
                "0.1": "1000"
            },
            "bids": {
                "1.08" : "18541",
                "0.1" : "18849",
                "2.02" : "1907",
                "0.09" : "5878"
            },
            "timestamp": 1667454784,
            "exchange": "okex"
            }"#;
        let price_message_payload =
            serde_json::from_str::<OrderBookPayload>(raw).expect("Could not parse payload");
        price_message_payload.into()
    }

    #[test]
    fn cents_from_sats_volume() -> anyhow::Result<()> {
        // start from max
        let latest_snapshot = get_trivial_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());
        let sats_volume = Sats::from_decimal(dec!(100_000_000));

        let cents = converter.cents_from_sats(sats_volume);

        assert_eq!(cents.floor(), UsdCents::from_major(65));

        Ok(())
    }

    #[test]
    fn sats_from_cents_for_trivial_payload() -> anyhow::Result<()> {
        let latest_snapshot = get_trivial_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(10)));
        assert_eq!(sats, Sats::from_major(100));

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(149)));
        assert_eq!(sats, Sats::from_major(1009));

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(1500)));
        assert_eq!(sats, Sats::from_major(1280));

        Ok(())
    }

    #[test]
    fn sats_from_cents_for_complex_payload() -> anyhow::Result<()> {
        let latest_snapshot = get_complex_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(10)));
        assert_eq!(sats, Sats::from_major(111));

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(51_893)));
        assert_eq!(sats, Sats::from_major(275238));

        Ok(())
    }
}
