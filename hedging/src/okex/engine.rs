use sqlxmq::NamedJob;

use okex_client::OkexClient;

use super::{config::*, funding_adjustment::*, hedge_adjustment::*, job, orders::*, transfers::*};
use crate::error::HedgingError;

pub struct OkexEngine {
    config: OkexConfig,
    orders: OkexOrders,
    transfers: OkexTransfers,
    okex_client: OkexClient,
}

impl OkexEngine {
    pub async fn init(pool: sqlx::PgPool, config: OkexConfig) -> Result<Self, HedgingError> {
        let okex_client = OkexClient::new(config.client.clone()).await?;
        let orders = OkexOrders::new(pool.clone()).await?;
        let transfers = OkexTransfers::new(pool).await?;
        okex_client
            .check_leverage(config.funding.high_bound_ratio_leverage)
            .await?;
        Ok(Self {
            config,
            okex_client,
            orders,
            transfers,
        })
    }

    pub fn add_context_to_job_registry(&self, runner: &mut sqlxmq::JobRegistry) {
        runner.set_context(self.okex_client.clone());
        runner.set_context(self.orders.clone());
        runner.set_context(self.transfers.clone());
        runner.set_context(job::OkexPollDelay(self.config.poll_frequency));
        let funding_adjustment =
            FundingAdjustment::new(self.config.funding.clone(), self.config.hedging.clone());
        runner.set_context(funding_adjustment);
        let hedging_adjustment = HedgingAdjustment::new(self.config.hedging.clone());
        runner.set_context(hedging_adjustment);
        runner.set_context(self.config.funding.clone());
    }

    pub fn register_jobs(jobs: &mut Vec<&'static NamedJob>, channels: &mut Vec<&str>) {
        jobs.push(job::adjust_hedge);
        jobs.push(job::poll_okex);
        jobs.push(job::adjust_funding);
        channels.push("hedging.okex");
    }
}
