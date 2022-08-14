use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum OkexResponse {
    WithData(ResponseWithData),
    WithoutData(ResponseWithoutData),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum OkexResponseData {
    DepositAddress(DepositAddressData),
    TransferState(TransferStateData),
    Transfer(TransferData),
    Balance(BalanceData),
}

/// Response struct from OKEX
#[derive(Deserialize, Debug)]
pub struct ResponseWithData {
    pub code: String,
    pub msg: String,
    pub data: Vec<OkexResponseData>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseWithoutData {
    pub code: String,
    pub msg: String,
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
pub struct BalanceData {
    pub avail_bal: String,
    pub bal: String,
    pub ccy: String,
    pub frozen_bal: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferStateData {
    pub amt: String,
    pub ccy: String,
    pub client_id: String,
    pub from: String,
    pub state: String,
    pub sub_acct: String,
    pub to: String,
    pub trans_id: String,
}
