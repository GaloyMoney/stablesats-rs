use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{AddressMethod, ClientId, Currency, MessageId, Wallet};

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

#[derive(Deserialize, Debug)]
pub struct OrderDetails {
    pub id: u64,
    pub group_id: Option<u64>,
    pub client_id: ClientId,
    pub symbol: String,
    pub creation_timestamp: u64,
    pub update_timestamp: u64,
    pub amount: Decimal,
    pub amount_original: Decimal,
    pub order_type: String,
    pub previous_order_type: Option<String>,
    pub time_in_force: u64,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub flags: Option<u64>,
    pub order_status: Option<String>,

    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_2: Option<String>,

    pub price: Decimal,
    pub price_avg: Decimal,
    pub price_trailing: Option<Decimal>,
    pub price_aux_limit: Option<Decimal>,

    #[serde(skip_serializing)]
    __placeholder_3: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_4: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_5: Option<String>,

    pub notify: bool,
    pub hidden: bool,
    pub placed_id: Option<u64>,

    #[serde(skip_serializing)]
    _placeholder_6: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_7: Option<String>,

    pub routing: String,

    #[serde(skip_serializing)]
    _placeholder_8: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_9: Option<String>,

    #[serde(skip)]
    pub meta: String,
    #[serde(skip)]
    pub complete: bool,
}

#[derive(Deserialize, Debug)]
pub struct WalletDetails {
    pub wallet_type: String,
    pub currency: Currency,
    pub balance: Decimal,
    pub unsettled_interest: Decimal,
    pub balance_available: Decimal,
    pub last_change: Option<String>,
    pub trade_details: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PositionDetails {
    pub symbol: String,
    pub status: String,
    pub amount: Decimal,
    pub base_price: Decimal,
    pub funding: Decimal,
    pub funding_type: bool,
    pub pl: Decimal,
    pub pl_perc: Decimal,
    pub price_liq: Decimal,
    pub leverage: Decimal,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub position_id: u64,
    pub mts_create: u64,
    pub mts_update: u64,

    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,

    pub position_type: u64,

    #[serde(skip_serializing)]
    _placeholder_2: Option<String>,

    pub collateral: Decimal,
    pub collateral_min: Decimal,

    #[serde(skip)]
    pub meta: String,
}

#[derive(Deserialize, Debug)]
pub struct DepositAddressDetails {
    pub mts: u64,
    pub address_type: String,
    pub message_id: Option<u64>,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub address: DepositAddress,

    pub code: Option<u64>,
    pub status: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct DepositAddress {
    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub method: AddressMethod,
    pub currency_code: Currency,

    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,

    pub address: String,
    pub pool_address: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TransferDetails {
    pub mts: u64,
    pub transfer_type: String,
    pub message_id: MessageId,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub address: Transfer,

    pub code: Option<u64>,
    pub status: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct Transfer {
    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub mts_update: String,
    pub wallet_from: Wallet,
    pub wallet_to: Wallet,

    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,

    pub currency: Currency,
    pub currency_to: Currency,

    #[serde(skip_serializing)]
    _placeholder_2: Option<String>,

    pub amount: Decimal,
}

#[derive(Deserialize, Debug)]
pub struct WithdrawDetails {
    pub mts: u64,
    pub withdraw_type: String,
    pub message_id: MessageId,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub address: Withdraw,

    pub code: Option<u64>,
    pub status: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct Withdraw {
    pub id: u64,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub method: AddressMethod,
    pub payment_id: String,
    pub wallet_from: Wallet,
    pub amount: Decimal,

    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_2: Option<String>,

    pub fee: Decimal,
}

#[derive(Deserialize, Debug)]
pub struct InvoiceDetails {
    pub invoice_hash: String,
    pub invoice: String,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_2: Option<String>,

    pub amount: Decimal,
}

#[derive(Deserialize, Debug)]
pub struct TransactionDetails {
    pub transactions: Vec<Transaction>,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub id: u64,
    pub currency: Currency,
    pub currency_name: String,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,

    pub mts_started: u64,
    pub mts_updated: u64,

    #[serde(skip_serializing)]
    _placeholder_2: Option<String>,

    #[serde(skip_serializing)]
    _placeholder_3: Option<String>,

    pub status: String,

    #[serde(skip_serializing)]
    _placeholder_4: Option<String>,

    #[serde(skip_serializing)]
    _placeholder_5: Option<String>,

    pub amount: Decimal,
    pub fees: Decimal,

    #[serde(skip_serializing)]
    _placeholder_6: Option<String>,

    #[serde(skip_serializing)]
    _placeholder_7: Option<String>,

    pub destination_address: String,

    #[serde(skip_serializing)]
    _placeholder_8: Option<String>,

    #[serde(skip_serializing)]
    _placeholder_9: Option<String>,

    #[serde(skip_serializing)]
    _placeholder_a: Option<String>,

    pub transaction_id: String,
    pub withdraw_transaction_note: String,
}

#[derive(Deserialize, Debug)]
pub struct SubmittedOrderDetails {
    pub mts: u64,
    pub order_type: String,
    pub message_id: MessageId,

    #[serde(skip_serializing)]
    _placeholder_0: Option<String>,

    pub orders: Vec<OrderDetails>,

    pub code: Option<u64>,
    pub status: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct ApiKeyDetails {
    pub scope: String,
    #[serde(deserialize_with = "boolean")]
    pub read: bool,
    #[serde(deserialize_with = "boolean")]
    pub write: bool,
}

fn boolean<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Bool(b) => b,
        serde_json::Value::String(s) => s == "yes",
        serde_json::Value::Number(num) => {
            num.as_i64()
                .ok_or_else(|| serde::de::Error::custom("Invalid number"))?
                != 0
        }
        serde_json::Value::Null => false,
        _ => return Err(serde::de::Error::custom("Wrong type, expected boolean")),
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn last_price_data() {
        let response_text =
            "[16808,24.10170847,16809,55.3107456,-26,-0.0015,16809,147.2349813,16884,16769]";
        let details = serde_json::from_str::<LastPriceData>(response_text).unwrap();
        assert_eq!(details.high, dec!(16884));
    }

    #[test]
    fn btc_on_chain_deposit_address_details() {
        let response_text = "[1672987082929,\"acc_dep\",null,null,[null,\"BITCOIN\",\"BTC\",null,\"address\",null],null,\"SUCCESS\",\"success\"]";
        let details = serde_json::from_str::<DepositAddressDetails>(response_text).unwrap();
        assert_eq!(details.address.address, "address");
    }

    #[test]
    fn ln_deposit_address_details() {
        let response_text = "[1672985819613,\"acc_dep\",null,null,[null,\"LNX\",\"LNX\",null,\"address\",null],null,\"SUCCESS\",\"success\"]";
        let details = serde_json::from_str::<DepositAddressDetails>(response_text).unwrap();
        assert_eq!(details.address.address, "address");
    }

    #[test]
    fn invoice_details() {
        let response_text = "[\"hash\",\"invoice\",null,null,\"0.001\"]";
        let details = serde_json::from_str::<InvoiceDetails>(response_text).unwrap();
        assert_eq!(details.invoice_hash, "hash");
        assert_eq!(details.invoice, "invoice");
        assert_eq!(details.amount, dec!(0.001));
    }

    #[test]
    fn wallet_details() {
        // let response_text = "[[\"exchange\",\"TESTBTC\",0.01,0,0.01,null,null],[\"exchange\",\"TESTUSD\",100,0,100,null,null],[\"exchange\",\"TESTUSDT\",200,0,200,null,null]]";
        let response_text = "[[\"exchange\",\"TESTBTC\",0.005,0,0.005,null,null],[\"exchange\",\"TESTUSD\",100,0,100,null,null],[\"exchange\",\"TESTUSDT\",200,0,200,null,null],[\"margin\",\"TESTBTC\",0.005,0,0.005,null,null]]";
        let _details = serde_json::from_str::<Vec<WalletDetails>>(response_text).unwrap();
    }
}
