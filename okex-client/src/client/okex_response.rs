use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct OkexResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: Option<Vec<T>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressData {
    pub chain: String,
    pub ct_addr: String,
    pub ccy: String,
    pub to: String,
    pub addr: String,
    pub selected: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferData {
    pub trans_id: String,
    pub ccy: String,
    pub client_id: String,
    pub from: String,
    pub amt: String,
    pub to: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundingBalanceData {
    pub avail_bal: String,
    pub bal: String,
    pub ccy: String,
    pub frozen_bal: String,
}
