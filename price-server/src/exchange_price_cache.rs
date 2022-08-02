use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use shared::{money::*, payload::*, time::*};

#[derive(Error, Debug)]
pub enum ExchangePriceCacheError {
    #[error("UnexpectedMessage: {0}")]
    UnexpectedMessage(String),
}

#[derive(Clone)]
pub struct ExchangePriceCache {
    inner: Arc<RwLock<ExchangePriceCacheInner>>,
}

impl ExchangePriceCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ExchangePriceCacheInner::new())),
        }
    }

    pub async fn apply_update(
        &self,
        payload: OkexBtcUsdSwapPricePayload,
    ) -> Result<(), ExchangePriceCacheError> {
        self.inner.write().await.update_price(payload).await;
        Ok(())
    }

    //     pub async fn get_price(&self, exchange: Exchange) -> Option<Money> {
    //         self.inner.read().await.get_price(exchange)
    //     }
}

struct ExchangePriceCacheInner {
    last_update: Option<TimeStamp>,
}

impl ExchangePriceCacheInner {
    fn new() -> Self {
        Self { last_update: None }
    }

    async fn update_price(&mut self, payload: impl Into<PriceMessagePayload>) {
        let payload = payload.into();
        if let Some(ref last_update) = self.last_update {
            if last_update > &payload.timestamp {
                return;
            }
        }
        self.last_update = Some(payload.timestamp);
    }
}
