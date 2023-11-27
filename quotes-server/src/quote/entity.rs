use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{currency::*, entity::*};

use super::QuoteError;

crate::entity_id!(QuoteId);

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
        sats_spread: Satoshis,
        cents_spread: UsdCents,
        expires_at: DateTime<Utc>,
    },
    Accepted {
        accepted_at: DateTime<Utc>,
    },
}

#[derive(Builder, Debug)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Quote {
    pub id: QuoteId,
    pub direction: Direction,
    pub sat_amount: Satoshis,
    pub cent_amount: UsdCents,
    pub sats_spread: Satoshis,
    pub cents_spread: UsdCents,
    pub immediate_execution: bool,
    pub expires_at: DateTime<Utc>,

    pub(super) events: EntityEvents<QuoteEvent>,
}

impl Quote {
    pub fn is_accepted(&self) -> bool {
        for event in self.events.iter() {
            if let QuoteEvent::Accepted { .. } = event {
                return true;
            }
        }
        false
    }

    pub fn accept(&mut self) -> Result<(), QuoteError> {
        if self.is_accepted() {
            return Err(QuoteError::QuoteAlreadyAccepted);
        }
        if self.is_expired() {
            return Err(QuoteError::QuoteExpiredError);
        }
        self.events.push(QuoteEvent::Accepted {
            accepted_at: Utc::now(),
        });
        Ok(())
    }

    fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    pub fn accepted_at(&self) -> Option<DateTime<Utc>> {
        for event in self.events.iter() {
            if let QuoteEvent::Accepted { accepted_at } = event {
                return Some(*accepted_at);
            }
        }
        None
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
    pub(super) sats_spread: Satoshis,
    pub(super) cents_spread: UsdCents,
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
            sats_spread: self.sats_spread,
            cents_spread: self.cents_spread,
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
                sats_spread,
                cents_spread,
                expires_at,
            } = event
            {
                builder = builder
                    .id(*id)
                    .direction(direction.clone())
                    .immediate_execution(*immediate_execution)
                    .sat_amount(*sat_amount)
                    .cent_amount(*cent_amount)
                    .sats_spread(*sats_spread)
                    .cents_spread(*cents_spread)
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use rust_decimal::Decimal;

    fn init_events(expired: bool) -> EntityEvents<QuoteEvent> {
        let expiration_interval = Duration::from_std(std::time::Duration::from_secs(120)) // 2 minutes = 120 seconds
            .unwrap();
        let expiration_time = if !expired {
            Utc::now() + Duration::from_std(expiration_interval.to_std().unwrap()).unwrap()
        } else {
            Utc::now() - Duration::from_std(expiration_interval.to_std().unwrap()).unwrap()
        };
        EntityEvents::init([QuoteEvent::Initialized {
            id: QuoteId::new(),
            direction: Direction::BuyCents,
            immediate_execution: false,
            sat_amount: Satoshis::from(Decimal::from(1000)),
            cent_amount: UsdCents::from(Decimal::from(100)),
            sats_spread: Satoshis::from(Decimal::from(10)),
            cents_spread: UsdCents::from(Decimal::from(1)),
            expires_at: expiration_time,
        }])
    }

    #[test]
    fn accept_quote() {
        let events = init_events(false);
        let mut quote = Quote::try_from(events).unwrap();
        assert!(quote.accept().is_ok());
        assert!(matches!(
            quote.events.last(1)[0],
            QuoteEvent::Accepted { .. }
        ));
    }

    #[test]
    fn can_only_accept_quote_once() {
        let mut events = init_events(false);
        events.push(QuoteEvent::Accepted {
            accepted_at: Utc::now(),
        });
        let mut quote = Quote::try_from(events).unwrap();
        assert!(matches!(
            quote.accept(),
            Err(QuoteError::QuoteAlreadyAccepted)
        ));
    }

    #[test]
    fn cannot_accept_expired_quote() {
        let events = init_events(true);
        let mut quote = Quote::try_from(events).unwrap();
        assert!(matches!(quote.accept(), Err(QuoteError::QuoteExpiredError)));
    }
}
