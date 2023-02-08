mod config;

use futures::stream::StreamExt;
use sqlxmq::OwnedHandle;
use tracing::{info_span, Instrument};

use galoy_client::*;
use okex_client::*;
use shared::{
    exchanges_config::OkexConfig,
    health::HealthCheckTrigger,
    payload::{OkexBtcUsdSwapPositionPayload, PriceStreamPayload},
    pubsub::{memory, CorrelationId, PubSubConfig, Publisher, Subscriber},
};

use crate::{
    adjustment_action::*, error::*, job, okex_orders::*, okex_transfers::*, rebalance_action::*,
};

pub use config::*;

pub struct HedgingApp {
    _runner: OwnedHandle,
}

impl HedgingApp {
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        pool: sqlx::PgPool,
        health_check_trigger: HealthCheckTrigger,
        HedgingAppConfig {
            okex_poll_frequency: okex_poll_delay,
            health: health_cfg,
            hedging: hedging_config,
            funding: funding_config,
            ..
        }: HedgingAppConfig,
        okex_client_config: OkexConfig,
        galoy_client_cfg: GaloyClientConfig,
        pubsub_config: PubSubConfig,
        price_receiver: memory::Subscriber<PriceStreamPayload>,
    ) -> Result<Self, HedgingError> {
        let okex_orders = OkexOrders::new(pool.clone()).await?;
        let okex_transfers = OkexTransfers::new(pool.clone()).await?;
        let okex = OkexClient::new(okex_client_config).await?;
        okex.check_leverage(funding_config.high_bound_ratio_leverage)
            .await?;
        let funding_adjustment =
            FundingAdjustment::new(funding_config.clone(), hedging_config.clone());
        let hedging_adjustment = HedgingAdjustment::new(hedging_config);
        let ledger = ledger::Ledger::init(&pool).await?;
        let job_runner = job::start_job_runner(
            pool.clone(),
            ledger.clone(),
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
        Self::spawn_liability_balance_listener(pool.clone(), ledger.clone()).await?;
        let position_sub = Self::spawn_okex_position_listener(
            pubsub_config.clone(),
            pool.clone(),
            ledger.clone(),
            hedging_adjustment,
        )
        .await?;
        Self::spawn_okex_price_listener(
            pool.clone(),
            ledger,
            okex,
            funding_adjustment,
            price_receiver.resubscribe(),
        )
        .await?;
        Self::spawn_non_stop_polling(pool.clone(), okex_poll_delay).await?;
        Self::spawn_health_checker(
            health_check_trigger,
            health_cfg,
            position_sub,
            price_receiver,
        )
        .await;
        let app = HedgingApp {
            _runner: job_runner,
        };
        Ok(app)
    }

    async fn spawn_okex_price_listener(
        pool: sqlx::PgPool,
        ledger: ledger::Ledger,
        okex: OkexClient,
        funding_adjustment: FundingAdjustment,
        mut tick_recv: memory::Subscriber<PriceStreamPayload>,
    ) -> Result<(), HedgingError> {
        tokio::spawn(async move {
            while let Some(msg) = tick_recv.next().await {
                if let PriceStreamPayload::OkexBtcSwapPricePayload(_) = msg.payload {
                    let correlation_id = msg.meta.correlation_id;
                    let span = info_span!(
                        "hedging.okex_btc_usd_swap_price_received",
                        message_type = %msg.payload_type,
                        correlation_id = %correlation_id,
                        error = tracing::field::Empty,
                        error.level = tracing::field::Empty,
                        error.message = tracing::field::Empty,
                    );
                    shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                    let _ = Self::handle_received_okex_price(
                        correlation_id,
                        &pool,
                        &ledger,
                        &okex,
                        funding_adjustment.clone(),
                    )
                    .instrument(span)
                    .await;
                }
            }
        });
        Ok(())
    }

    async fn handle_received_okex_price(
        correlation_id: CorrelationId,
        pool: &sqlx::PgPool,
        ledger: &ledger::Ledger,
        okex: &OkexClient,
        funding_adjustment: FundingAdjustment,
    ) -> Result<(), HedgingError> {
        let target_liability_in_cents = ledger.balances().target_liability_in_cents().await?;
        let current_position_in_cents = okex.get_position_in_signed_usd_cents().await?.usd_cents;
        let last_price_in_usd_cents = okex.get_last_price_in_usd_cents().await?.usd_cents;
        let trading_available_balance = okex.trading_account_balance().await?;
        let funding_available_balance = okex.funding_account_balance().await?;

        if funding_adjustment
            .determine_action(
                target_liability_in_cents,
                current_position_in_cents.into(),
                trading_available_balance.total_amt_in_btc,
                last_price_in_usd_cents,
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
        tokio::spawn(async move {
            loop {
                let _ = job::spawn_poll_okex(&pool, std::time::Duration::from_secs(1)).await;
                tokio::time::sleep(delay).await;
            }
        });
        Ok(())
    }

    async fn spawn_liability_balance_listener(
        pool: sqlx::PgPool,
        ledger: ledger::Ledger,
    ) -> Result<(), HedgingError> {
        tokio::spawn(async move {
            let _ = job::spawn_adjust_hedge(&pool, uuid::Uuid::new_v4()).await;
            let mut events = ledger.usd_liability_balance_events().await;
            loop {
                match events.recv().await {
                    Ok(received) => {
                        if let ledger::LedgerEventData::BalanceUpdated(data) = received.data {
                            let _ = job::spawn_adjust_hedge(&pool, data.entry_id).await;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => (),
                    _ => {
                        break;
                    }
                }
            }
        });
        Ok(())
    }

    async fn spawn_okex_position_listener(
        config: PubSubConfig,
        pool: sqlx::PgPool,
        ledger: ledger::Ledger,
        hedging_adjustment: HedgingAdjustment,
    ) -> Result<Subscriber, HedgingError> {
        let mut subscriber = Subscriber::new(config).await?;
        let mut stream = subscriber
            .subscribe::<OkexBtcUsdSwapPositionPayload>()
            .await?;
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let correlation_id = msg.meta.correlation_id;
                let span = info_span!(
                    "hedging.okex_btc_usd_swap_position_received",
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
                    &ledger,
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
        position_sub: Subscriber,
        price_sub: memory::Subscriber<PriceStreamPayload>,
    ) {
        while let Some(check) = health_check_trigger.next().await {
            match (
                position_sub
                    .healthy(health_cfg.unhealthy_msg_interval_position)
                    .await,
                price_sub
                    .healthy(health_cfg.unhealthy_msg_interval_price)
                    .await,
            ) {
                (Err(e), _) | (_, Err(e)) => check.send(Err(e)).expect("Couldn't send response"),
                _ => check.send(Ok(())).expect("Couldn't send response"),
            }
        }
    }

    async fn handle_received_okex_position(
        payload: OkexBtcUsdSwapPositionPayload,
        correlation_id: CorrelationId,
        pool: &sqlx::PgPool,
        ledger: &ledger::Ledger,
        hedging_adjustment: HedgingAdjustment,
    ) -> Result<(), HedgingError> {
        let amount = ledger.balances().target_liability_in_cents().await?;
        if hedging_adjustment
            .determine_action(amount, payload.signed_usd_exposure)
            .action_required()
        {
            job::spawn_adjust_hedge(pool, correlation_id).await?;
        }
        Ok(())
    }
}
