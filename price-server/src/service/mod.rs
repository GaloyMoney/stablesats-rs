mod convert;

pub mod proto {
    tonic::include_proto!("services.price.v1");
}

use proto::{price_server::Price, *};
use tonic::{Request, Response, Status};

use shared::currency::*;
use crate::app::*;

pub struct PriceService {
    app: PriceApp,
}

#[tonic::async_trait]
impl Price for PriceService {
    async fn get_cents_from_sats_for_immediate_buy(
        &self,
        request: Request<GetCentsFromSatsForImmediateBuyRequest>,
    ) -> Result<Response<GetCentsFromSatsForImmediateBuyResponse>, Status> {
        let req = request.into_inner();
        let amount_in_cents = self.app.get_cents_from_sats_for_immediate_buy(Sats::from_major(req.amount_in_satoshis)).await?;
        Ok(Response::new(GetCentsFromSatsForImmediateBuyResponse {
            amount_in_cents,
        }))
    }

    async fn get_cents_from_sats_for_immediate_sell(
        &self,
        request: Request<GetCentsFromSatsForImmediateSellRequest>,
    ) -> Result<Response<GetCentsFromSatsForImmediateSellResponse>, Status> {
        let req = request.into_inner();
        let amount_in_cents = self.app.get_cents_from_sats_for_immediate_sell(Sats::from_major(req.amount_in_satoshis)).await?;
        Ok(Response::new(GetCentsFromSatsForImmediateSellResponse {
            amount_in_cents,
        }))
    }

    async fn get_cents_from_sats_for_future_buy(
        &self,
        request: Request<GetCentsFromSatsForFutureBuyRequest>,
    ) -> Result<Response<GetCentsFromSatsForFutureBuyResponse>, Status> {
        let req = request.into_inner();
        let amount_in_cents = self.app.get_cents_from_sats_for_future_buy(Sats::from_major(req.amount_in_satoshis)).await?;
        Ok(Response::new(GetCentsFromSatsForFutureBuyResponse {
            amount_in_cents,
        }))
    }

    async fn get_cents_from_sats_for_future_sell(
        &self,
        request: Request<GetCentsFromSatsForFutureSellRequest>,
    ) -> Result<Response<GetCentsFromSatsForFutureSellResponse>, Status> {
        let req = request.into_inner();
        let amount_in_cents = self.app.get_cents_from_sats_for_future_sell(Sats::from_major(req.amount_in_satoshis)).await?;
        Ok(Response::new(GetCentsFromSatsForFutureSellResponse {
            amount_in_cents,
        }))
    }

    async fn get_sats_from_cents_for_immediate_buy(
        &self,
        request: Request<GetSatsFromCentsForImmediateBuyRequest>,
    ) -> Result<Response<GetSatsFromCentsForImmediateBuyResponse>, Status> {
        let req = request.into_inner();
        let amount_in_satoshis = self.app.get_sats_from_cents_for_immediate_buy(Sats::from_major(req.amount_in_cents)).await?;
        Ok(Response::new(GetSatsFromCentsForImmediateBuyResponse { amount_in_satoshis }))
    }

    async fn get_sats_from_cents_for_immediate_sell(
        &self,
        request: Request<GetSatsFromCentsForImmediateSellRequest>,
    ) -> Result<Response<GetSatsFromCentsForImmediateSellResponse>, Status> {
        let req = request.into_inner();
        let amount_in_satoshis = self.app.get_sats_from_cents_for_immediate_sell(Sats::from_major(req.amount_in_cents)).await?;
        Ok(Response::new(GetSatsFromCentsForImmediateSellResponse { amount_in_satoshis }))
    }

    async fn get_sats_from_cents_for_future_buy(
        &self,
        request: Request<GetSatsFromCentsForFutureBuyRequest>,
    ) -> Result<Response<GetSatsFromCentsForFutureBuyResponse>, Status> {
        let req = request.into_inner();
        let amount_in_satoshis = self.app.get_sats_from_cents_for_future_buy(Sats::from_major(req.amount_in_cents)).await?;
        Ok(Response::new(GetSatsFromCentsForFutureBuyResponse { amount_in_satoshis }))
    }

    async fn get_sats_from_cents_for_future_sell(
        &self,
        request: Request<GetSatsFromCentsForFutureSellRequest>,
    ) -> Result<Response<GetSatsFromCentsForFutureSellResponse>, Status> {
        let req = request.into_inner();
        let amount_in_satoshis = self.app.get_sats_from_cents_for_future_sell(Sats::from_major(req.amount_in_cents)).await?;
        Ok(Response::new(GetSatsFromCentsForFutureSellResponse { amount_in_satoshis }))
    }

    async fn get_cents_per_sats_exchange_mid_rate(
        &self,
        _request: Request<GetCentsPerSatsExchangeMidRateRequest>,
    ) -> Result<Response<GetCentsPerSatsExchangeMidRateResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
