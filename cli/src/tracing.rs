use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Sampler;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    host: String,
    port: u16,
    service_name: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 4318,
            service_name: "stablesats-dev".to_string(),
        }
    }
}

pub fn init_tracer(config: TracingConfig) -> anyhow::Result<()> {
    let tracing_endpoint = format!("http://{}:{}", config.host, config.port);
    println!("Sending traces to {tracing_endpoint}");
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(tracing_endpoint),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(opentelemetry_sdk::Resource::new(vec![KeyValue::new(
                    "service.name",
                    config.service_name,
                )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = fmt::layer().json();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,sqlx=warn,sqlx_ledger=info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(telemetry)
        .try_init()?;

    Ok(())
}
