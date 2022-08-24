mod config;

use rust_decimal_macros::dec;
use std::time::Duration;

use crate::{error::*, user_trade::*, user_trade_balances::*, user_trade_unit::*};
pub use config::*;
use shared::{payload::SynthUsdExposurePayload, pubsub::*};

pub struct UserTradesApp {
    _user_trades: UserTrades,
}

impl UserTradesApp {
    pub async fn run(
        UserTradesAppConfig {
            pg_con,
            migrate_on_start,
            publish_frequency,
        }: UserTradesAppConfig,
        pubsub_cfg: PubSubConfig,
    ) -> Result<Self, UserTradesError> {
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        if migrate_on_start {
            sqlx::migrate!().run(&pool).await?;
        }
        let units = UserTradeUnits::load(&pool).await?;
        let user_trade_balances = UserTradeBalances::new(pool.clone(), units.clone()).await?;
        Self::publish_exposure(user_trade_balances, pubsub_cfg, publish_frequency).await?;
        Ok(Self {
            _user_trades: UserTrades::new(pool, units),
        })
    }

    async fn publish_exposure(
        user_trade_balances: UserTradeBalances,
        pubsub_cfg: PubSubConfig,
        publish_frequency: Duration,
    ) -> Result<(), UserTradesError> {
        let pubsub = Publisher::new(pubsub_cfg).await?;
        tokio::spawn(async move {
            loop {
                if let Ok(balances) = user_trade_balances.fetch_all().await {
                    let _ = pubsub
                        .publish(SynthUsdExposurePayload {
                            exposure: balances
                                .get(&UserTradeUnit::SynthCent)
                                .expect("SynthCents should always be present")
                                .current_balance
                                * dec!(-1),
                        })
                        .await;
                    tokio::time::sleep(publish_frequency).await;
                }
            }
        });
        Ok(())
    }
}
