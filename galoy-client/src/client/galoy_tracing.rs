use opentelemetry::{
    propagation::{Injector, TextMapPropagator},
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

pub fn extract_tracing_data() -> HeaderMap {
    let mut header_wrapper = HeaderMapWrapper::new();
    let propagator = TraceContextPropagator::new();
    let context = Span::current().context();
    propagator.inject_context(&context, &mut header_wrapper);

    header_wrapper.map
}
