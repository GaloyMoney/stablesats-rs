use rust_decimal::Decimal;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub struct DepositAddress {
    pub value: String,
}

#[derive(Debug)]
pub struct TransferId {
    pub value: String,
}

#[derive(Debug)]
pub struct AvailableBalance {
    pub amt_in_btc: Decimal,
}

#[derive(Debug)]
pub struct TransferState {
    pub value: String,
}

#[derive(Debug)]
pub struct WithdrawId {
    pub value: String,
}

#[derive(Debug)]
pub struct DepositStatus {
    pub status: String,
}

#[derive(Debug)]
pub struct OrderId {
    pub value: String,
}

#[derive(Debug)]
pub struct PositionId {
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum OkexInstrumentId {
    BtcUsdSwap,
    BtcUsd,
}

impl Display for OkexInstrumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OkexInstrumentId::BtcUsd => write!(f, "BTC-USD"),
            OkexInstrumentId::BtcUsdSwap => write!(f, "BTC-USD-SWAP"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OkexMarginMode {
    Cross,
    Isolated,
}

impl Display for OkexMarginMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OkexMarginMode::Cross => write!(f, "cross"),
            OkexMarginMode::Isolated => write!(f, "isolated"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OkexPositionMode {
    LongShort,
    Net,
}

impl Display for OkexPositionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OkexPositionMode::LongShort => write!(f, "long_short_mode"),
            OkexPositionMode::Net => write!(f, "net_mode"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OkexPositionSide {
    Long,
    Short,
    Net,
}

impl Display for OkexPositionSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OkexPositionSide::Net => write!(f, "net"),
            OkexPositionSide::Long => write!(f, "long"),
            OkexPositionSide::Short => write!(f, "short"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OkexOrderSide {
    Buy,
    Sell,
}

impl Display for OkexOrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OkexOrderSide::Buy => write!(f, "buy"),
            OkexOrderSide::Sell => write!(f, "sell"),
        }
    }
}

pub enum OkexOrderType {
    Market,
    Limit,
    PostOnly,
    Fok,
    Ioc,
    OptimalLimitIoc,
}

impl Display for OkexOrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OkexOrderType::Market => write!(f, "market"),
            OkexOrderType::Limit => write!(f, "limit"),
            OkexOrderType::PostOnly => write!(f, "post_only"),
            OkexOrderType::Fok => write!(f, "fok"),
            OkexOrderType::Ioc => write!(f, "ioc"),
            OkexOrderType::OptimalLimitIoc => write!(f, "optimal_limit_ioc"),
        }
    }
}

pub struct OkexClientConfig {
    pub api_key: String,
    pub passphrase: String,
    pub secret_key: String,
    pub simulated: bool,
    pub position_mode: OkexPositionMode,
}
