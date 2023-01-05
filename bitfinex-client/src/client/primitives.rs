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
            BitfinexInstrumentId::BtcUsdSwap => write!(f, "tBTCF0:USTF0"),
            BitfinexInstrumentId::UsdSpot => write!(f, "fUSD"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LastPrice {
    pub usd_cents: Decimal,
}

#[derive(Debug, Clone)]
pub struct FundingInfo {
    pub key: String,
    pub symbol: String,
    pub yield_loan: Decimal,
    pub yield_lend: Decimal,
    pub duration_loan: Decimal,
    pub duration_lend: Decimal,
}
