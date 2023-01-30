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

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Instrument {
    BtcUsdSwap,
}

impl Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Instrument::BtcUsdSwap => write!(f, "BTC-PERPETUAL"),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Currency {
    BTC,
    USDC,
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Currency::BTC => write!(f, "BTC"),
            Currency::USDC => write!(f, "USDC"),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    Insane,
    ExtremeHigh,
    VeryHigh,
    High,
    Mid,
    Low,
    VeryLow,
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Priority::Insane => write!(f, "insane"),
            Priority::ExtremeHigh => write!(f, "extreme_high"),
            Priority::VeryHigh => write!(f, "very_high"),
            Priority::High => write!(f, "high"),
            Priority::Mid => write!(f, "mid"),
            Priority::Low => write!(f, "low"),
            Priority::VeryLow => write!(f, "very_low"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LastPrice {
    pub usd_cents: Decimal,
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
