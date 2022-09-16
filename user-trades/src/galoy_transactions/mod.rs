use sqlx::{Executor, PgPool, Postgres, QueryBuilder, Transaction};

use crate::error::UserTradesError;
use galoy_client::GaloyTransaction;

pub struct LatestCursor<'a> {
    id: Option<String>,
    cursor: Option<String>,
    tx: Transaction<'a, Postgres>,
}
impl<'a> LatestCursor<'a> {
    pub fn take(&mut self) -> Option<String> {
        self.cursor.take()
    }
}

pub struct UnpairedTransaction {
    created_at: chrono::DateTime<chrono::Utc>,
}
pub struct UnpairedTransactions<'a> {
    list: Vec<GaloyTransaction>,
    tx: Transaction<'a, Postgres>,
}

#[derive(Clone)]
pub struct GaloyTransactions {
    pool: PgPool,
}
impl GaloyTransactions {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn persist_all<'a>(
        &self,
        LatestCursor {
            id: latest_id,
            mut tx,
            ..
        }: LatestCursor<'a>,
        transactions: Vec<GaloyTransaction>,
    ) -> Result<(), UserTradesError> {
        if transactions.is_empty() {
            return Ok(());
        }
        if let Some(latest_id) = latest_id {
            sqlx::query!(
                "UPDATE galoy_transactions SET is_latest_cursor = NULL WHERE id = $1",
                latest_id
            )
            .execute(&mut tx)
            .await?;
        }
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO galoy_transactions (id, is_latest_cursor, cursor, is_paired, settlement_amount, settlement_currency, settlement_method, cents_per_unit, amount_in_usd_cents, created_at)"
        );
        let latest_cursor = transactions.first().unwrap().cursor.clone();
        query_builder.push_values(
            transactions,
            |mut builder,
             GaloyTransaction {
                 created_at,
                 id,
                 cursor,
                 settlement_amount,
                 settlement_method,
                 settlement_currency,
                 cents_per_unit,
                 amount_in_usd_cents,
                 status: _,
             }| {
                builder.push_bind(id);
                builder.push_bind(if latest_cursor == cursor {
                    Some(true)
                } else {
                    None
                });
                builder.push_bind(String::from(cursor));
                builder.push_bind(false);
                builder.push_bind(settlement_amount);
                builder.push_bind(settlement_currency.to_string());
                builder.push_bind(settlement_method.to_string());
                builder.push_bind(cents_per_unit);
                builder.push_bind(amount_in_usd_cents);
                builder.push_bind(created_at);
            },
        );
        query_builder.push("ON CONFLICT DO NOTHING");
        let query = query_builder.build();
        query.execute(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Uses optimistic locking as described
    /// https://stackoverflow.com/questions/71987836/postgresql-select-for-update-lock-new-rows/71988854#71988854
    pub async fn get_latest_cursor(&self) -> Result<LatestCursor, UserTradesError> {
        let mut tx = self.pool.begin().await?;
        tx.execute("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .await?;
        let res = sqlx::query!(
            "SELECT id, cursor FROM galoy_transactions WHERE is_latest_cursor = 'true'"
        )
        .fetch_all(&mut tx)
        .await?;

        if let Some(res) = res.into_iter().next() {
            Ok(LatestCursor {
                id: Some(res.id),
                cursor: Some(res.cursor),
                tx,
            })
        } else {
            Ok(LatestCursor {
                id: None,
                cursor: None,
                tx,
            })
        }
    }

    pub async fn list_unpaired_transactions(
        &self,
    ) -> Result<UnpairedTransactions, UserTradesError> {
        unimplemented!()
    }
}
