use chrono::{prelude::*, Duration};
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
impl std::fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.timestamp())
    }
}
impl std::ops::Sub for &TimeStamp {
    type Output = Duration;
    fn sub(self, other: Self) -> Self::Output {
        self.0.signed_duration_since(other.0)
    }
}
