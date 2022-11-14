use chrono::{prelude::*, Duration};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeStampError {
    #[error("ParseIntError - {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("EmptyNaiveDateTime - {0}")]
    EmptyNaiveDateTime(String),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeStamp(#[serde(with = "chrono::serde::ts_seconds")] DateTime<Utc>);
impl TimeStamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn duration_since(&self) -> Duration {
        &Self::now() - self
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
impl std::ops::Sub for TimeStamp {
    type Output = Duration;
    fn sub(self, other: Self) -> Self::Output {
        self.0.signed_duration_since(other.0)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeStampMilliStr(String);
impl TryFrom<&TimeStampMilliStr> for TimeStamp {
    type Error = TimeStampError;

    fn try_from(value: &TimeStampMilliStr) -> Result<Self, Self::Error> {
        let millis = value.0.parse::<i64>()?;
        if let Some(naive) = NaiveDateTime::from_timestamp_opt(millis / 1000, 0) {
            let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            Ok(Self(datetime))
        } else {
            Err(TimeStampError::EmptyNaiveDateTime(
                "Out-of-range no of seconds or invalid nanosecond".to_string(),
            ))
        }
    }
}
