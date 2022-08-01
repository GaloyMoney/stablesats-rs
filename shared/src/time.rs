use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeStamp(#[serde(with = "chrono::serde::ts_seconds")] DateTime<Utc>);
impl TimeStamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}
