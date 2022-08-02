use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeStamp(#[serde(with = "chrono::serde::ts_seconds")] DateTime<Utc>);
impl TimeStamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}
impl PartialOrd for TimeStamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
