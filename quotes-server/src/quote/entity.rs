use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::currency::*;

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
}

pub struct Quote {
    pub id: QuoteId,
}
