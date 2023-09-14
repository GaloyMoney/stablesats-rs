use rust_decimal::Decimal;

use crate::{error::*, quote::*};

pub struct QuotesApp {}

impl QuotesApp {
    pub async fn run() -> Result<(), QuotesAppError> {
        Ok(())
    }

    pub async fn quote_buy_cents_from_sats(
        &self,
        sats: Decimal,
        execute: bool,
    ) -> Result<Quote, QuotesAppError> {
        let quote = NewQuote::builder()
            .direction(Direction::BuyCents)
            .immediate_execution(execute)
            .build()
            .expect("Could not build quote");
        unimplemented!()
    }
}
