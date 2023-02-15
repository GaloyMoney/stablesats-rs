use rust_decimal::Decimal;
use std::{fmt::Display, str::FromStr};

use crate::DeribitClientError;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ClientId(pub(super) String);
impl ClientId {
    pub fn new() -> Self {
        use rand::distributions::{Alphanumeric, DistString};
        Self(Alphanumeric.sample_string(&mut rand::thread_rng(), 32))
    }
}
impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<ClientId> for String {
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
pub enum KeyUsage {
    ForFunding,
    ForTrading,
}

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

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum OrderState {
    Open,
    Filled,
    Rejected,
    Cancelled,
    Untriggered,
}

impl Display for OrderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OrderState::Open => write!(f, "open"),
            OrderState::Filled => write!(f, "filled"),
            OrderState::Rejected => write!(f, "rejected"),
            OrderState::Cancelled => write!(f, "cancelled"),
            OrderState::Untriggered => write!(f, "untriggered"),
        }
    }
}

impl FromStr for OrderState {
    type Err = DeribitClientError;

    fn from_str(input: &str) -> Result<OrderState, Self::Err> {
        match input {
            "open" => Ok(OrderState::Open),
            "filled" => Ok(OrderState::Filled),
            "rejected" => Ok(OrderState::Rejected),
            "cancelled" => Ok(OrderState::Cancelled),
            "untriggered" => Ok(OrderState::Untriggered),
            _ => Err(DeribitClientError::CannotConvertOrderStateFromStr),
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
        let id = ClientId::new();
        assert_eq!(id.0.len(), 32);
    }
}
