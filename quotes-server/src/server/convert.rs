use rust_decimal::prelude::ToPrimitive;

use crate::{
    error::QuotesAppError,
    proto::{GetQuoteToBuyUsdResponse, GetQuoteToSellUsdResponse},
    quote::Quote,
};

impl From<QuotesAppError> for tonic::Status {
    fn from(_err: QuotesAppError) -> Self {
        tonic::Status::new(tonic::Code::Unknown, "Unknown error")
    }
}

impl From<Quote> for GetQuoteToBuyUsdResponse {
    fn from(quote: Quote) -> Self {
        Self {
            quote_id: quote.id.to_string(),
            amount_to_sell_in_sats: quote
                .sat_amount
                .amount()
                .to_u64()
                .expect("sat_amount should always parse to u64"),
            amount_to_buy_in_cents: quote
                .cent_amount
                .amount()
                .to_u64()
                .expect("cent_amount should always parse to u64"),
            expires_at: quote
                .expires_at
                .timestamp()
                .to_u32()
                .expect("timestamp should always parse to u32"),
            executed: false, // hardcoded for now
        }
    }
}

impl From<Quote> for GetQuoteToSellUsdResponse {
    fn from(quote: Quote) -> Self {
        Self {
            quote_id: quote.id.to_string(),
            amount_to_buy_in_sats: quote
                .sat_amount
                .amount()
                .to_u64()
                .expect("sat_amount should always parse to u64"),
            amount_to_sell_in_cents: quote
                .cent_amount
                .amount()
                .to_u64()
                .expect("cent_amount should always parse to u64"),
            expires_at: quote
                .expires_at
                .timestamp()
                .to_u32()
                .expect("timestamp should always parse to u32"),
            executed: false, // hardcoded for now
        }
    }
}
