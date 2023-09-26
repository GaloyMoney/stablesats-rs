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
        vec.sort_by(|a, b| b.0.cmp(&a.0));

        let mut cents = Decimal::ZERO;
        let mut volume_acc = Decimal::ZERO;
        for (quote_price, volume, total_sats_at_volume) in vec.iter() {
            if volume_acc + total_sats_at_volume <= *sats.amount() {
                cents += volume.inner();
                volume_acc += total_sats_at_volume;
            } else {
                let remaining_volume = *sats.amount() - volume_acc;
                cents += quote_price.inner() * remaining_volume;
                volume_acc += remaining_volume;
            }
        }

        // to account for when the order book depth is not enough to fill the volume
        if *sats.amount() > volume_acc {
            let remaining_volume = *sats.amount() - volume_acc;
            cents += vec[vec.len() - 1].0.inner() * remaining_volume;
        }

        UsdCents::from_decimal(cents.floor())
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
            let remaining_volume = *cents.amount() - volume_acc;
            sats += remaining_volume / vec[vec.len() - 1].0.inner();
        }

        Sats::from_decimal(sats.floor())
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
            "bids": {
                "1.1" : "147", 
                "0.9" : "259",
                "2.73" : "493"
            },
            "asks": {
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
    fn cents_from_sats_for_trivial_payload() -> anyhow::Result<()> {
        let latest_snapshot = get_trivial_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter());

        let cents = converter.cents_from_sats(Sats::from_decimal(dec!(20)));
        assert_eq!(cents, UsdCents::from_major(20));

        let cents = converter.cents_from_sats(Sats::from_decimal(dec!(550)));
        assert_eq!(cents, UsdCents::from_major(150));

        let cents = converter.cents_from_sats(Sats::from_decimal(dec!(999)));
        assert_eq!(cents, UsdCents::from_major(239));

        Ok(())
    }

    #[test]
    fn cents_from_sats_for_complex_payload() -> anyhow::Result<()> {
        let latest_snapshot = get_complex_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.bids.iter());

        let cents = converter.cents_from_sats(Sats::from_decimal(dec!(602)));
        assert_eq!(cents, UsdCents::from_decimal(dec!(898)));

        let cents = converter.cents_from_sats(Sats::from_decimal(dec!(650)));
        assert_eq!(cents, UsdCents::from_decimal(dec!(942)));

        Ok(())
    }

    #[test]
    fn sats_from_cents_for_trivial_payload() -> anyhow::Result<()> {
        let latest_snapshot = get_trivial_payload();
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter().rev());

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
        let converter = VolumeBasedPriceConverter::new(latest_snapshot.asks.iter().rev());

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(10)));
        assert_eq!(sats, Sats::from_major(111));

        let sats = converter.sats_from_cents(UsdCents::from_decimal(dec!(51_893)));
        assert_eq!(sats, Sats::from_major(275238));

        Ok(())
    }
}
