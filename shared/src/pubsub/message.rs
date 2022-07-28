use chrono::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, Debug, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct CorrelationId(Uuid);
impl CorrelationId {
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        // tracing::Span::current().record("correlation_id", &tracing::field::display(id));
        Self(id)
    }
}
impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventMetadata {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub published_at: DateTime<Utc>,
    pub correlation_id: CorrelationId,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            correlation_id: CorrelationId::new(),
            published_at: Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<P: MessagePayload> {
    // pub tracing_data: TracingData,
    pub meta: EventMetadata,
    pub payload_type: String,
    #[serde(bound = "P: DeserializeOwned")]
    pub payload: P,
}

impl<P: MessagePayload> Envelope<P> {
    pub(super) fn new(payload: P) -> Self {
        Self {
            meta: EventMetadata::new(),
            payload_type: <P as MessagePayload>::message_type().to_string(),
            payload,
        }
    }
}

pub trait MessagePayload:
    Serialize + DeserializeOwned + Clone + Sized + Sync + Send + Unpin + 'static
{
    fn message_type() -> &'static str;
    fn channel() -> &'static str;
}

pub mod serialize_as_string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
