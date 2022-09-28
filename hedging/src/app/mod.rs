mod config;

use futures::stream::StreamExt;
use opentelemetry::{propagation::TextMapPropagator, sdk::propagation::TraceContextPropagator};
use sqlxmq::OwnedHandle;
use tracing::{info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use okex_client::*;
use shared::{
    health::HealthCheckTrigger,
    payload::{OkexBtcUsdSwapPositionPayload, SynthUsdLiabilityPayload},
    pubsub::{CorrelationId, PubSubConfig, Publisher, Subscriber},
};

use crate::{adjustment_action, error::*, job, okex_orders::*, synth_usd_liability::*};

pub use config::*;

pub struct HedgingApp {
    _runner: OwnedHandle,
}

impl HedgingApp {
    pub async fn run(
        health_check_trigger: HealthCheckTrigger,
        HedgingAppConfig {
            pg_con,
            migrate_on_start,
            okex_poll_frequency: okex_poll_delay,
        }: HedgingAppConfig,
        okex_client_config: OkexClientConfig,
        pubsub_config: PubSubConfig,
    ) -> Result<Self, HedgingError> {
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        if migrate_on_start {
            sqlx::migrate!().run(&pool).await?;
        }
        let synth_usd_liability = SynthUsdLiability::new(pool.clone());
        let okex_orders = OkexOrders::new(pool.clone()).await?;
        let job_runner = job::start_job_runner(
            pool.clone(),
            synth_usd_liability.clone(),
            OkexClient::new(okex_client_config).await?,
            okex_orders,
            Publisher::new(pubsub_config.clone()).await?,
            okex_poll_delay,
        )
        .await?;
        let liability_sub =
            Self::spawn_synth_usd_listener(pubsub_config.clone(), synth_usd_liability.clone())
                .await?;
        let position_sub =
            Self::spawn_okex_position_listener(pubsub_config, pool.clone(), synth_usd_liability)
                .await?;
        Self::spawn_health_checker(health_check_trigger, liability_sub, position_sub).await;
        Self::spawn_okex_polling(pool, okex_poll_delay).await?;
        let app = HedgingApp {
            _runner: job_runner,
        };
        Ok(app)
    }

    async fn spawn_okex_polling(
        pool: sqlx::PgPool,
        delay: std::time::Duration,
    ) -> Result<(), HedgingError> {
        loop {
            let _ = job::spawn_poll_okex(&pool, std::time::Duration::from_secs(1)).await;
            tokio::time::sleep(delay).await;
        }
    }

    async fn spawn_synth_usd_listener(
        config: PubSubConfig,
        synth_usd_liability: SynthUsdLiability,
    ) -> Result<Subscriber, HedgingError> {
        let subscriber = Subscriber::new(config).await?;
        let mut stream = subscriber.subscribe::<SynthUsdLiabilityPayload>().await?;
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
                let _ = Self::handle_received_synth_usd_liability(
                    msg.payload,
                    correlation_id,
                    &synth_usd_liability,
                )
                .instrument(span)
                .await;
            }
        });
        Ok(subscriber)
    }

    async fn spawn_okex_position_listener(
        config: PubSubConfig,
        pool: sqlx::PgPool,
        synth_usd_liability: SynthUsdLiability,
    ) -> Result<Subscriber, HedgingError> {
        let subscriber = Subscriber::new(config).await?;
        let mut stream = subscriber
            .subscribe::<OkexBtcUsdSwapPositionPayload>()
            .await?;
        let _ = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let correlation_id = msg.meta.correlation_id;
                let span = info_span!(
                    "okex_btc_usd_swap_position_received",
                    message_type = %msg.payload_type,
                    correlation_id = %correlation_id,
                    error = tracing::field::Empty,
                    error.level = tracing::field::Empty,
                    error.message = tracing::field::Empty,
                );
                shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                let _ = Self::handle_received_okex_position(
                    msg.payload,
                    correlation_id,
                    &pool,
                    &synth_usd_liability,
                )
                .instrument(span)
                .await;
            }
        });
        Ok(subscriber)
    }

    async fn spawn_health_checker(
        mut health_check_trigger: HealthCheckTrigger,
        liability_sub: Subscriber,
        position_sub: Subscriber,
    ) {
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.next().await {
                let duration = chrono::Duration::seconds(20);
                match (
                    liability_sub.healthy(duration).await,
                    position_sub.healthy(duration).await,
                ) {
                    (Err(e), _) | (_, Err(e)) => {
                        check.send(Err(e)).expect("Couldn't send response")
                    }
                    _ => check.send(Ok(())).expect("Couldn't send response"),
                }
            }
        });
    }

    async fn handle_received_synth_usd_liability(
        payload: SynthUsdLiabilityPayload,
        correlation_id: CorrelationId,
        synth_usd_liability: &SynthUsdLiability,
    ) -> Result<(), HedgingError> {
        match synth_usd_liability
            .insert_if_new(correlation_id, payload.liability)
            .await
        {
            Ok(Some(mut tx)) => {
                job::spawn_adjust_hedge(&mut tx, correlation_id).await?;
                tx.commit().await?;
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(e) => {
                shared::tracing::insert_error_fields(tracing::Level::ERROR, &e);
                Err(e)
            }
        }
    }

    async fn handle_received_okex_position(
        payload: OkexBtcUsdSwapPositionPayload,
        correlation_id: CorrelationId,
        pool: &sqlx::PgPool,
        synth_usd_liability: &SynthUsdLiability,
    ) -> Result<(), HedgingError> {
        let amount = synth_usd_liability.get_latest_liability().await?;
        if adjustment_action::determine_action(amount, payload.signed_usd_exposure)
            .action_required()
        {
            job::spawn_adjust_hedge(pool, correlation_id).await?;
        }
        Ok(())
    }
}
