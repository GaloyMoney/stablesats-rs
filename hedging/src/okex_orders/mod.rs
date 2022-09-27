mod instruments;

use rust_decimal::Decimal;
use sqlx::{Executor, PgPool};
use uuid::Uuid;

use okex_client::{ClientOrderId, OrderDetails};
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
        let res = sqlx::query!(
            r#"SELECT client_order_id FROM okex_orders WHERE complete = false AND lost = false"#
        )
        .fetch_all(&mut tx)
        .await?;

        if !res.is_empty() {
            return Ok(None);
        }
        let id = ClientOrderId::new();
        sqlx::query!(
            r#"INSERT INTO okex_orders (
              client_order_id, correlation_id, instrument,
              action, size, unit, size_usd_value, target_usd_value,
              position_usd_value_before_order
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            String::from(id.clone()),
            Uuid::from(reservation.correlation_id),
            "BTC-USD-SWAP",
            reservation.action.action_type(),
            reservation.action.size().map(Decimal::from),
            reservation.action.unit(),
            reservation.action.size_in_usd(),
            reservation.target_usd_value,
            reservation.usd_value_before_order,
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(Some(id))
    }

    pub async fn open_orders(&self) -> Result<Vec<ClientOrderId>, HedgingError> {
        let res = sqlx::query!(r#"SELECT client_order_id FROM okex_orders WHERE complete = false"#)
            .fetch_all(&self.pool)
            .await?;
        Ok(res
            .into_iter()
            .map(|r| ClientOrderId::from(r.client_order_id))
            .collect())
    }

    pub async fn update_order(&self, details: OrderDetails) -> Result<(), HedgingError> {
        sqlx::query!(
            r#"UPDATE okex_orders SET lost = false, order_id = $1, avg_price = $2, fee = $3, state = $4, complete = $5 WHERE client_order_id = $6"#,
            details.ord_id,
            details.avg_px,
            details.fee,
            details.state,
            details.complete,
            String::from(details.cl_ord_id),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_as_lost(&self, id: ClientOrderId) -> Result<(), HedgingError> {
        sqlx::query!(
            r#"UPDATE okex_orders SET lost = true WHERE client_order_id = $1"#,
            String::from(id),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
