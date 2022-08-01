use shared::exchange::*;
use shared::money::*;
use shared::time::*;

pub struct ExchangePriceCache {
    numerator_unit: CurrencyRaw,
    denominator_unit: CurrencyRaw,
    last_update: Option<TimeStamp>,
    current_bid_price: Option<Money>,
    current_ask_price: Option<Money>,
}

impl ExchangePriceCache {
    pub fn new(numerator_unit: CurrencyRaw, denominator_unit: CurrencyRaw) -> Self {
        Self {
            numerator_unit,
            denominator_unit,
            last_update: None,
            current_bid_price: None,
            current_ask_price: None,
        }
    }

    pub fn update_price(&mut self, timestamp: TimeStamp, bid_price: Money, ask_price: Money) {
        self.last_update = Some(timestamp);
        self.current_bid_price = Some(bid_price);
        self.current_ask_price = Some(ask_price);
    }
}
