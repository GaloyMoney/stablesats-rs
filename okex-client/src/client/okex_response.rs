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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradingBalanceData {
    pub adj_eq: String,
    pub details: Vec<TradingBalanceDetails>,
    pub imr: String,
    pub iso_eq: String,
    pub mgn_ratio: String,
    pub mmr: String,
    pub notional_usd: String,
    pub ord_froz: String,
    pub total_eq: String,
    pub u_time: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradingBalanceDetails {
    pub avail_bal: String,
    pub avail_eq: String,
    pub cash_bal: String,
    pub ccy: String,
    pub cross_liab: String,
    pub dis_eq: String,
    pub eq: String,
    pub eq_usd: String,
    pub frozen_bal: String,
    pub interest: String,
    pub iso_eq: String,
    pub iso_liab: String,
    pub iso_upl: String,
    pub liab: String,
    pub max_loan: String,
    pub mgn_ratio: String,
    pub notional_lever: String,
    pub ord_frozen: String,
    pub twap: String,
    pub u_time: String,
    pub upl: String,
    pub upl_liab: String,
    pub stgy_eq: String,
}
