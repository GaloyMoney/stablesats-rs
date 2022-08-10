use shared::currency::{Sats, UsdCents};


pub struct SatCentConverter {
    pub price_of_sat: UsdCents
}

impl SatCentConverter {

    pub fn convert(&self, sat_amount: Sats) -> UsdCents {

        let result = sat_amount.amount() * self.price_of_sat.amount();

        UsdCents::from_decimal(result)
    }
}


#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::exchange_price_cache::ExchangePriceCache;
    use super::*;

    #[test]
    fn test_convert_sat_to_cents() {
        let sats = Sats::from_major(1000);

        let price_cache = ExchangePriceCache::new(Duration::seconds(30));

        //initialise btc and get the latest tick

    }
}
