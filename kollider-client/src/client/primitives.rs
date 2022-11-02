use std::fmt::Debug;
use std::fmt::Display;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UserBalances {
    pub cash: Cash,
    pub cross_margin: String,
    pub isolated_margin: IsolatedMargin,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Cash {
    pub kkp: String,
    pub sat: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct IsolatedMargin {}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub payment_request: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub enum KolliderOrderSide {
    Bid,
    Ask,
}

impl Display for KolliderOrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KolliderOrderSide::Bid => write!(f, "Bid"),
            KolliderOrderSide::Ask => write!(f, "Ask"),
        }
    }
}
