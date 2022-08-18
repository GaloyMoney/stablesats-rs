use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct OkexResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: Option<Vec<T>>,
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferData {
    pub trans_id: String,
    pub ccy: String,
    pub client_id: String,
    pub from: String,
    pub amt: String,
    pub to: String,
}

#[derive(Deserialize, Debug)]
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
    pub cash_bal: Decimal,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawData {
    pub amt: String,
    pub wd_id: String,
    pub ccy: String,
    pub client_id: String,
    pub chain: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DepositHistoryData {
    pub actual_dep_blk_confirm: String,
    pub amt: String,
    pub ccy: String,
    pub chain: String,
    pub dep_id: String,
    pub from: String,
    pub state: String,
    pub to: String,
    pub ts: String,
    pub tx_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
    pub cl_ord_id: String,
    pub ord_id: String,
    pub tag: String,
    pub s_code: String,
    pub s_msg: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PositionData {
    pub adl: String,
    pub avail_pos: String,
    pub avg_px: String,
    pub c_time: String,
    pub ccy: String,
    pub delta_b_s: String,
    pub delta_p_a: String,
    pub gamma_b_s: String,
    pub gamma_p_a: String,
    pub imr: String,
    pub inst_id: String,
    pub inst_type: String,
    pub interest: String,
    pub usd_px: String,
    pub last: String,
    pub lever: String,
    pub liab: String,
    pub liab_ccy: String,
    pub liq_px: String,
    pub mark_px: String,
    pub margin: String,
    pub mgn_mode: String,
    pub mgn_ratio: String,
    pub mmr: String,
    pub notional_usd: String,
    pub opt_val: String,
    // pub p_time: String,
    pub pos: String,
    pub pos_ccy: String,
    pub pos_id: String,
    pub pos_side: String,
    pub theta_b_s: String,
    pub theta_p_a: String,
    pub trade_id: String,
    pub u_time: String,
    pub upl: String,
    pub upl_ratio: String,
    pub vega_b_s: String,
    pub vega_p_a: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClosePositionData {
    pub inst_id: String,
    pub pos_side: String,
}
