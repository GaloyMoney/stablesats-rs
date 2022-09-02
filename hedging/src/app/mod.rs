mod config;

use futures::stream::StreamExt;
use opentelemetry::{propagation::TextMapPropagator, sdk::propagation::TraceContextPropagator};
use sqlxmq::OwnedHandle;
use tracing::{info_span, instrument, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use okex_client::*;
use shared::{
    payload::SynthUsdLiabilityPayload,
    pubsub::{PubSubConfig, Subscriber},
};

use crate::{error::*, job, synth_usd_liability::*};

pub use config::*;

pub struct HedgingApp {
    _runner: OwnedHandle,
}

impl HedgingApp {
    pub async fn run(
        HedgingAppConfig {
            pg_con,
            migrate_on_start: _,
        }: HedgingAppConfig,
        okex_client_config: OkexClientConfig,
        config: PubSubConfig,
    ) -> Result<Self, HedgingError> {
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        let subscriber = Subscriber::new(config).await?;
        let mut stream = subscriber.subscribe::<SynthUsdLiabilityPayload>().await?;
        let synth_usd_liability = SynthUsdLiability::new(pool.clone());
        let job_runner = job::start_job_runner(
            pool.clone(),
            synth_usd_liability.clone(),
            OkexClient::new(okex_client_config).await?,
        )
        .await?;
        let app = HedgingApp {
            _runner: job_runner,
        };
        let _ = tokio::spawn(async move {
            let propagator = TraceContextPropagator::new();

            while let Some(msg) = stream.next().await {
                let correlation_id = msg.meta.correlation_id;
                let span = info_span!(
                    "synth_usd_liability_received",
                    message_type = %msg.payload_type,
                    correlation_id = %correlation_id
                );
                let context = propagator.extract(&msg.meta.tracing_data);
                span.set_parent(context);

                match synth_usd_liability
                    .insert_if_new(correlation_id, msg.payload.liability)
                    .await
                {
                    Ok(true) => if let Err(_) = job::spawn_adjust_hedge(&pool).await {},
                    _ => {}
                }
            }
        });
        Ok(app)
    }
}
