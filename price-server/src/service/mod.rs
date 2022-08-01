pub mod proto {
    tonic::include_proto!("services.price.v1");
}

use proto::{price_server::Price, *};
use tonic::{Request, Response, Status};

pub struct PriceService {}

#[tonic::async_trait]
impl Price for PriceService {
    async fn get_cents_from_sats_for_immediate_buy(
        &self,
        _request: Request<GetCentsFromSatsForImmediateBuyRequest>,
    ) -> Result<Response<GetCentsFromSatsForImmediateBuyResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_cents_from_sats_for_immediate_sell(
        &self,
        _request: Request<GetCentsFromSatsForImmediateSellRequest>,
    ) -> Result<Response<GetCentsFromSatsForImmediateSellResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_cents_from_sats_for_future_buy(
        &self,
        _request: Request<GetCentsFromSatsForFutureBuyRequest>,
    ) -> Result<Response<GetCentsFromSatsForFutureBuyResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_cents_from_sats_for_future_sell(
        &self,
        _request: Request<GetCentsFromSatsForFutureSellRequest>,
    ) -> Result<Response<GetCentsFromSatsForFutureSellResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_sats_from_cents_for_immediate_buy(
        &self,
        _request: Request<GetSatsFromCentsForImmediateBuyRequest>,
    ) -> Result<Response<GetSatsFromCentsForImmediateBuyResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_sats_from_cents_for_immediate_sell(
        &self,
        _request: Request<GetSatsFromCentsForImmediateSellRequest>,
    ) -> Result<Response<GetSatsFromCentsForImmediateSellResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_sats_from_cents_for_future_buy(
        &self,
        _request: Request<GetSatsFromCentsForFutureBuyRequest>,
    ) -> Result<Response<GetSatsFromCentsForFutureBuyResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_sats_from_cents_for_future_sell(
        &self,
        _request: Request<GetSatsFromCentsForFutureSellRequest>,
    ) -> Result<Response<GetSatsFromCentsForFutureSellResponse>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn get_cents_per_sats_exchange_mid_rate(
        &self,
        _request: Request<GetCentsPerSatsExchangeMidRateRequest>,
    ) -> Result<Response<GetCentsPerSatsExchangeMidRateResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
