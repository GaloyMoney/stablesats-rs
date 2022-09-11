use opentelemetry::{propagation::TextMapPropagator, sdk::propagation::TraceContextPropagator};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use std::collections::HashMap;

pub fn extract_tracing_data() -> HashMap<String, String> {
    let mut tracing_data = HashMap::new();
    let propagator = TraceContextPropagator::new();
    let context = Span::current().context();
    propagator.inject_context(&context, &mut tracing_data);
    tracing_data
}

pub fn inject_tracing_data(span: &Span, tracing_data: &HashMap<String, String>) {
    let propagator = TraceContextPropagator::new();
    let context = propagator.extract(tracing_data);
    span.set_parent(context);
}

pub async fn record_error<
    T,
    E: std::fmt::Display,
    F: FnOnce() -> R,
    R: std::future::Future<Output = Result<T, E>>,
>(
    func: F,
) -> Result<T, E> {
    let result = func().await;
    if let Err(ref e) = result {
        insert_error_fields(e);
    }
    result
}

pub fn insert_error_fields(error: impl std::fmt::Display) {
    Span::current().record("error", &tracing::field::display("true"));
    Span::current().record("error.message", &tracing::field::display(error));
}
