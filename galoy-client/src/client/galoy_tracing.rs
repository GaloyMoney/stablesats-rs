use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_http::HeaderInjector;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use reqwest::header::HeaderMap;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub fn inject_trace() -> HeaderMap {
    let mut header_map = HeaderMap::new();
    let mut header_wrapper = HeaderInjector(&mut header_map);
    let propagator = TraceContextPropagator::new();
    let context = Span::current().context();
    propagator.inject_context(&context, &mut header_wrapper);

    header_map
}
