use futures::stream::StreamExt;
use sqlxmq::OwnedHandle;

use galoy_client::*;
use shared::{
    health::HealthCheckTrigger,
    payload::PriceStreamPayload,
    pubsub::{memory, PubSubConfig, Publisher, Subscriber},
};

use crate::{config::*, error::*, okex::*};

pub struct HedgingApp {
    _job_runner_handle: OwnedHandle,
}

impl HedgingApp {
    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        pool: sqlx::PgPool,
        health_check_trigger: HealthCheckTrigger,
        HedgingAppConfig {
            health: health_cfg, ..
        }: HedgingAppConfig,
        okex_config: OkexConfig,
        galoy_client_cfg: GaloyClientConfig,
        pubsub_config: PubSubConfig,
        price_receiver: memory::Subscriber<PriceStreamPayload>,
    ) -> Result<Self, HedgingError> {
        let (mut jobs, mut channels) = (Vec::new(), Vec::new());
        OkexEngine::register_jobs(&mut jobs, &mut channels);

        let mut job_registry = sqlxmq::JobRegistry::new(&jobs);

        let ledger = ledger::Ledger::init(&pool).await?;
        job_registry.set_context(ledger.clone());
        job_registry.set_context(GaloyClient::connect(galoy_client_cfg).await?);
        job_registry.set_context(Publisher::new(pubsub_config.clone()).await?);

        let (okex_engine, subscriber) = OkexEngine::run(
            pool.clone(),
            okex_config,
            ledger,
            pubsub_config,
            price_receiver.resubscribe(),
        )
        .await?;

        okex_engine.add_context_to_job_registry(&mut job_registry);

        let job_runner_handle = job_registry
            .runner(&pool)
            .set_channel_names(&channels)
            .run()
            .await?;

        Self::spawn_health_checker(health_check_trigger, health_cfg, subscriber, price_receiver)
            .await;
        let app = HedgingApp {
            _job_runner_handle: job_runner_handle,
        };
        Ok(app)
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
                (Err(e), _) | (_, Err(e)) => {
                    let _ = check.send(Err(e));
                }
                _ => {
                    let _ = check.send(Ok(()));
                }
            }
        }
    }
}
