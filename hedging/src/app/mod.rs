mod config;

use futures::stream::StreamExt;
use opentelemetry::{propagation::TextMapPropagator, sdk::propagation::TraceContextPropagator};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlxmq::OwnedHandle;
use tracing::{info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use galoy_client::*;
use okex_client::*;
use shared::{
    health::HealthCheckTrigger,
    payload::{
        OkexBtcUsdSwapPositionPayload, OkexBtcUsdSwapPricePayload, SynthUsdLiabilityPayload,
    },
    pubsub::{memory, CorrelationId, PubSubConfig, Publisher, Subscriber},
};

use crate::{
    adjustment_action::*, error::*, job, okex_orders::*, okex_transfers::*, rebalance_action::*,
    synth_usd_liability::*,
};

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
            health: health_cfg,
            hedging: hedging_config,
            funding: funding_config,
            ..
        }: HedgingAppConfig,
        okex_client_config: OkexClientConfig,
        galoy_client_cfg: GaloyClientConfig,
        pubsub_config: PubSubConfig,
        tick_receiver: memory::Subscriber<OkexBtcUsdSwapPricePayload>,
    ) -> Result<Self, HedgingError> {
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        if migrate_on_start {
            sqlx::migrate!().run(&pool).await?;
        }
        let synth_usd_liability = SynthUsdLiability::new(pool.clone());
        let okex_orders = OkexOrders::new(pool.clone()).await?;
        let okex_transfers = OkexTransfers::new(pool.clone()).await?;
        let okex = OkexClient::new(okex_client_config).await?;
        let funding_adjustment =
            FundingAdjustment::new(funding_config.clone(), hedging_config.clone());
        let hedging_adjustment = HedgingAdjustment::new(hedging_config);
        let job_runner = job::start_job_runner(
            pool.clone(),
            synth_usd_liability.clone(),
            okex.clone(),
            okex_orders,
            okex_transfers.clone(),
            GaloyClient::connect(galoy_client_cfg).await?,
            Publisher::new(pubsub_config.clone()).await?,
            okex_poll_delay,
            funding_adjustment.clone(),
            hedging_adjustment.clone(),
            funding_config.clone(),
        )
        .await?;
        let liability_sub =
            Self::spawn_synth_usd_listener(pubsub_config.clone(), synth_usd_liability.clone())
                .await?;
        let position_sub = Self::spawn_okex_position_listener(
            pubsub_config.clone(),
            pool.clone(),
            synth_usd_liability.clone(),
            hedging_adjustment,
        )
        .await?;
        Self::spawn_okex_price_listener(
            pool.clone(),
            synth_usd_liability,
            okex,
            funding_adjustment,
            tick_receiver.resubscribe(),
        )
        .await?;
        Self::spawn_health_checker(
            health_check_trigger,
            health_cfg,
            liability_sub,
            position_sub,
            tick_receiver,
        )
        .await;
        Self::spawn_non_stop_polling(pool.clone(), okex_poll_delay).await?;
        let app = HedgingApp {
            _runner: job_runner,
        };
        Ok(app)
    }

    async fn spawn_okex_price_listener(
        pool: sqlx::PgPool,
        synth_usd_liability: SynthUsdLiability,
        okex: OkexClient,
        funding_adjustment: FundingAdjustment,
        mut tick_recv: memory::Subscriber<OkexBtcUsdSwapPricePayload>,
    ) -> Result<(), HedgingError> {
        let _ = tokio::spawn(async move {
            while let Some(msg) = tick_recv.next().await {
                let correlation_id = msg.meta.correlation_id;
                let span = info_span!(
                    "okex_btc_usd_swap_price_received",
                    message_type = %msg.payload_type,
                    correlation_id = %correlation_id,
                    error = tracing::field::Empty,
                    error.level = tracing::field::Empty,
                    error.message = tracing::field::Empty,
                );
                shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                let _ = Self::handle_received_okex_price(
                    msg.payload,
                    correlation_id,
                    &pool,
                    &synth_usd_liability,
                    &okex,
                    funding_adjustment.clone(),
                )
                .instrument(span)
                .await;
            }
        });
        Ok(())
    }

    async fn handle_received_okex_price(
        payload: OkexBtcUsdSwapPricePayload,
        correlation_id: CorrelationId,
        pool: &sqlx::PgPool,
        synth_usd_liability: &SynthUsdLiability,
        okex: &OkexClient,
        funding_adjustment: FundingAdjustment,
    ) -> Result<(), HedgingError> {
        let target_liability_in_cents = synth_usd_liability.get_latest_liability().await?;
        let current_position_in_cents = okex.get_position_in_signed_usd_cents().await?.usd_cents;
        let trading_available_balance = okex.trading_account_balance().await?;
        let funding_available_balance = okex.funding_account_balance().await?;

        let mid_price_in_cents: Decimal =
            (payload.bid_price.numerator_amount() + payload.ask_price.numerator_amount()) / dec!(2);

        if funding_adjustment
            .determine_action(
                target_liability_in_cents,
                current_position_in_cents.into(),
                trading_available_balance.total_amt_in_btc,
                mid_price_in_cents,
                funding_available_balance.total_amt_in_btc,
            )
            .action_required()
        {
            job::spawn_adjust_funding(pool, correlation_id).await?;
        }
        Ok(())
    }

    async fn spawn_non_stop_polling(
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
        let mut subscriber = Subscriber::new(config).await?;
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
        hedging_adjustment: HedgingAdjustment,
    ) -> Result<Subscriber, HedgingError> {
        let mut subscriber = Subscriber::new(config).await?;
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
                    hedging_adjustment.clone(),
                )
                .instrument(span)
                .await;
            }
        });
        Ok(subscriber)
    }

    async fn spawn_health_checker(
        mut health_check_trigger: HealthCheckTrigger,
        health_cfg: HedgingAppHealthConfig,
        liability_sub: Subscriber,
        position_sub: Subscriber,
        price_sub: memory::Subscriber<OkexBtcUsdSwapPricePayload>,
    ) {
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.next().await {
                match (
                    liability_sub
                        .healthy(health_cfg.unhealthy_msg_interval_liability)
                        .await,
                    position_sub
                        .healthy(health_cfg.unhealthy_msg_interval_position)
                        .await,
                    price_sub
                        .healthy(health_cfg.unhealthy_msg_interval_price)
                        .await,
                ) {
                    (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
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
        hedging_adjustment: HedgingAdjustment,
    ) -> Result<(), HedgingError> {
        let amount = synth_usd_liability.get_latest_liability().await?;
        if hedging_adjustment
            .determine_action(amount, payload.signed_usd_exposure)
            .action_required()
        {
            job::spawn_adjust_hedge(pool, correlation_id).await?;
        }
        Ok(())
    }
}
