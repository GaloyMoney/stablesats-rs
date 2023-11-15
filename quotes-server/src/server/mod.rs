mod config;
mod convert;
mod error;

#[allow(clippy::all)]
pub mod proto {
    tonic::include_proto!("services.quotes.v1");
}

use opentelemetry::{
    propagation::{Extractor, TextMapPropagator},
    sdk::propagation::TraceContextPropagator,
};
use proto::{quote_service_server::QuoteService, *};
use rust_decimal::Decimal;
use tonic::{transport::Server, Request, Response, Status};
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{app::*, error::QuotesAppError};

pub use config::*;
pub use error::*;

pub struct Quotes {
    app: QuotesApp,
}

#[tonic::async_trait]
impl QuoteService for Quotes {
    #[instrument(name = "quotes_server.get_quote_to_buy_usd", skip_all,
        fields(error, error.level, error.message),
        err
    )]
    async fn get_quote_to_buy_usd(
        &self,
        request: Request<GetQuoteToBuyUsdRequest>,
    ) -> Result<Response<GetQuoteToBuyUsdResponse>, Status> {
        shared::tracing::record_error(tracing::Level::ERROR, || async move {
            extract_tracing(&request);
            let req = request.into_inner();
            let quote = match req.quote_for {
                Some(get_quote_to_buy_usd_request::QuoteFor::AmountToSellInSats(amount)) => {
                    self.app
                        .quote_cents_from_sats_for_buy(
                            Decimal::from(amount),
                            req.immediate_execution,
                        )
                        .await?
                }
                Some(get_quote_to_buy_usd_request::QuoteFor::AmountToBuyInCents(amount)) => {
                    self.app
                        .quote_sats_from_cents_for_buy(
                            Decimal::from(amount),
                            req.immediate_execution,
                        )
                        .await?
                }
                _ => return Err(Status::invalid_argument("invalid argument for quote")),
            };
            let response = GetQuoteToBuyUsdResponse::from(quote);

            Ok(Response::new(response))
        })
        .await
    }

    #[instrument(name = "quotes_server.get_quote_to_sell_usd", skip_all,
    fields(error, error.level, error.message),
    err
    )]
    async fn get_quote_to_sell_usd(
        &self,
        request: Request<GetQuoteToSellUsdRequest>,
    ) -> Result<Response<GetQuoteToSellUsdResponse>, Status> {
        shared::tracing::record_error(tracing::Level::ERROR, || async move {
            extract_tracing(&request);
            let req = request.into_inner();
            let quote = match req.quote_for {
                Some(get_quote_to_sell_usd_request::QuoteFor::AmountToSellInCents(amount)) => {
                    self.app
                        .quote_sats_from_cents_for_sell(
                            Decimal::from(amount),
                            req.immediate_execution,
                        )
                        .await?
                }
                Some(get_quote_to_sell_usd_request::QuoteFor::AmountToBuyInSats(amount)) => {
                    self.app
                        .quote_cents_from_sats_for_sell(
                            Decimal::from(amount),
                            req.immediate_execution,
                        )
                        .await?
                }
                _ => return Err(Status::invalid_argument("invalid argument for quote")),
            };
            let response = GetQuoteToSellUsdResponse::from(quote);

            Ok(Response::new(response))
        })
        .await
    }

    #[instrument(name = "quotes_server.accept_quote", skip_all,
    fields(error, error.level, error.message),
    err
    )]
    async fn accept_quote(
        &self,
        request: Request<AcceptQuoteRequest>,
    ) -> Result<Response<AcceptQuoteResponse>, Status> {
        shared::tracing::record_error(tracing::Level::ERROR, || async move {
            extract_tracing(&request);
            let req = request.into_inner();
            self.app
                .accept_quote(
                    req.quote_id
                        .parse()
                        .map_err(QuotesAppError::CouldNotParseIncomingUuid)?,
                )
                .await?;
            Ok(Response::new(AcceptQuoteResponse {}))
        })
        .await
    }
}

pub(crate) async fn start(
    server_config: QuotesServerConfig,
    app: QuotesApp,
) -> Result<(), QuotesServerError> {
    let quote_service = Quotes { app };
    Server::builder()
        .add_service(quote_service_server::QuoteServiceServer::new(quote_service))
        .serve(([0, 0, 0, 0], server_config.listen_port).into())
        .await?;
    Ok(())
}

pub fn extract_tracing<T>(request: &Request<T>) {
    let propagator = TraceContextPropagator::new();
    let parent_cx = propagator.extract(&RequestContextExtractor(request));
    tracing::Span::current().set_parent(parent_cx)
}

struct RequestContextExtractor<'a, T>(&'a Request<T>);

impl<'a, T> Extractor for RequestContextExtractor<'a, T> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.metadata().get(key).and_then(|s| s.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .metadata()
            .keys()
            .filter_map(|k| {
                if let tonic::metadata::KeyRef::Ascii(key) = k {
                    Some(key.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}
