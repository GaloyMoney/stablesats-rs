use bria_client::{BriaClient, BriaClientConfig};
use futures::stream::StreamExt;
use rust_decimal::Decimal;
use sqlxmq::JobRunnerHandle;
use tracing::instrument;

use galoy_client::*;
use shared::{health::HealthCheckTrigger, payload::PriceStreamPayload, pubsub::memory};

use crate::{config::*, error::*, okex::*};

pub struct HedgingApp {
    _job_runner_handle: JobRunnerHandle,
}

impl HedgingApp {
    #[allow(clippy::too_many_arguments)]
    #[instrument(name = "HedgingApp.run", skip_all, fields(error, error.level, error.message))]
    pub async fn run(
        pool: sqlx::PgPool,
        health_check_trigger: HealthCheckTrigger,
        HedgingAppConfig {
            health: health_cfg, ..
        }: HedgingAppConfig,
        okex_config: OkexConfig,
        galoy_client_cfg: GaloyClientConfig,
        bria_client_cfg: BriaClientConfig,
        price_receiver: memory::Subscriber<PriceStreamPayload>,
    ) -> Result<Self, HedgingError> {
        let (mut jobs, mut channels) = (Vec::new(), Vec::new());
        OkexEngine::register_jobs(&mut jobs, &mut channels);
        let mut job_registry = sqlxmq::JobRegistry::new(&jobs);

        let ledger = ledger::Ledger::init(&pool).await?;
        job_registry.set_context(ledger.clone());
        job_registry.set_context(
            shared::tracing::record_error(tracing::Level::ERROR, || async move {
                GaloyClient::connect(galoy_client_cfg).await
            })
            .await?,
        );
        job_registry.set_context(
            shared::tracing::record_error(tracing::Level::ERROR, || async move {
                BriaClient::connect(bria_client_cfg).await
            })
            .await?,
        );

        let okex_engine = OkexEngine::run(
            pool.clone(),
            okex_config,
            ledger.clone(),
            price_receiver.resubscribe(),
        )
        .await?;

        okex_engine.add_context_to_job_registry(&mut job_registry);

        let job_runner_handle = job_registry
            .runner(&pool)
            .set_channel_names(&channels)
            .run()
            .await?;

        let _ = Self::spawn_global_liability_listener(pool.clone(), ledger).await;
        Self::spawn_health_checker(health_check_trigger, health_cfg, price_receiver).await;
        let app = HedgingApp {
            _job_runner_handle: job_runner_handle,
        };
        Ok(app)
    }

    async fn spawn_health_checker(
        mut health_check_trigger: HealthCheckTrigger,
        health_cfg: HedgingAppHealthConfig,
        price_sub: memory::Subscriber<PriceStreamPayload>,
    ) {
        while let Some(check) = health_check_trigger.next().await {
            match price_sub
                .healthy(health_cfg.unhealthy_msg_interval_price)
                .await
            {
                Err(e) => {
                    let _ = check.send(Err(e));
                }
                _ => {
                    let _ = check.send(Ok(()));
                }
            }
        }
    }

    async fn spawn_global_liability_listener(
        pool: sqlx::PgPool,
        ledger: ledger::Ledger,
    ) -> Result<(), HedgingError> {
        let mut events = ledger.usd_omnibus_balance_events().await?;
        tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(received) => {
                        if let ledger::LedgerEventData::BalanceUpdated(_data) = received.data {
                            let _ = async {
                                let liability_balances =
                                    ledger.balances().usd_liability_balances().await?;
                                let tx = pool.begin().await?;
                                let unallocated_usd = liability_balances.unallocated_usd;
                                if unallocated_usd == Decimal::ZERO {
                                    // no need to do anything
                                } else {
                                    let adjustment_params =
                                        ledger::AdjustExchangeAllocationParams {
                                            okex_allocation_adjustment_usd_cents_amount:
                                                unallocated_usd,
                                            bitfinex_allocation_adjustment_usd_cents_amount:
                                                Decimal::ZERO,
                                            meta: ledger::AdjustExchangeAllocationMeta {
                                                timestamp: chrono::Utc::now(),
                                            },
                                        };
                                    ledger
                                        .adjust_exchange_allocation(tx, adjustment_params)
                                        .await?;
                                }
                                Ok::<(), ledger::LedgerError>(())
                            }
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
}
