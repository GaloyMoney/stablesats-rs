use sqlx::{Pool, Postgres, Transaction};
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
    pub async fn create(
        &self,
        mut tx: &mut Transaction<'_, Postgres>,
        quote: NewQuote,
    ) -> Result<Quote, QuoteError> {
        sqlx::query!(
            r#"INSERT INTO stablesats_quotes (id)
               VALUES ($1)"#,
            quote.id as QuoteId
        )
        .execute(&mut **tx)
        .await?;

        let id = quote.id;
        let mut initial_events = quote.initial_events();

        EntityEvents::<QuoteEvent>::persist(
            "stablesats_quote_events",
            &mut tx,
            initial_events.new_serialized_events(id),
        )
        .await?;

        initial_events.mark_persisted();
        let res = Quote::try_from(initial_events).expect("initial events should be valid");

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

    pub async fn update(
        &self,
        quote: &Quote,
        mut tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(), QuoteError> {
        if !quote.events.is_dirty() {
            return Ok(());
        }
        EntityEvents::<QuoteEvent>::persist(
            "stablesats_quote_events",
            &mut tx,
            quote.events.new_serialized_events(quote.id),
        )
        .await?;
        Ok(())
    }
}
