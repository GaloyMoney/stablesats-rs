use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeribitResponse {
    pub jsonrpc: String,
    pub result: Option<LastPriceData>,

    pub us_in: Option<u64>,
    pub us_out: Option<u64>,
    pub us_diff: Option<u64>,
    pub testnet: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ErrorData {
    pub message: String,
    pub data: Option<HashMap<String, Value>>,
    pub code: i64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeribitErrorResponse {
    pub jsonrpc: String,
    pub error: ErrorData,

    pub us_in: u64,
    pub us_out: u64,
    pub us_diff: u64,
    pub testnet: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LastPriceDataStats {
    pub volume_usd: Decimal,
    pub volume: Decimal,
    pub price_change: Decimal,
    pub low: Decimal,
    pub high: Decimal,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LastPriceData {
    pub best_ask_amount: Decimal,
    pub best_ask_price: Decimal,
    pub best_bid_amount: Decimal,
    pub best_bid_price: Decimal,
    pub current_funding: Decimal,
    pub estimated_delivery_price: Decimal,
    pub funding_8h: Decimal,
    pub index_price: Decimal,
    pub instrument_name: String,
    pub interest_value: Decimal,
    pub last_price: Decimal,
    pub mark_price: Decimal,
    pub max_price: Decimal,
    pub min_price: Decimal,
    pub open_interest: Decimal,
    pub settlement_price: Decimal,
    pub state: String,
    pub stats: LastPriceDataStats,
    pub timestamp: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LastPriceDataDetails {
    pub jsonrpc: String,
    pub result: LastPriceData,

    // #[serde(rename = "camelCase")]
    // pub us_in: Option<u64>,
    // pub us_out: Option<u64>,
    // pub us_diff: Option<u64>,
    pub testnet: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DepositAddressData {
    pub address: String,
    pub creation_timestamp: u64,
    pub currency: String,
    #[serde(alias = "type")]
    pub address_type: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressDetails {
    pub jsonrpc: String,
    pub result: Option<DepositAddressData>,

    pub us_in: Option<u64>,
    pub us_out: Option<u64>,
    pub us_diff: Option<u64>,
    pub testnet: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Deposit {
    pub address: String,
    pub amount: Decimal,
    pub currency: String,
    pub state: String,
    pub transaction_id: String,
    pub received_timestamp: u64,
    pub updated_timestamp: u64,
    pub note: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DepositData {
    pub count: u32,
    pub data: Vec<Deposit>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositDetails {
    pub jsonrpc: String,
    pub result: DepositData,

    pub us_in: Option<u64>,
    pub us_out: Option<u64>,
    pub us_diff: Option<u64>,
    pub testnet: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Transfer {
    pub id: u64,
    pub amount: Decimal,
    pub currency: String,
    pub direction: String,
    pub state: String,
    #[serde(alias = "type")]
    pub transfer_type: String,
    pub other_side: String,
    pub created_timestamp: u64,
    pub updated_timestamp: u64,
    pub note: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransferData {
    pub count: u32,
    pub data: Vec<Transfer>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferDetails {
    pub jsonrpc: String,
    pub result: TransferData,

    pub us_in: Option<u64>,
    pub us_out: Option<u64>,
    pub us_diff: Option<u64>,
    pub testnet: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Withdrawal {
    pub id: u64,
    pub amount: Decimal,
    pub fee: Decimal,
    pub currency: String,
    pub address: String,
    pub priority: Decimal,
    pub state: String,
    pub transaction_id: String,
    pub created_timestamp: u64,
    pub confirmed_timestamp: u64,
    pub updated_timestamp: u64,
    pub note: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WithdrawalData {
    pub count: u32,
    pub data: Vec<Withdrawal>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalDetails {
    pub jsonrpc: String,
    pub result: WithdrawalData,

    pub us_in: Option<u64>,
    pub us_out: Option<u64>,
    pub us_diff: Option<u64>,
    pub testnet: Option<bool>,
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::Currency;

    use super::*;

    #[test]
    fn last_price_data() {
        let response_text =
            "{\"jsonrpc\":\"2.0\",\"result\":{\"timestamp\":1674461497839,\"stats\":{\"volume_usd\":359696880.0,\"volume\":15819.90886478,\"price_change\":-0.5074,\"low\":22133.0,\"high\":23331.0},\"state\":\"open\",\"settlement_price\":22700.49,\"open_interest\":4126543180,\"min_price\":22407.0,\"max_price\":23089.45,\"mark_price\":22746.89,\"last_price\":22745.0,\"interest_value\":0.0000648967073158397,\"instrument_name\":\"BTC-PERPETUAL\",\"index_price\":22740.69,\"funding_8h\":-0.00004213,\"estimated_delivery_price\":22740.69,\"current_funding\":0.0,\"best_bid_price\":22745.0,\"best_bid_amount\":42880.0,\"best_ask_price\":22745.5,\"best_ask_amount\":62660.0},\"usIn\":1674461498693343,\"usOut\":1674461498693470,\"usDiff\":127,\"testnet\":true}";
        let details = serde_json::from_str::<LastPriceDataDetails>(response_text).unwrap();
        assert_eq!(details.result.last_price, dec!(22745.0));
    }

    #[test]
    fn btc_on_chain_deposit_address_details_empty() {
        let response_text = "{\"jsonrpc\": \"2.0\", \"result\": null, \"usIn\": 1674807312116504, \"usOut\": 1674807312116808, \"usDiff\": 304, \"testnet\": true}";
        let details = serde_json::from_str::<DepositAddressDetails>(response_text).unwrap();
        assert!(details.result.is_none())
    }

    #[test]
    fn btc_on_chain_deposit_address_details() {
        let response_text = "{\"jsonrpc\": \"2.0\", \"id\": 3461, \"usIn\": 1674807312116504, \"usOut\": 1674807312116808, \"usDiff\": 304, \"result\": {\"address\": \"2N8udZGBc1hLRCFsU9kGwMPpmYUwMFTuCwB\", \"creation_timestamp\": 1550575165170, \"currency\": \"BTC\", \"type\": \"deposit\"}, \"testnet\": true}";
        let details = serde_json::from_str::<DepositAddressDetails>(response_text).unwrap();
        if let Some(data) = details.result {
            assert_eq!(data.address, "2N8udZGBc1hLRCFsU9kGwMPpmYUwMFTuCwB");
        } else {
            panic!()
        }
    }

    #[test]
    fn get_deposits_empty() {
        let response_text = "{\"jsonrpc\": \"2.0\",\"result\": {\"data\": [],\"count\": 0},\"usIn\": 1675065077579453,\"usOut\": 1675065077579739,\"usDiff\": 286,\"testnet\": true}";
        let details = serde_json::from_str::<DepositDetails>(response_text).unwrap();
        assert_eq!(details.result.count, 0);
        assert!(details.result.data.is_empty());
    }

    #[test]
    fn get_deposits() {
        let response_text = "{\"jsonrpc\": \"2.0\",\"result\": {\"data\": [{\"updated_timestamp\": 1675062738879,\"transaction_id\": \"878b71d6b5f2221bb6e52090c55f27dc330d838efc096731d481b86091358e51\",\"state\": \"completed\",\"received_timestamp\": 1675062700792,\"note\": \"\",\"currency\": \"BTC\",\"amount\": 0.9,\"address\": \"bcrt1qxqrpgcsneyt6zwgujakphyuqupl5g75rzlthxa\"}],\"count\": 1},\"usIn\": 1675063983490363,\"usOut\": 1675063983490583,\"usDiff\": 220,\"testnet\": true}";
        let details = serde_json::from_str::<DepositDetails>(response_text).unwrap();
        assert!(!details.result.data.is_empty());
        assert_eq!(
            details.result.data[0].address,
            "bcrt1qxqrpgcsneyt6zwgujakphyuqupl5g75rzlthxa"
        );
    }

    #[test]
    fn get_transfers_empty() {
        let response_text = "{\"jsonrpc\": \"2.0\",\"result\": {\"data\": [],\"count\": 0},\"usIn\": 1675065077579453,\"usOut\": 1675065077579739,\"usDiff\": 286,\"testnet\": true}";
        let details = serde_json::from_str::<TransferDetails>(response_text).unwrap();
        assert_eq!(details.result.count, 0);
        assert!(details.result.data.is_empty());
    }

    #[test]
    fn get_transfers() {
        let response_text = "{\"jsonrpc\": \"2.0\",\"result\": {\"count\": 1,\"data\": [{\"id\": 291951,\"amount\": 0.1,\"currency\": \"BTC\",\"direction\": \"payment\",\"state\": \"confirmed\",\"type\": \"subaccount\",\"other_side\": \"new_user_1_1\",\"created_timestamp\": 1675065819382,\"updated_timestamp\": 1675065819382,\"note\": \"\"}]},\"usIn\": 1675065835747899,\"usOut\": 1675065835748142,\"usDiff\": 243,\"testnet\": true}";
        let details = serde_json::from_str::<TransferDetails>(response_text).unwrap();
        assert!(!details.result.data.is_empty());
        assert_eq!(details.result.data[0].currency, Currency::BTC.to_string(),);
    }

    #[test]
    fn get_withdrawals_empty() {
        let response_text = "{\"jsonrpc\": \"2.0\",\"result\": {\"data\": [],\"count\": 0},\"usIn\": 1675066247615127,\"usOut\": 1675066247615279,\"usDiff\": 152,\"testnet\": true}";
        let details = serde_json::from_str::<WithdrawalDetails>(response_text).unwrap();
        assert_eq!(details.result.count, 0);
        assert!(details.result.data.is_empty());
    }

    #[test]
    fn get_withdrawals() {
        let response_text = "";
        let details = serde_json::from_str::<WithdrawalDetails>(response_text).unwrap();
        assert!(!details.result.data.is_empty());
        assert_eq!(details.result.data[0].currency, Currency::BTC.to_string(),);
    }
}
