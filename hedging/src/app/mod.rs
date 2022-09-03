mod config;

use futures::stream::StreamExt;
use opentelemetry::{propagation::TextMapPropagator, sdk::propagation::TraceContextPropagator};
use sqlxmq::OwnedHandle;
use tracing::{info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use okex_client::*;
use shared::{
    payload::SynthUsdLiabilityPayload,
    pubsub::{CorrelationId, PubSubConfig, Subscriber},
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
            migrate_on_start
        }: HedgingAppConfig,
        okex_client_config: OkexClientConfig,
        config: PubSubConfig,
    ) -> Result<Self, HedgingError> {
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        if migrate_on_start {
            sqlx::migrate!().run(&pool).await?;
        }
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
                println!("MESSAGE: {:?}", msg);
                let correlation_id = msg.meta.correlation_id;
                let span = info_span!(
                    "synth_usd_liability_received",
                    message_type = %msg.payload_type,
                    correlation_id = %correlation_id
                );
                let context = propagator.extract(&msg.meta.tracing_data);
                span.set_parent(context);
                let res = Self::handle_received_synth_usd_liability(
                    msg.payload,
                    correlation_id,
                    &synth_usd_liability,
                )
                .instrument(span)
                .await;
                println!("HANDLE RETURNED: {:?}", res);
            }
        });
        Ok(app)
    }

    async fn handle_received_synth_usd_liability(
        payload: SynthUsdLiabilityPayload,
        correlation_id: CorrelationId,
        synth_usd_liability: &SynthUsdLiability,
    ) -> Result<(), HedgingError> {
        println!("HANDLE");
        match synth_usd_liability
            .insert_if_new(correlation_id, payload.liability)
            .await
        {
            Ok(Some(mut tx)) => {
                job::spawn_adjust_hedge(&mut tx).await?;
                tx.commit().await?;
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
