mod config;
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
use tonic::{transport::Server, Request, Response, Status};
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::app::*;

pub use config::*;
pub use error::*;

pub struct Quotes {
    app: QuotesApp,
}

#[tonic::async_trait]
impl QuoteService for Quotes {
    async fn get_quote_to_buy_cents(
        &self,
        request: Request<GetQuoteToBuyCentsRequest>,
    ) -> Result<Response<GetQuoteToBuyCentsResponse>, Status> {
        unimplemented!()
    }

    async fn get_quote_to_sell_cents(
        &self,
        request: Request<GetQuoteToSellCentsRequest>,
    ) -> Result<Response<GetQuoteToSellCentsResponse>, Status> {
        unimplemented!()
    }

    async fn accept_quote(
        &self,
        request: Request<AcceptQuoteRequest>,
    ) -> Result<Response<AcceptQuoteResponse>, Status> {
        unimplemented!()
    }
}

pub(crate) async fn start(
    server_config: QuoteServerConfig,
    app: QuotesApp,
) -> Result<(), QuotesServerError> {
    let quote_service = Quotes { app };
    Server::builder()
        .add_service(proto::quote_service_server::QuoteServiceServer::new(
            quote_service,
        ))
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
