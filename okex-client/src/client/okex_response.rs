use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum OkexResponse {
    WithData(ResponseWithData),
    WithoutData(ResponseWithoutData),
}

/// Response struct from OKEX
#[derive(Deserialize, Debug)]
pub struct ResponseWithData {
    pub code: String,
    pub msg: String,
    pub data: Vec<DepositAddressData>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseWithoutData {
    pub code: String,
    pub msg: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressData {
    pub chain: String,
    pub ct_addr: String,
    pub ccy: String,
    pub to: String,
    pub addr: String,
    pub selected: bool,
}
