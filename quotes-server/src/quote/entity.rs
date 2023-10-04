use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::currency::*;
use crate::EntityEvents;

shared::entity_id! { QuoteId }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    BuyCents,
    SellCents,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuoteEvent {
    Initialized {
        id: QuoteId,
        direction: Direction,
        immediate_execution: bool,
        sat_amount: Satoshis,
        cent_amount: UsdCents,
        expires_at: DateTime<Utc>,
    },
}

#[derive(Builder, Clone, Debug)]
pub struct NewQuote {
    #[builder(private)]
    pub(super) id: QuoteId,
    pub(super) direction: Direction,
    pub(super) immediate_execution: bool,
    pub(super) sat_amount: Satoshis,
    pub(super) cent_amount: UsdCents,
    pub(super) expires_at: DateTime<Utc>,
}

impl NewQuote {
    pub fn builder() -> NewQuoteBuilder {
        let mut builder = NewQuoteBuilder::default();
        builder.id(QuoteId::new());
        builder
    }

    pub(super) fn initial_events(self) -> EntityEvents<QuoteEvent> {
        let mut events = EntityEvents::init([QuoteEvent::Initialized {
            id: self.id,
            direction: self.direction,
            immediate_execution: self.immediate_execution,
            sat_amount: self.sat_amount,
            cent_amount: self.cent_amount,
            expires_at: self.expires_at,
        }]);
        events
    }
}

pub struct Quote {
    pub id: QuoteId,
    pub direction: Direction,
    pub immediate_execution: bool,
    pub sat_amount: Satoshis,
    pub cent_amount: UsdCents,

    pub(super) events: EntityEvents<QuoteEvent>,
}

pub mod pg {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
    #[sqlx(type_name = "direction_enum", rename_all = "snake_case")]
    pub enum PgDirection {
        BuyCents,
        SellCents,
    }

    impl From<super::Direction> for PgDirection {
        fn from(direction: super::Direction) -> Self {
            match direction {
                super::Direction::BuyCents => Self::BuyCents,
                super::Direction::SellCents => Self::SellCents,
            }
        }
    }

    impl From<PgDirection> for super::Direction {
        fn from(direction: PgDirection) -> Self {
            match direction {
                PgDirection::BuyCents => Self::BuyCents,
                PgDirection::SellCents => Self::SellCents,
            }
        }
    }
}
