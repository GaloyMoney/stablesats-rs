use rust_decimal::Decimal;
use sqlx::{Executor, PgPool};
use uuid::Uuid;

use okex_client::{ClientTransferId, TransferState};
use shared::pubsub::CorrelationId;

use crate::error::HedgingError;

pub struct ReservationSharedData {
    pub correlation_id: CorrelationId,
    pub action_type: String,
    pub action_unit: String,
    pub target_usd_exposure: Decimal,
    pub current_usd_exposure: Decimal,
    pub trading_btc_used_balance: Decimal,
    pub trading_btc_total_balance: Decimal,
    pub current_usd_btc_price: Decimal,
    pub funding_btc_total_balance: Decimal,
}

pub struct Reservation<'a> {
    pub action_size: Option<Decimal>,
    pub transfer_type: String,
    pub fee: Decimal,
    pub transfer_from: String,
    pub transfer_to: String,
    pub shared: &'a ReservationSharedData,
}

#[derive(Clone)]
pub struct OkexTransfers {
    pool: PgPool,
}

impl OkexTransfers {
    pub async fn new(pool: PgPool) -> Result<Self, HedgingError> {
        Ok(Self { pool })
    }

    pub async fn reserve_transfer_slot<'a>(
        &self,
        reservation: Reservation<'a>,
    ) -> Result<Option<ClientTransferId>, HedgingError> {
        let mut tx = self.pool.begin().await?;
        tx.execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .await?;
        let res = sqlx::query!(
            r#"SELECT client_transfer_id FROM okex_transfers WHERE state = 'pending' AND lost = false AND transfer_type = $1"#,
            reservation.transfer_type,
        )
        .fetch_all(&mut tx)
        .await?;

        if !res.is_empty() {
            return Ok(None);
        }
        let id = ClientTransferId::new();
        sqlx::query!(
            r#"INSERT INTO okex_transfers (
                client_transfer_id, 
                correlation_id, 
                action, 
                transfer_type, 
                currency,
                amount,
                fee,
                transfer_from,
                transfer_to,
                target_usd_exposure,
                current_usd_exposure,
                trading_btc_used_balance,
                trading_btc_total_balance,
                current_usd_btc_price,
                funding_btc_total_balance,
                state
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)"#,
            String::from(id.clone()),
            Uuid::from(reservation.shared.correlation_id),
            reservation.shared.action_type,
            reservation.transfer_type,
            reservation.shared.action_unit,
            reservation.action_size,
            reservation.fee,
            reservation.transfer_from,
            reservation.transfer_to,
            reservation.shared.target_usd_exposure,
            reservation.shared.current_usd_exposure,
            reservation.shared.trading_btc_used_balance,
            reservation.shared.trading_btc_total_balance,
            reservation.shared.current_usd_btc_price,
            reservation.shared.funding_btc_total_balance,
            "pending"
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(Some(id))
    }

    pub async fn open_external_deposit(
        &self,
    ) -> Result<
        Vec<(
            ClientTransferId,
            String,
            Decimal,
            chrono::DateTime<chrono::Utc>,
        )>,
        HedgingError,
    > {
        let res =
            sqlx::query!(r#"SELECT client_transfer_id, transfer_to, amount, created_at FROM okex_transfers WHERE action = 'deposit' AND transfer_type = 'external' AND state = 'pending'"#)
                .fetch_all(&self.pool)
                .await?;
        Ok(res
            .into_iter()
            .map(|r| {
                (
                    ClientTransferId::from(r.client_transfer_id),
                    r.transfer_to.unwrap_or_default(),
                    r.amount,
                    r.created_at,
                )
            })
            .collect())
    }

    pub async fn update_external_deposit(
        &self,
        client_id: ClientTransferId,
        state: String,
        transfer_id: String,
    ) -> Result<(), HedgingError> {
        sqlx::query!(
            r#"UPDATE okex_transfers SET lost = false, transfer_id = $1, state = $2 WHERE client_transfer_id = $3"#,
            transfer_id,
            state,
            String::from(client_id),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn open_non_external_deposit(&self) -> Result<Vec<ClientTransferId>, HedgingError> {
        let res =
            sqlx::query!(r#"SELECT client_transfer_id FROM okex_transfers WHERE (action != 'deposit' OR transfer_type != 'external') AND state = 'pending'"#)
                .fetch_all(&self.pool)
                .await?;
        Ok(res
            .into_iter()
            .map(|r| ClientTransferId::from(r.client_transfer_id))
            .collect())
    }

    pub async fn update_non_external_deposit(
        &self,
        details: TransferState,
    ) -> Result<(), HedgingError> {
        sqlx::query!(
            r#"UPDATE okex_transfers SET lost = false, transfer_id = $1, state = $2 WHERE client_transfer_id = $3"#,
            details.transfer_id,
            details.state,
            String::from(details.client_id),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_as_lost(&self, id: ClientTransferId) -> Result<(), HedgingError> {
        sqlx::query!(
            r#"UPDATE okex_transfers SET lost = true WHERE client_transfer_id = $1"#,
            String::from(id),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn sweep_lost_records(&self) -> Result<(), HedgingError> {
        sqlx::query!(
            r#"UPDATE okex_transfers SET state = 'deleted' WHERE lost = true AND state = 'pending' AND created_at < now() - interval '1 day'"#
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
