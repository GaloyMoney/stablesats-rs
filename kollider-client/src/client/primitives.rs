use std::fmt::Debug;
use std::fmt::Display;

use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct KolliderErrorResponse {
    pub error: String,
    #[serde(rename = "msg")]
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UserBalances {
    pub cash: Cash,
    pub cross_margin: String,
    pub isolated_margin: IsolatedMargin,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Cash {
    pub kkp: String,
    pub sat: String,
}

#[derive(Debug, Deserialize)]
pub struct IsolatedMargin {
    #[serde(rename = "BTCUSD.PERP")]
    pub btc_usd: String,
}

#[derive(Debug, Deserialize)]
pub struct PaymentRequest {
    pub payment_request: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaceOrderResult {
    pub timestamp: i64,
    pub order_id: i64,
    pub ext_order_id: String,
    pub uid: i64,
    pub symbol: String,
    pub quantity: i64,
    pub order_type: String,
    pub price: i64,
    pub leverage: i64,
}

#[derive(Debug, Deserialize)]
pub struct Products {
    #[serde(rename = "BTCUSD.PERP")]
    pub btcusd_perp: Product,
    #[serde(rename = "BTCEUR.PERP")]
    pub btceur_perp: Product,
}

#[derive(Debug, Deserialize)]
pub struct Product {
    pub symbol: String,
    pub contract_size: String,
    pub max_leverage: String,
    pub base_margin: String,
    pub is_inverse_priced: bool,
    pub price_dp: i64,
    pub underlying_symbol: String,
    pub last_price: String,
    pub tick_size: String,
    pub risk_limit: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenPositions {
    #[serde(rename = "BTCUSD.PERP")]
    pub btcusd_perp: Option<BtcusdPerp>,
}

#[derive(Debug, Deserialize)]
pub struct BtcusdPerp {
    pub uid: i64,
    pub timestamp: i64,
    pub symbol: String,
    pub upnl: String,
    pub leverage: String,
    pub entry_price: String,
    pub side: String,
    pub quantity: String,
    pub liq_price: String,
    pub open_order_ids: Vec<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Eq)]
pub struct OpenOrders {
    #[serde(rename = "BTCUSD.PERP")]
    pub btcusd_perp: Vec<OpenOrderBtcusdPerp>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct OpenOrderBtcusdPerp {
    pub order_id: i64,
    pub uid: i64,
    pub price: i64,
    pub timestamp: i64,
    pub filled: i64,
    pub ext_order_id: String,
    pub order_type: String,
    pub advanced_order_type: Option<String>,
    pub trigger_price_type: Option<String>,
    pub side: String,
    pub quantity: i64,
    pub symbol: String,
    pub leverage: i64,
    pub margin_type: String,
    pub settlement_type: String,
    pub origin: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum KolliderOrderType {
    Market,
    Limit,
}
impl Display for KolliderOrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KolliderOrderType::Market => write!(f, "Market"),
            KolliderOrderType::Limit => write!(f, "Limit"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum KolliderInstrumentId {
    BtcUsdSwap,
}

impl Display for KolliderInstrumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KolliderInstrumentId::BtcUsdSwap => write!(f, "BTCUSD.PERP"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum KolliderOrderSide {
    Buy,
    Sell,
}

impl Display for KolliderOrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KolliderOrderSide::Buy => write!(f, "Bid"),
            KolliderOrderSide::Sell => write!(f, "Ask"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum KolliderMarginType {
    Cross,
    Isolated,
}

impl Display for KolliderMarginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KolliderMarginType::Cross => write!(f, "Cross"),
            KolliderMarginType::Isolated => write!(f, "Isolated"),
        }
    }
}
