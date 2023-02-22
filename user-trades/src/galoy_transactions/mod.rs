use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};

use crate::error::UserTradesError;
use galoy_client::{GaloyTransaction, SettlementCurrency};

pub struct LatestCursor(pub String);

#[derive(Debug, Clone)]
pub struct UnpairedTransaction {
    pub id: String,
    pub settlement_amount: Decimal,
    pub settlement_currency: SettlementCurrency,
    pub direction: String,
    pub settlement_method: String,
    pub memo: Option<String>,
    pub amount_in_usd_cents: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct UnpairedTransactions<'a> {
    pub list: Vec<UnpairedTransaction>,
    pub tx: Transaction<'a, Postgres>,
}

#[derive(Clone)]
pub struct GaloyTransactions {
    pool: PgPool,
}
impl GaloyTransactions {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn persist_all(
        &self,
        transactions: Vec<GaloyTransaction>,
    ) -> Result<(), UserTradesError> {
        if transactions.is_empty() {
            return Ok(());
        }
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO galoy_transactions (id, cursor, is_paired, settlement_amount, settlement_currency, settlement_method, cents_per_unit, amount_in_usd_cents, created_at, memo, direction)"
        );
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
                 memo,
                 direction,
                 status: _,
             }| {
                builder.push_bind(id);
                builder.push_bind(String::from(cursor));
                builder.push_bind(false);
                builder.push_bind(settlement_amount);
                builder.push_bind(settlement_currency.to_string());
                builder.push_bind(settlement_method.to_string());
                builder.push_bind(cents_per_unit);
                builder.push_bind(amount_in_usd_cents);
                builder.push_bind(created_at);
                builder.push_bind(memo);
                builder.push_bind(direction.to_string());
            },
        );
        query_builder.push("ON CONFLICT DO NOTHING");
        let query = query_builder.build();
        query.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_latest_cursor(&self) -> Result<Option<LatestCursor>, UserTradesError> {
        let res =
            sqlx::query!("SELECT cursor FROM galoy_transactions ORDER BY created_at DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;

        if let Some(res) = res {
            Ok(Some(LatestCursor(res.cursor)))
        } else {
            Ok(None)
        }
    }

    pub async fn list_unpaired_transactions(
        &self,
    ) -> Result<UnpairedTransactions, UserTradesError> {
        let mut tx = self.pool.begin().await?;
        let res = sqlx::query!(
            "
            SELECT id, direction, amount_in_usd_cents, memo, settlement_method, settlement_amount, settlement_currency, created_at
            FROM galoy_transactions
            WHERE is_paired = false AND amount_in_usd_cents != 0 ORDER BY created_at FOR UPDATE
         "
        )
        .fetch_all(&mut tx)
        .await?;
        Ok(UnpairedTransactions {
            list: res
                .into_iter()
                .map(|res| UnpairedTransaction {
                    id: res.id,
                    settlement_amount: res.settlement_amount,
                    settlement_currency: res
                        .settlement_currency
                        .parse()
                        .expect("Couldn't parse settlement currency"),
                    direction: res.direction,
                    memo: res.memo,
                    amount_in_usd_cents: res.amount_in_usd_cents,
                    settlement_method: res.settlement_method,
                    created_at: res.created_at,
                })
                .collect(),
            tx,
        })
    }

    pub async fn update_paired_ids<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        ids: &[String],
    ) -> Result<(), UserTradesError> {
        if ids.is_empty() {
            return Ok(());
        }
        sqlx::query!(
            "UPDATE galoy_transactions SET is_paired = 'true' WHERE id = ANY($1)",
            &ids[..]
        )
        .execute(tx)
        .await?;
        Ok(())
    }
}
