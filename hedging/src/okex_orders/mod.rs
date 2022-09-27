mod instruments;

use rust_decimal::Decimal;
use sqlx::{Executor, PgPool};
use uuid::Uuid;

use okex_client::ClientOrderId;
use shared::pubsub::CorrelationId;

use crate::{adjustment_action::AdjustmentAction, error::HedgingError};

use instruments::*;

pub struct Reservation<'a> {
    pub correlation_id: CorrelationId,
    pub action: &'a AdjustmentAction,
    pub target_usd_value: Decimal,
    pub usd_value_before_order: Decimal,
}

pub struct Adjustment {
    pub correlation_id: CorrelationId,
    pub exchange_ref: Option<String>,
    pub action: AdjustmentAction,
    pub target_usd_value: Decimal,
    pub usd_value_before_adjustment: Decimal,
    pub usd_value_after_adjustment: Decimal,
}

#[derive(Clone)]
pub struct OkexOrders {
    pool: PgPool,
    instruments: HedgingInstruments,
}

impl OkexOrders {
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

    pub async fn reserve_order_slot<'a>(
        &self,
        reservation: Reservation<'a>,
    ) -> Result<Option<ClientOrderId>, HedgingError> {
        let mut tx = self.pool.begin().await?;
        tx.execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .await?;
        let res =
            sqlx::query!(r#"SELECT client_order_id FROM okex_orders WHERE completed = false"#)
                .fetch_all(&mut tx)
                .await?;

        if !res.is_empty() {
            return Ok(None);
        }
        let id = ClientOrderId::new();
        sqlx::query!(
            r#"INSERT INTO okex_orders (
            client_order_id, correlation_id, instrument_id,
            action, size, unit, size_usd_value, target_usd_value,
            position_usd_value_before_order
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            String::from(id),
            Uuid::from(reservation.correlation_id),
            self.instruments.get_id(HedgingInstrument::OkexBtcUsdSwap),
            reservation.action.action_type(),
            reservation.action.size().map(Decimal::from),
            reservation.action.unit(),
            reservation.action.size_in_usd(),
            reservation.target_usd_value,
            reservation.usd_value_before_order,
        )
        .execute(&mut tx)
        .await?;
        unimplemented!()
    }
}
