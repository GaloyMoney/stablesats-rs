use chrono::Utc;
use rust_decimal::Decimal;
use std::fmt::Display;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ClientId(pub(super) i64);
impl ClientId {
    pub fn new() -> Self {
        Self(Utc::now().timestamp_millis())
    }
}
impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(s.parse::<i64>().unwrap())
    }
}
impl From<ClientId> for i64 {
    fn from(id: ClientId) -> Self {
        id.0
    }
}
impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct MessageId(pub(super) i64);

#[derive(Debug, Clone)]
pub enum Instrument {
    TestBtcUsdSwap,
    BtcUsdSwap,
    UsdSpot,
}

impl Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Instrument::TestBtcUsdSwap => write!(f, "tTESTBTCF0:TESTUSDTF0"),
            Instrument::BtcUsdSwap => write!(f, "tBTCF0:USTF0"),
            Instrument::UsdSpot => write!(f, "fUSD"),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub enum Wallet {
    EXCHANGE,
    MARGIN,
    FUNDING,
    TRADING,
    DEPOSIT,
}

impl Display for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Wallet::EXCHANGE => write!(f, "exchange"),
            Wallet::MARGIN => write!(f, "margin"),
            Wallet::FUNDING => write!(f, "funding"),
            Wallet::TRADING => write!(f, "trading"),
            Wallet::DEPOSIT => write!(f, "deposit"),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub enum Currency {
    BTC,
    LNX,
    USD,
    UST,
    USTF0,
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Currency::BTC => write!(f, "BTC"),
            Currency::LNX => write!(f, "LNX"),
            Currency::USD => write!(f, "USD"),
            Currency::UST => write!(f, "UST"),
            Currency::USTF0 => write!(f, "USTF0"),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub enum AddressMethod {
    BITCOIN,
    LNX,
}

impl Display for AddressMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AddressMethod::BITCOIN => write!(f, "bitcoin"),
            AddressMethod::LNX => write!(f, "lnx"),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub enum OrderType {
    LIMIT,
    MARKET,
}

impl Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OrderType::LIMIT => write!(f, "limit"),
            OrderType::MARKET => write!(f, "market"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_order_id() {
        let ts = Utc::now().timestamp_millis();
        let id = ClientId::new();
        assert!(id.0 >= ts);
    }
}
