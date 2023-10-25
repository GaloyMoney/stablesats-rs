use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{currency::*, entity::*};

shared::entity_id! { QuoteId }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    Accepted {},
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Quote {
    pub id: QuoteId,
    pub direction: Direction,
    pub sat_amount: Satoshis,
    pub cent_amount: UsdCents,
    pub immediate_execution: bool,
    pub expires_at: DateTime<Utc>,

    pub(super) events: EntityEvents<QuoteEvent>,
}

impl Quote {
    pub fn is_accepted(&self) -> bool {
        for event in self.events.iter() {
            if let QuoteEvent::Accepted {} = event {
                return true;
            }
        }
        false
    }

    pub fn accept(&mut self) {
        self.events.push(QuoteEvent::Accepted {});
    }
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
        EntityEvents::init([QuoteEvent::Initialized {
            id: self.id,
            direction: self.direction,
            immediate_execution: self.immediate_execution,
            sat_amount: self.sat_amount,
            cent_amount: self.cent_amount,
            expires_at: self.expires_at,
        }])
    }
}

impl TryFrom<EntityEvents<QuoteEvent>> for Quote {
    type Error = EntityError;

    fn try_from(events: EntityEvents<QuoteEvent>) -> Result<Self, Self::Error> {
        let mut builder = QuoteBuilder::default();

        for event in events.iter() {
            if let QuoteEvent::Initialized {
                id,
                direction,
                immediate_execution,
                sat_amount,
                cent_amount,
                expires_at,
            } = event
            {
                builder = builder
                    .id(*id)
                    .direction(direction.clone())
                    .immediate_execution(*immediate_execution)
                    .sat_amount(sat_amount.clone())
                    .cent_amount(cent_amount.clone())
                    .expires_at(*expires_at);
            }
        }
        builder.events(events).build()
    }
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
