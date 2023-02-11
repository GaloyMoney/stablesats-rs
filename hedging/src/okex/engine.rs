use okex_client::OkexClient;

use super::{orders::OkexOrders, transfers::OkexTransfers};
use crate::error::HedgingError;
use shared::exchanges_config::OkexConfig;

pub struct OkexEngine {}

impl OkexEngine {
    pub async fn start(
        pool: sqlx::PgPool,
        okex_client_config: OkexConfig,
    ) -> Result<Self, HedgingError> {
        let okex = OkexClient::new(okex_client_config).await?;
        // let funding_adjustment =
        //     FundingAdjustment::new(funding_config.clone(), hedging_config.clone());
        okex.check_leverage(funding_config.high_bound_ratio_leverage)
            .await?;
        let okex_orders = OkexOrders::new(pool.clone()).await?;
        let okex_transfers = OkexTransfers::new(pool.clone()).await?;

        Ok(OkexEngine {})
    }
}
