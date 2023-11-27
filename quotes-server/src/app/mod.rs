mod config;

use chrono::{DateTime, Duration, Utc};
use futures::stream::StreamExt;
use rust_decimal::Decimal;
use sqlx::{Postgres, Transaction};
use tracing::{info_span, Instrument};

use shared::{
    health::HealthCheckTrigger,
    payload::{PriceStreamPayload, BITFINEX_EXCHANGE_ID, OKEX_EXCHANGE_ID},
    pubsub::*,
};

use ledger::*;

use crate::{cache::*, currency::*, error::*, price::*, quote::*};
pub use config::*;

pub struct QuotesApp {
    price_calculator: PriceCalculator,
    quotes: Quotes,
    ledger: Ledger,
    pool: sqlx::PgPool,
    config: QuotesConfig,
}

#[allow(clippy::too_many_arguments)]
impl QuotesApp {
    pub async fn run(
        pool: sqlx::PgPool,
        mut health_check_trigger: HealthCheckTrigger,
        health_check_cfg: QuotesServerHealthCheckConfig,
        fee_calc_cfg: QuotesFeeCalculatorConfig,
        subscriber: memory::Subscriber<PriceStreamPayload>,
        price_cache_config: QuotesExchangePriceCacheConfig,
        exchange_weights: ExchangeWeights,
        config: QuotesConfig,
    ) -> Result<Self, QuotesAppError> {
        let health_subscriber = subscriber.resubscribe();
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.next().await {
                let _ = check.send(
                    health_subscriber
                        .healthy(health_check_cfg.unhealthy_msg_interval_price)
                        .await,
                );
            }
        });

        let mut price_mixer = PriceMixer::new();

        if let Some(weight) = exchange_weights.okex {
            if weight > Decimal::ZERO {
                let okex_order_book_cache = OrderBookCache::new(price_cache_config.clone());
                Self::subscribe_okex(subscriber.resubscribe(), okex_order_book_cache.clone())
                    .await?;
                price_mixer.add_provider(OKEX_EXCHANGE_ID, okex_order_book_cache, weight);
            }
        }

        if let Some(weight) = exchange_weights.bitfinex {
            if weight > Decimal::ZERO {
                let bitfinex_price_cache = ExchangeTickCache::new(price_cache_config.clone());
                Self::subscribe_bitfinex(subscriber.resubscribe(), bitfinex_price_cache.clone())
                    .await?;
                price_mixer.add_provider(BITFINEX_EXCHANGE_ID, bitfinex_price_cache, weight);
            }
        }

        let quotes = Quotes::new(&pool);
        let ledger = Ledger::init(&pool).await?;

        Ok(Self {
            price_calculator: PriceCalculator::new(fee_calc_cfg, price_mixer),
            quotes,
            ledger,
            pool,
            config,
        })
    }

    pub async fn quote_cents_from_sats_for_buy(
        &self,
        sats: Decimal,
        immediate_execution: bool,
    ) -> Result<Quote, QuotesAppError> {
        let sats = Satoshis::from(sats);
        let res = self
            .price_calculator
            .cents_from_sats_for_buy(sats.clone(), immediate_execution)
            .await?;
        let expiry_time = expiration_time_from_duration(self.config.expiration_interval);
        let new_quote = NewQuote::builder()
            .direction(Direction::BuyCents)
            .immediate_execution(immediate_execution)
            .cent_amount(res.cents)
            .sat_amount(res.sats)
            .cents_spread(res.cents_spread)
            .sats_spread(res.sats_spread)
            .expires_at(expiry_time)
            .build()
            .expect("Could not build quote");
        let mut tx = self.pool.begin().await?;
        let mut quote = self.quotes.create(&mut tx, new_quote).await?;
        if immediate_execution {
            self.accept_quote_in_tx(tx, &mut quote).await?;
        } else {
            tx.commit().await?;
        }

        Ok(quote)
    }

    pub async fn quote_cents_from_sats_for_sell(
        &self,
        sats: Decimal,
        immediate_execution: bool,
    ) -> Result<Quote, QuotesAppError> {
        let sats = Satoshis::from(sats);
        let res = self
            .price_calculator
            .cents_from_sats_for_sell(sats.clone(), immediate_execution)
            .await?;
        let expiry_time = expiration_time_from_duration(self.config.expiration_interval);
        let new_quote = NewQuote::builder()
            .direction(Direction::SellCents)
            .immediate_execution(immediate_execution)
            .cent_amount(res.cents)
            .sat_amount(res.sats)
            .cents_spread(res.cents_spread)
            .sats_spread(res.sats_spread)
            .expires_at(expiry_time)
            .build()
            .expect("Could not build quote");
        let mut tx = self.pool.begin().await?;
        let mut quote = self.quotes.create(&mut tx, new_quote).await?;
        if immediate_execution {
            self.accept_quote_in_tx(tx, &mut quote).await?;
        } else {
            tx.commit().await?;
        }

        Ok(quote)
    }

    pub async fn quote_sats_from_cents_for_sell(
        &self,
        cents: Decimal,
        immediate_execution: bool,
    ) -> Result<Quote, QuotesAppError> {
        let cents = UsdCents::from(cents);
        let res = self
            .price_calculator
            .sats_from_cents_for_sell(cents.clone(), immediate_execution)
            .await?;
        let expiry_time = expiration_time_from_duration(self.config.expiration_interval);
        let new_quote = NewQuote::builder()
            .direction(Direction::SellCents)
            .immediate_execution(immediate_execution)
            .cent_amount(res.cents)
            .sat_amount(res.sats)
            .cents_spread(res.cents_spread)
            .sats_spread(res.sats_spread)
            .expires_at(expiry_time)
            .build()
            .expect("Could not build quote");
        let mut tx = self.pool.begin().await?;
        let mut quote = self.quotes.create(&mut tx, new_quote).await?;
        if immediate_execution {
            self.accept_quote_in_tx(tx, &mut quote).await?;
        } else {
            tx.commit().await?;
        }

        Ok(quote)
    }

    pub async fn quote_sats_from_cents_for_buy(
        &self,
        cents: Decimal,
        immediate_execution: bool,
    ) -> Result<Quote, QuotesAppError> {
        let cents = UsdCents::from(cents);
        let res = self
            .price_calculator
            .sats_from_cents_for_buy(cents.clone(), immediate_execution)
            .await?;
        let expiry_time = expiration_time_from_duration(self.config.expiration_interval);
        let new_quote = NewQuote::builder()
            .direction(Direction::BuyCents)
            .immediate_execution(immediate_execution)
            .cent_amount(res.cents)
            .sat_amount(res.sats)
            .cents_spread(res.cents_spread)
            .sats_spread(res.sats_spread)
            .expires_at(expiry_time)
            .build()
            .expect("Could not build quote");
        let mut tx = self.pool.begin().await?;
        let mut quote = self.quotes.create(&mut tx, new_quote).await?;
        if immediate_execution {
            self.accept_quote_in_tx(tx, &mut quote).await?;
        } else {
            tx.commit().await?;
        }
        Ok(quote)
    }

    pub async fn accept_quote(&self, id: QuoteId) -> Result<(), QuotesAppError> {
        let mut quote = self.quotes.find_by_id(id).await?;
        let tx = self.pool.begin().await?;
        self.accept_quote_in_tx(tx, &mut quote).await?;
        Ok(())
    }

    async fn accept_quote_in_tx(
        &self,
        mut tx: Transaction<'_, Postgres>,
        quote: &mut Quote,
    ) -> Result<(), QuotesAppError> {
        quote.accept()?;
        if quote.direction == Direction::SellCents {
            let params = SellUsdQuoteAcceptedParams {
                usd_cents_amount: *quote.cent_amount.amount(),
                satoshi_amount: *quote.sat_amount.amount(),
                meta: SellUsdQuoteAcceptedMeta {
                    timestamp: quote.accepted_at().expect("Quote was just accepted"),
                },
            };
            self.quotes.update(&mut tx, quote).await?;
            self.ledger
                .sell_usd_quote_accepted(tx, LedgerTxId::new(), params)
                .await?;
        } else {
            let params = BuyUsdQuoteAcceptedParams {
                usd_cents_amount: *quote.cent_amount.amount(),
                satoshi_amount: *quote.sat_amount.amount(),
                meta: BuyUsdQuoteAcceptedMeta {
                    timestamp: quote.accepted_at().expect("Quote was just accepted"),
                },
            };
            self.quotes.update(&mut tx, quote).await?;
            self.ledger
                .buy_usd_quote_accepted(tx, LedgerTxId::new(), params)
                .await?;
        }

        Ok(())
    }

    async fn subscribe_okex(
        mut subscriber: memory::Subscriber<PriceStreamPayload>,
        order_book_cache: OrderBookCache,
    ) -> Result<(), QuotesAppError> {
        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                if let PriceStreamPayload::OkexBtcUsdSwapOrderBookPayload(price_msg) = msg.payload {
                    let span = info_span!(
                        "quotes_server.okex_order_book_received",
                        message_type = %msg.payload_type,
                        correlation_id = %msg.meta.correlation_id
                    );
                    shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                    async {
                        order_book_cache.apply_update(price_msg).await;
                    }
                    .instrument(span)
                    .await;
                }
            }
        });

        Ok(())
    }

    async fn subscribe_bitfinex(
        mut subscriber: memory::Subscriber<PriceStreamPayload>,
        price_cache: ExchangeTickCache,
    ) -> Result<(), QuotesAppError> {
        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                if let PriceStreamPayload::BitfinexBtcUsdSwapPricePayload(price_msg) = msg.payload {
                    let span = info_span!(
                        "quotes_server.bitfinex_price_tick_received",
                        message_type = %msg.payload_type,
                        correlation_id = %msg.meta.correlation_id
                    );
                    shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                    async {
                        price_cache
                            .apply_update(price_msg, msg.meta.correlation_id)
                            .await;
                    }
                    .instrument(span)
                    .await;
                }
            }
        });

        Ok(())
    }
}

fn expiration_time_from_duration(duration: Duration) -> DateTime<Utc> {
    Utc::now()
        + chrono::Duration::from_std(duration.to_std().expect("Failed to convert duration"))
            .expect("Failed to create chrono::Duration")
}
