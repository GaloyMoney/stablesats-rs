use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelArgs {
    pub channel: String,
    pub inst_id: String,
}

#[derive(serde::Serialize)]
pub struct Subscribe {
    op: String,
    args: Vec<ChannelArgs>,
}

impl Subscribe {
    pub fn new() -> Self {
        Self {
            op: "subscribe".to_string(),
            args: vec![ChannelArgs {
                channel: "tickers".to_string(),
                inst_id: "BTC-USD-SWAP".to_string(),
            }],
        }
    }
}

impl Default for Subscribe {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TickersChannelData {
    pub ask_px: String,
    pub bid_px: String,
    pub ts: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OkexPriceTick {
    pub arg: ChannelArgs,
    pub data: Vec<TickersChannelData>,
}
