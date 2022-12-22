use rust_decimal::Decimal;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum BitfinexInstrumentId {
    BtcUsdSwap,
    UsdSpot,
}

impl Display for BitfinexInstrumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BitfinexInstrumentId::BtcUsdSwap => write!(f, "tBTCUSD"),
            BitfinexInstrumentId::UsdSpot => write!(f, "fUSD"),
        }
    }
}

#[derive(Debug)]
pub struct LastPrice {
    pub usd_cents: Decimal,
}

#[derive(Debug)]
pub struct FundingInfo {
    pub key: String,
    pub symbol: String,
    pub yield_loan: Decimal,
    pub yield_lend: Decimal,
    pub duration_loan: Decimal,
    pub duration_lend: Decimal,
}
