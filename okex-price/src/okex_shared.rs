use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelArgs {
    pub channel: String,
    pub inst_id: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub struct UnsubscribeResponse {
    pub event: String,
    pub arg: ChannelArgs,
}
