use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BitfinexErrorResponse {
    pub error: String,
    pub code: u32,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct LastPriceData {
    pub bid: Decimal,
    pub bid_size: Decimal,
    pub ask: Decimal,
    pub ask_size: Decimal,
    pub daily_change: Decimal,
    pub daily_change_perc: Decimal,
    pub last_price: Decimal,
    pub volume: Decimal,
    pub high: Decimal,
    pub low: Decimal,
}

#[derive(Deserialize, Debug)]
pub struct FundingInfoData {
    pub key: String,
    pub symbol: String,
    pub funding: FundingData,
}

#[derive(Deserialize, Debug)]
pub struct FundingData {
    pub yield_loan: Decimal,
    pub yield_lend: Decimal,
    pub duration_loan: Decimal,
    pub duration_lend: Decimal,
}
