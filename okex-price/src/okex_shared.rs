use serde::Deserialize;

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelArgs {
    pub channel: String,
    pub inst_id: String,
}
