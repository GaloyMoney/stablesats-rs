use sqlx::PgPool;

use shared::payload::SyntheticCentLiability;

use crate::error::FundingError;

#[derive(Clone)]
pub struct SynthUsdLiability {
    pool: PgPool,
}

impl SynthUsdLiability {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_latest_liability(&self) -> Result<SyntheticCentLiability, FundingError> {
        let result =
            sqlx::query!("SELECT amount FROM synth_usd_liability ORDER BY idx DESC LIMIT 1")
                .fetch_one(&self.pool)
                .await?;

        Ok(SyntheticCentLiability::try_from(result.amount).expect("invalid liability"))
    }
}
