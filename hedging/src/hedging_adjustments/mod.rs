mod instruments;

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use shared::pubsub::CorrelationId;

use crate::{adjustment_action::AdjustmentAction, error::HedgingError};

use instruments::*;

pub struct Adjustment {
    pub correlation_id: CorrelationId,
    pub exchange_ref: Option<String>,
    pub action: AdjustmentAction,
    pub target_usd_value: Decimal,
    pub usd_value_before_adjustment: Decimal,
    pub usd_value_after_adjustment: Decimal,
}

#[derive(Clone)]
pub struct HedgingAdjustments {
    pool: PgPool,
    instruments: HedgingInstruments,
}

impl HedgingAdjustments {
    pub async fn new(pool: PgPool) -> Result<Self, HedgingError> {
        Ok(Self {
            instruments: HedgingInstruments::load(&pool).await?,
            pool,
        })
    }

    pub async fn persist(
        &self,
        Adjustment {
            correlation_id,
            exchange_ref,
            action,
            target_usd_value,
            usd_value_before_adjustment,
            usd_value_after_adjustment,
        }: Adjustment,
    ) -> Result<(), HedgingError> {
        sqlx::query!(
            r#"INSERT INTO hedging_adjustments (
              correlation_id, instrument_id, exchange_ref, action, size, unit, size_usd_value,
              target_usd_value, position_usd_value_before_adjustment, position_usd_value_after_adjustment
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            Uuid::from(correlation_id),
            self.instruments.get_id(HedgingInstrument::OkexBtcUsdSwap),
            exchange_ref,
            action.action_type(),
            action.size().map(Decimal::from),
            action.unit(),
            action.size_in_usd(),
            target_usd_value,
            usd_value_before_adjustment,
            usd_value_after_adjustment,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
