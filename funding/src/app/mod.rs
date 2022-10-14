mod config;

use futures::stream::StreamExt;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlxmq::OwnedHandle;
use tracing::{info_span, Instrument};

use galoy_client::*;
use okex_client::*;
use shared::{
    health::HealthCheckTrigger,
    payload::OkexBtcUsdSwapPricePayload,
    pubsub::{CorrelationId, PubSubConfig, Publisher, Subscriber},
};

use crate::{error::*, job, okex_transfers::*, rebalance_action, synth_usd_liability::*};

pub use config::*;

pub struct FundingApp {
    _runner: OwnedHandle,
}

impl FundingApp {
    pub async fn run(
        health_check_trigger: HealthCheckTrigger,
        FundingAppConfig {
            pg_con,
            migrate_on_start,
            okex_poll_frequency: okex_poll_delay,
        }: FundingAppConfig,
        okex_client_config: OkexClientConfig,
        galoy_client_cfg: GaloyClientConfig,
        pubsub_config: PubSubConfig,
    ) -> Result<Self, FundingError> {
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        if migrate_on_start {
            sqlx::migrate!().run(&pool).await?;
        }
        let synth_usd_liability = SynthUsdLiability::new(pool.clone());
        let okex_transfers = OkexTransfers::new(pool.clone()).await?;
        let okex = OkexClient::new(okex_client_config).await?;
        let job_runner = job::start_job_runner(
            pool.clone(),
            synth_usd_liability.clone(),
            okex.clone(),
            okex_transfers,
            GaloyClient::connect(galoy_client_cfg).await?,
            Publisher::new(pubsub_config.clone()).await?,
            okex_poll_delay,
        )
        .await?;
        let price_sub =
            Self::spawn_okex_price_listener(pubsub_config, pool.clone(), synth_usd_liability, okex)
                .await?;
        Self::spawn_health_checker(health_check_trigger, price_sub).await;
        let app = FundingApp {
            _runner: job_runner,
        };
        Ok(app)
    }

    async fn spawn_okex_price_listener(
        config: PubSubConfig,
        pool: sqlx::PgPool,
        synth_usd_liability: SynthUsdLiability,
        okex: OkexClient,
    ) -> Result<Subscriber, FundingError> {
        let subscriber = Subscriber::new(config).await?;
        let mut stream = subscriber.subscribe::<OkexBtcUsdSwapPricePayload>().await?;
        let _ = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
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
                )
                .instrument(span)
                .await;
            }
        });
        Ok(subscriber)
    }

    async fn handle_received_okex_price(
        payload: OkexBtcUsdSwapPricePayload,
        correlation_id: CorrelationId,
        pool: &sqlx::PgPool,
        synth_usd_liability: &SynthUsdLiability,
        okex: &OkexClient,
    ) -> Result<(), FundingError> {
        let target_liability = synth_usd_liability.get_latest_liability().await?;
        let current_position = okex.get_position_in_signed_usd_cents().await?.usd_cents;
        let _funding_available_balance = okex.funding_account_balance().await?;
        let trading_available_balance = okex.trading_account_balance().await?;

        //
        // TODO: streamline this conversion & verify all units match
        //
        let mid_price: Decimal =
            (payload.bid_price.numerator_amount() + payload.ask_price.numerator_amount()) / dec!(2);
        if rebalance_action::determine_action(
            target_liability,
            current_position.into(),
            trading_available_balance.used_amt_in_btc,
            trading_available_balance.total_amt_in_btc,
            mid_price,
        )
        .action_required()
        {
            job::spawn_adjust_funding(pool, correlation_id).await?;
        }
        Ok(())
    }

    async fn spawn_health_checker(
        mut health_check_trigger: HealthCheckTrigger,
        price_sub: Subscriber,
    ) {
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.next().await {
                let duration = chrono::Duration::seconds(20);
                match price_sub.healthy(duration).await {
                    Err(e) => check.send(Err(e)).expect("Couldn't send response"),
                    _ => check.send(Ok(())).expect("Couldn't send response"),
                }
            }
        });
    }
}
