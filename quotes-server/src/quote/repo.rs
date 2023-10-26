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
            r#"INSERT INTO stablesats_quotes (id)
               VALUES ($1)"#,
            quote.id as QuoteId
        )
        .execute(&mut *tx)
        .await?;
        let res = Quote {
            id: quote.id,
            direction: quote.direction.clone(),
            immediate_execution: quote.immediate_execution,
            sat_amount: quote.sat_amount.clone(),
            cent_amount: quote.cent_amount.clone(),
            expires_at: quote.expires_at,
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

    pub async fn find_by_id(&self, id: QuoteId) -> Result<Quote, QuoteError> {
        let rows = sqlx::query!(
            r#"
                SELECT q.id, e.sequence, e.event
                FROM stablesats_quotes q
                JOIN stablesats_quote_events e ON q.id = e.id
                WHERE q.id = $1
                ORDER BY q.created_at, q.id, e.sequence
            "#,
            id as QuoteId
        )
        .fetch_all(&self.pool)
        .await?;

        let mut entity_events = EntityEvents::new();
        for row in rows {
            entity_events.load_event(row.sequence as usize, row.event)?;
        }

        Ok(Quote::try_from(entity_events)?)
    }

    pub async fn update(&self, quote: Quote) -> Result<(), QuoteError> {
        if !quote.events.is_dirty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;
        EntityEvents::<QuoteEvent>::persist(
            "stablesats_quote_events",
            &mut tx,
            quote.events.new_serialized_events(quote.id),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
