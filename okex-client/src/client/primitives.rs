use rust_decimal::Decimal;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BtcUsdSwapContracts(pub(super) u32);
impl From<u32> for BtcUsdSwapContracts {
    fn from(contracts: u32) -> Self {
        Self(contracts)
    }
}
impl From<&BtcUsdSwapContracts> for u32 {
    fn from(contracts: &BtcUsdSwapContracts) -> Self {
        contracts.0
    }
}
impl std::fmt::Display for BtcUsdSwapContracts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
pub struct PositionSize {
    pub instrument_id: OkexInstrumentId,
    pub usd_cents: Decimal,
}

#[derive(Debug, Clone)]
pub enum OkexInstrumentId {
    BtcUsdSwap,
}

impl Display for OkexInstrumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OkexInstrumentId::BtcUsdSwap => write!(f, "BTC-USD-SWAP"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OkexMarginMode {
    Cross,
}

impl Display for OkexMarginMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OkexMarginMode::Cross => write!(f, "cross"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OkexPositionMode {
    Net,
}

impl Display for OkexPositionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
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

#[derive(Debug, Clone)]
pub enum TradeCurrency {
    BTC,
    USD,
}

impl Display for TradeCurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TradeCurrency::BTC => write!(f, "BTC"),
            TradeCurrency::USD => write!(f, "USD"),
        }
    }
}
