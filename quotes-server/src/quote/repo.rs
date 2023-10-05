use rust_decimal::prelude::ToPrimitive;
use sqlx::{Pool, Postgres};
use tracing::instrument;

use crate::entity::*;

use super::{entity::*, error::QuoteError};

#[derive(Debug, Clone)]
pub struct Quotes {
    pool: Pool<Postgres>,
}

impl Quotes {
    pub fn new(pool: &Pool<Postgres>) -> Self {
        Self { pool: pool.clone() }
    }

    #[instrument(name = "quotes.create", skip(self))]
    pub async fn create(&self, quote: NewQuote) -> Result<Quote, QuoteError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
        r#"
        INSERT INTO stablesats_quote (id, direction, immediate_execution, sat_amount, cent_amount, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        quote.id as QuoteId,
        pg::PgDirection::from(quote.direction.clone()) as pg::PgDirection,
        quote.immediate_execution,
        quote.sat_amount.amount().to_i64(),
        quote.cent_amount.amount().to_i64(),
        quote.expires_at
    )
            .execute(&mut *tx)
            .await?;
        let res = Quote {
            id: quote.id,
            direction: quote.direction.clone(),
            immediate_execution: quote.immediate_execution,
            sat_amount: quote.sat_amount.clone(),
            cent_amount: quote.cent_amount.clone(),
            events: quote.clone().initial_events(),
        };

        EntityEvents::<QuoteEvent>::persist(
            "stablesats_quote_events",
            &mut tx,
            quote.initial_events().new_serialized_events(res.id),
        )
        .await?;

        tx.commit().await?;
        Ok(res)
    }
}
