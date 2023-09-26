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
}

impl<'a, I: Iterator<Item = (&'a QuotePriceCentsForOneSat, &'a VolumeInCents)> + Clone> VolumePicker
    for VolumeBasedPriceConverter<'a, I>
{
    fn cents_from_sats(&self, sats: Sats) -> UsdCents {
        let pairs = self.pairs.clone();
        let mut cents = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;
        let mut last_quote_price = None;
        for (quote_price, volume) in pairs {
            let total_sats_at_volume = volume.inner() / quote_price.inner();
            if volume_acc + total_sats_at_volume <= *sats.amount() {
                cents += volume.inner();
                volume_acc += total_sats_at_volume;
                last_quote_price = Some(quote_price);
            } else {
                let remaining_volume = *sats.amount() - volume_acc;
                cents += quote_price.inner() * remaining_volume;
                volume_acc += remaining_volume;
                break;
            }
        }

        // to account for when the order book depth is not enough to fill the volume
        if *sats.amount() > volume_acc {
            let remaining_volume = *sats.amount() - volume_acc;
            if let Some(quote_price) = last_quote_price {
                cents += quote_price.inner() * remaining_volume
            }
        }

        UsdCents::from_decimal(cents)
    }

    fn sats_from_cents(&self, cents: UsdCents) -> Sats {
        let pairs = self.pairs.clone();
        let mut sats = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;
        let mut last_quote_price = None;
        for (quote_price, volume) in pairs {
            let total_sats_at_volume = volume.inner() / quote_price.inner();
            if volume_acc + volume.inner() <= *cents.amount() {
                sats += total_sats_at_volume;
                volume_acc += volume.inner();
                last_quote_price = Some(quote_price);
            } else {
                let remaining_volume = *cents.amount() - volume_acc;
                sats += remaining_volume / quote_price.inner();
                volume_acc += remaining_volume;
                break;
            }
        }

        // to account for when the order book depth is not enough to fill the volume
        if *cents.amount() > volume_acc {
            let remaining_volume = *cents.amount() - volume_acc;
            if let Some(quote_price) = last_quote_price {
                sats += remaining_volume / quote_price.inner();
            }
        }

        Sats::from_decimal(sats)
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
            "bids": {
                "0.2" : "100",
                "1.0" : "50"  
            },
            "asks": {
                "0.1" : "100",
                "5.0" : "1000"
            },
            "timestamp": 1667454784,
            "exchange": "okex"
            }"#;
        let price_message_payload =
            serde_json::from_str::<OrderBookPayload>(raw).expect("Could not parse payload");
        price_message_payload.into()
    }

    #[test]
    fn cents_from_sats_for_trivial_payload() -> anyhow::Result<()> {
        let latest_snapshot = get_trivial_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter().rev());

        let cents = converter.cents_from_sats(Sats::from_decimal(dec!(20)));
        assert_eq!(cents, UsdCents::from_major(20));

        let cents = converter.cents_from_sats(Sats::from_decimal(dec!(550)));
        assert_eq!(cents, UsdCents::from_major(150));

        let cents = converter.cents_from_sats(Sats::from_decimal(dec!(1000)));
        assert_eq!(cents, UsdCents::from_major(240));

        Ok(())
    }

    #[test]
    fn sats_from_cents_for_trivial_payload() -> anyhow::Result<()> {
        let latest_snapshot = get_trivial_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter());

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(10)));
        assert_eq!(sats, Sats::from_major(100));

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(150)));
        assert_eq!(sats, Sats::from_major(1010));

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(1500)));
        assert_eq!(sats, Sats::from_major(1280));

        Ok(())
    }
}
