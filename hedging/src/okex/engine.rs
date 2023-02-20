use futures::stream::StreamExt;
use sqlxmq::NamedJob;
use tracing::{info_span, instrument, Instrument};

use std::sync::Arc;

use ledger::Ledger;
use okex_client::OkexClient;
use shared::{
    payload::*,
    pubsub::{memory, PubSubConfig, Subscriber},
};

use super::{config::*, funding_adjustment::*, hedge_adjustment::*, job, orders::*, transfers::*};
use crate::error::HedgingError;

pub struct OkexEngine {
    config: OkexConfig,
    pool: sqlx::PgPool,
    orders: OkexOrders,
    transfers: OkexTransfers,
    okex_client: OkexClient,
    ledger: Ledger,
    funding_adjustment: FundingAdjustment,
    hedging_adjustment: HedgingAdjustment,
}

impl OkexEngine {
    pub async fn run(
        pool: sqlx::PgPool,
        config: OkexConfig,
        ledger: Ledger,
        pubsub_config: PubSubConfig,
        price_receiver: memory::Subscriber<PriceStreamPayload>,
    ) -> Result<(Arc<Self>, Subscriber), HedgingError> {
        let okex_client = OkexClient::new(config.client.clone()).await?;
        let orders = OkexOrders::new(pool.clone()).await?;
        let transfers = OkexTransfers::new(pool.clone()).await?;
        okex_client
            .check_leverage(config.funding.high_bound_ratio_leverage)
            .await?;
        let funding_adjustment =
            FundingAdjustment::new(config.funding.clone(), config.hedging.clone());
        let hedging_adjustment = HedgingAdjustment::new(config.hedging.clone());
        let ret = Arc::new(Self {
            config,
            pool,
            okex_client,
            orders,
            transfers,
            ledger,
            funding_adjustment,
            hedging_adjustment,
        });

        Arc::clone(&ret)
            .spawn_okex_price_listener(price_receiver)
            .await?;

        let subscriber = Arc::clone(&ret)
            .spawn_position_listener(pubsub_config)
            .await?;

        Arc::clone(&ret).spawn_liability_listener().await?;

        Arc::clone(&ret).spawn_non_stop_polling().await?;

        Ok((ret, subscriber))
    }

    pub fn add_context_to_job_registry(&self, runner: &mut sqlxmq::JobRegistry) {
        runner.set_context(self.okex_client.clone());
        runner.set_context(self.orders.clone());
        runner.set_context(self.transfers.clone());
        runner.set_context(job::OkexPollDelay(self.config.poll_frequency));
        runner.set_context(self.funding_adjustment.clone());
        runner.set_context(self.hedging_adjustment.clone());
        runner.set_context(self.config.funding.clone());
    }

    pub fn register_jobs(jobs: &mut Vec<&'static NamedJob>, channels: &mut Vec<&str>) {
        jobs.push(job::adjust_hedge);
        jobs.push(job::poll_okex);
        jobs.push(job::adjust_funding);
        channels.push("hedging.okex");
    }

    async fn spawn_okex_price_listener(
        self: Arc<Self>,
        mut tick_recv: memory::Subscriber<PriceStreamPayload>,
    ) -> Result<(), HedgingError> {
        tokio::spawn(async move {
            while let Some(msg) = tick_recv.next().await {
                if let PriceStreamPayload::OkexBtcSwapPricePayload(_) = msg.payload {
                    let correlation_id = msg.meta.correlation_id;
                    let span = info_span!(
                        "hedging.okex.okex_btc_usd_swap_price_received",
                        message_type = %msg.payload_type,
                        correlation_id = %correlation_id,
                        error = tracing::field::Empty,
                        error.level = tracing::field::Empty,
                        error.message = tracing::field::Empty,
                        funding_action = tracing::field::Empty,
                    );
                    shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                    async {
                        if let Ok(current_position_in_cents) =
                            self.okex_client.get_position_in_signed_usd_cents().await
                        {
                            let _ = self
                                .conditionally_spawn_adjust_funding(
                                    correlation_id,
                                    current_position_in_cents.usd_cents.into(),
                                )
                                .await;
                        }
                    }
                    .instrument(span)
                    .await;
                }
            }
        });
        Ok(())
    }

    async fn spawn_liability_listener(self: Arc<Self>) -> Result<(), HedgingError> {
        job::spawn_adjust_hedge(&self.pool, uuid::Uuid::new_v4()).await?;
        job::spawn_adjust_funding(&self.pool, uuid::Uuid::new_v4()).await?;
        tokio::spawn(async move {
            let mut events = self.ledger.okex_liability_balance_events().await;
            loop {
                match events.recv().await {
                    Ok(received) => {
                        if let ledger::LedgerEventData::BalanceUpdated(data) = received.data {
                            let correlation_id = data.entry_id;
                            let span = info_span!(
                                parent: &received.span,
                                "hedging.okex.usd_liability_balance_event_received",
                                correlation_id = %correlation_id,
                                event_json = &tracing::field::display(
                                    serde_json::to_string(&data)
                                        .expect("failed to serialize event data")
                                ),
                                funding_action = tracing::field::Empty,
                                hedging_action = tracing::field::Empty,
                            );
                            async {
                                if let Ok(current_position_in_cents) =
                                    self.okex_client.get_position_in_signed_usd_cents().await
                                {
                                    let exposure = current_position_in_cents.usd_cents.into();
                                    let _ = self
                                        .conditionally_spawn_adjust_hedge(correlation_id, exposure)
                                        .await;
                                    let _ = self
                                        .conditionally_spawn_adjust_funding(
                                            correlation_id,
                                            exposure,
                                        )
                                        .await;
                                } else {
                                    let _ =
                                        job::spawn_adjust_hedge(&self.pool, correlation_id).await;
                                    let _ =
                                        job::spawn_adjust_funding(&self.pool, correlation_id).await;
                                }
                            }
                            .instrument(span)
                            .await;
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

    async fn spawn_position_listener(
        self: Arc<Self>,
        config: PubSubConfig,
    ) -> Result<Subscriber, HedgingError> {
        let mut subscriber = Subscriber::new(config).await?;
        let mut stream = subscriber
            .subscribe::<OkexBtcUsdSwapPositionPayload>()
            .await?;
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let correlation_id = msg.meta.correlation_id;
                let span = info_span!(
                    "hedging.okex.okex_btc_usd_swap_position_received",
                    message_type = %msg.payload_type,
                    correlation_id = %correlation_id,
                    signed_usd_exposure = %msg.payload.signed_usd_exposure,
                    error = tracing::field::Empty,
                    error.level = tracing::field::Empty,
                    error.message = tracing::field::Empty,
                    hedging_action = tracing::field::Empty,
                    funding_action = tracing::field::Empty,
                );
                shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                async {
                    let _ = self
                        .conditionally_spawn_adjust_hedge(
                            correlation_id,
                            msg.payload.signed_usd_exposure,
                        )
                        .await;
                    let _ = self
                        .conditionally_spawn_adjust_funding(
                            correlation_id,
                            msg.payload.signed_usd_exposure,
                        )
                        .await;
                }
                .instrument(span)
                .await;
            }
        });
        Ok(subscriber)
    }

    #[instrument(name = "hedging.okex.conditionally_spawn_adjust_hedge", skip(self))]
    async fn conditionally_spawn_adjust_hedge(
        &self,
        correlation_id: impl Into<uuid::Uuid> + std::fmt::Debug,
        signed_usd_exposure: SyntheticCentExposure,
    ) -> Result<(), HedgingError> {
        let amount = self
            .ledger
            .balances()
            .okex_target_liability_in_cents()
            .await?;
        let action = self
            .hedging_adjustment
            .determine_action(amount, signed_usd_exposure);
        tracing::Span::current().record("hedging_action", &tracing::field::display(&action));
        if action.action_required() {
            job::spawn_adjust_hedge(&self.pool, correlation_id).await?;
        }
        Ok(())
    }

    #[instrument(name = "hedging.okex.conditionally_spawn_adjust_funding", skip(self))]
    async fn conditionally_spawn_adjust_funding(
        &self,
        correlation_id: impl Into<uuid::Uuid> + std::fmt::Debug,
        signed_usd_exposure: SyntheticCentExposure,
    ) -> Result<(), HedgingError> {
        let target_liability_in_cents = self
            .ledger
            .balances()
            .okex_target_liability_in_cents()
            .await?;
        let last_price_in_usd_cents = self
            .okex_client
            .get_last_price_in_usd_cents()
            .await?
            .usd_cents;
        let trading_available_balance = self.okex_client.trading_account_balance().await?;
        let funding_available_balance = self.okex_client.funding_account_balance().await?;

        let action = self.funding_adjustment.determine_action(
            target_liability_in_cents,
            signed_usd_exposure,
            trading_available_balance.total_amt_in_btc,
            last_price_in_usd_cents,
            funding_available_balance.total_amt_in_btc,
        );
        tracing::Span::current().record("funding_action", &tracing::field::display(&action));
        if action.action_required() {
            job::spawn_adjust_funding(&self.pool, correlation_id).await?;
        }
        Ok(())
    }

    async fn spawn_non_stop_polling(self: Arc<Self>) -> Result<(), HedgingError> {
        tokio::spawn(async move {
            loop {
                let _ = job::spawn_poll_okex(&self.pool, std::time::Duration::from_secs(1)).await;
                tokio::time::sleep(self.config.poll_frequency).await;
            }
        });
        Ok(())
    }
}
