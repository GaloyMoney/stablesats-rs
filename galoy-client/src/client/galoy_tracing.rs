use opentelemetry::{
    propagation::{Extractor, Injector, TextMapPropagator},
    sdk::propagation::TraceContextPropagator,
};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub struct HeaderMapWrapper {
    pub map: HeaderMap,
}

impl HeaderMapWrapper {
    pub fn new() -> Self {
        Self {
            map: HeaderMap::new(),
        }
    }
}

impl Injector for HeaderMapWrapper {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(name) = HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(val) = HeaderValue::from_str(&value) {
                self.map.insert(name, val);
            }
        }
    }
}

impl Extractor for HeaderMapWrapper {
    /// Get a value for a key from the HeaderMap.  If the value is not valid ASCII, returns None.
    fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).and_then(|value| value.to_str().ok())
    }

    /// Collect all the keys from the HeaderMap.
    fn keys(&self) -> Vec<&str> {
        self.map
            .keys()
            .map(|value| value.as_str())
            .collect::<Vec<_>>()
    }
}

pub fn extract_tracing_data(mut header_wrapper: HeaderMapWrapper) -> HeaderMap {
    let propagator = TraceContextPropagator::new();
    let context = Span::current().context();
    propagator.inject_context(&context, &mut header_wrapper);

    header_wrapper.map
}

pub fn inject_tracing_data(span: &Span, tracing_data: &HeaderMapWrapper) {
    let propagator = TraceContextPropagator::new();
    let context = propagator.extract(tracing_data);
    span.set_parent(context);
}
