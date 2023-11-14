use rust_decimal::prelude::ToPrimitive;

use crate::{
    error::QuotesAppError,
    proto::{GetQuoteToBuyUsdResponse, GetQuoteToSellUsdResponse},
    quote::Quote,
    QuotesServerError,
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
            executed: quote.is_accepted(),
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
            executed: quote.is_accepted(),
        }
    }
}

impl From<QuotesServerError> for tonic::Status {
    fn from(err: QuotesServerError) -> Self {
        match err {
            QuotesServerError::CouldNotParseIncomingUuid(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}
