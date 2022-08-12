use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracer(tracing_endpoint: String, service: &str) -> anyhow::Result<()> {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_agent_endpoint(tracing_endpoint)
        .with_service_name(service)
        .install_simple()?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = fmt::layer();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(telemetry)
        .try_init()?;

    Ok(())
}
