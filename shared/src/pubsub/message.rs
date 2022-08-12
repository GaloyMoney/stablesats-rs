use opentelemetry::{propagation::TextMapPropagator, sdk::propagation::TraceContextPropagator};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

use std::collections::HashMap;

use crate::time::*;

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, Debug, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct CorrelationId(Uuid);
impl CorrelationId {
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        tracing::Span::current().record("correlation_id", &tracing::field::display(id));
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
pub struct MessageMetadata {
    #[serde(flatten)]
    pub tracing_data: HashMap<String, String>,
    pub published_at: TimeStamp,
    pub correlation_id: CorrelationId,
}

impl MessageMetadata {
    pub fn new() -> Self {
        let mut tracing_data = HashMap::new();
        let propagator = TraceContextPropagator::new();
        let context = Span::current().context();
        propagator.inject_context(&context, &mut tracing_data);

        Self {
            tracing_data,
            correlation_id: CorrelationId::new(),
            published_at: TimeStamp::now(),
        }
    }
}
impl Default for MessageMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<P: MessagePayload> {
    pub meta: MessageMetadata,
    pub payload_type: String,
    #[serde(bound = "P: DeserializeOwned")]
    pub payload: P,
}

impl<P: MessagePayload> Envelope<P> {
    pub(super) fn new(payload: P) -> Self {
        Self {
            meta: MessageMetadata::new(),
            payload_type: <P as MessagePayload>::message_type().to_string(),
            payload,
        }
    }
}

pub trait MessagePayload:
    std::fmt::Debug + Serialize + DeserializeOwned + Clone + Sized + Sync + Send + Unpin + 'static
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
