mod config;
mod convert;
mod error;

pub mod proto {
    tonic::include_proto!("services.price.v1");
}

use opentelemetry::{
    propagation::{Extractor, TextMapPropagator},
    sdk::propagation::TraceContextPropagator,
};
use proto::{price_service_server::PriceService, *};
use tonic::{transport::Server, Request, Response, Status};
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::app::*;
use shared::currency::*;

pub use config::*;
pub use error::*;

pub struct Price {
    app: PriceApp,
}

#[tonic::async_trait]
impl PriceService for Price {
    #[instrument(skip_all,
        fields(amount_in_satoshis = request.get_ref().amount_in_satoshis),
        err
    )]
    async fn get_cents_from_sats_for_immediate_buy(
        &self,
        request: Request<GetCentsFromSatsForImmediateBuyRequest>,
    ) -> Result<Response<GetCentsFromSatsForImmediateBuyResponse>, Status> {
        extract_tracing(&request);

        let req = request.into_inner();
        let amount_in_cents = self
            .app
            .get_cents_from_sats_for_immediate_buy(Sats::from_major(req.amount_in_satoshis))
            .await?;
        Ok(Response::new(GetCentsFromSatsForImmediateBuyResponse {
            amount_in_cents,
        }))
    }

    #[instrument(skip_all,
        fields(amount_in_satoshis = request.get_ref().amount_in_satoshis),
        err
     )]
    async fn get_cents_from_sats_for_immediate_sell(
        &self,
        request: Request<GetCentsFromSatsForImmediateSellRequest>,
    ) -> Result<Response<GetCentsFromSatsForImmediateSellResponse>, Status> {
        extract_tracing(&request);

        let req = request.into_inner();
        let amount_in_cents = self
            .app
            .get_cents_from_sats_for_immediate_sell(Sats::from_major(req.amount_in_satoshis))
            .await?;
        Ok(Response::new(GetCentsFromSatsForImmediateSellResponse {
            amount_in_cents,
        }))
    }

    #[instrument(skip_all,
        fields(amount_in_satoshis = request.get_ref().amount_in_satoshis,
                time_in_seconds = request.get_ref().time_in_seconds),
        err
    )]
    async fn get_cents_from_sats_for_future_buy(
        &self,
        request: Request<GetCentsFromSatsForFutureBuyRequest>,
    ) -> Result<Response<GetCentsFromSatsForFutureBuyResponse>, Status> {
        extract_tracing(&request);

        let req = request.into_inner();
        let amount_in_cents = self
            .app
            .get_cents_from_sats_for_future_buy(Sats::from_major(req.amount_in_satoshis))
            .await?;
        Ok(Response::new(GetCentsFromSatsForFutureBuyResponse {
            amount_in_cents,
        }))
    }

    #[instrument(skip_all,
        fields(amount_in_satoshis = request.get_ref().amount_in_satoshis,
                time_in_seconds = request.get_ref().time_in_seconds),
        err
    )]
    async fn get_cents_from_sats_for_future_sell(
        &self,
        request: Request<GetCentsFromSatsForFutureSellRequest>,
    ) -> Result<Response<GetCentsFromSatsForFutureSellResponse>, Status> {
        extract_tracing(&request);

        let req = request.into_inner();
        let amount_in_cents = self
            .app
            .get_cents_from_sats_for_future_sell(Sats::from_major(req.amount_in_satoshis))
            .await?;
        Ok(Response::new(GetCentsFromSatsForFutureSellResponse {
            amount_in_cents,
        }))
    }

    #[instrument(skip_all,
        fields(amount_in_cents = request.get_ref().amount_in_cents),
        err
    )]
    async fn get_sats_from_cents_for_immediate_buy(
        &self,
        request: Request<GetSatsFromCentsForImmediateBuyRequest>,
    ) -> Result<Response<GetSatsFromCentsForImmediateBuyResponse>, Status> {
        extract_tracing(&request);

        let req = request.into_inner();
        let amount_in_satoshis = self
            .app
            .get_sats_from_cents_for_immediate_buy(UsdCents::from_major(req.amount_in_cents))
            .await?;
        Ok(Response::new(GetSatsFromCentsForImmediateBuyResponse {
            amount_in_satoshis,
        }))
    }

    #[instrument(skip_all,
        fields(amount_in_cents = request.get_ref().amount_in_cents),
        err
    )]
    async fn get_sats_from_cents_for_immediate_sell(
        &self,
        request: Request<GetSatsFromCentsForImmediateSellRequest>,
    ) -> Result<Response<GetSatsFromCentsForImmediateSellResponse>, Status> {
        extract_tracing(&request);

        let req = request.into_inner();
        let amount_in_satoshis = self
            .app
            .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(req.amount_in_cents))
            .await?;
        Ok(Response::new(GetSatsFromCentsForImmediateSellResponse {
            amount_in_satoshis,
        }))
    }

    #[instrument(skip_all,
        fields(amount_in_cents = request.get_ref().amount_in_cents,
                time_in_seconds = request.get_ref().time_in_seconds),
        err
    )]
    async fn get_sats_from_cents_for_future_buy(
        &self,
        request: Request<GetSatsFromCentsForFutureBuyRequest>,
    ) -> Result<Response<GetSatsFromCentsForFutureBuyResponse>, Status> {
        extract_tracing(&request);

        let req = request.into_inner();
        let amount_in_satoshis = self
            .app
            .get_sats_from_cents_for_future_buy(UsdCents::from_major(req.amount_in_cents))
            .await?;
        Ok(Response::new(GetSatsFromCentsForFutureBuyResponse {
            amount_in_satoshis,
        }))
    }

    #[instrument(skip_all,
        fields(amount_in_cents = request.get_ref().amount_in_cents,
                time_in_seconds = request.get_ref().time_in_seconds),
        err
    )]
    async fn get_sats_from_cents_for_future_sell(
        &self,
        request: Request<GetSatsFromCentsForFutureSellRequest>,
    ) -> Result<Response<GetSatsFromCentsForFutureSellResponse>, Status> {
        extract_tracing(&request);

        let req = request.into_inner();
        let amount_in_satoshis = self
            .app
            .get_sats_from_cents_for_future_sell(UsdCents::from_major(req.amount_in_cents))
            .await?;
        Ok(Response::new(GetSatsFromCentsForFutureSellResponse {
            amount_in_satoshis,
        }))
    }

    #[instrument(skip_all, err)]
    async fn get_cents_per_sats_exchange_mid_rate(
        &self,
        request: Request<GetCentsPerSatsExchangeMidRateRequest>,
    ) -> Result<Response<GetCentsPerSatsExchangeMidRateResponse>, Status> {
        extract_tracing(&request);

        let sat_qty = 1;

        let mid_amount_in_cents = self
            .app
            .get_cents_per_sats_exchange_mid_rate(Sats::from_major(sat_qty))
            .await?;
        let ratio_in_cents_per_satoshis = mid_amount_in_cents as f64;
        Ok(Response::new(GetCentsPerSatsExchangeMidRateResponse {
            ratio_in_cents_per_satoshis,
        }))
    }
}

pub(crate) async fn start(
    server_config: PriceServerConfig,
    app: PriceApp,
) -> Result<(), PriceServerError> {
    let price_service = Price { app };
    Server::builder()
        .add_service(proto::price_service_server::PriceServiceServer::new(
            price_service,
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
