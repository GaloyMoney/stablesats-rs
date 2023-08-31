#[allow(clippy::all)]
mod proto {
    tonic::include_proto!("services.bria.v1");
}

use tracing::instrument;

use super::{config::BriaClientConfig, error::BriaClientError};

type ProtoClient = proto::bria_service_client::BriaServiceClient<tonic::transport::Channel>;

pub const PROFILE_API_KEY_HEADER: &str = "x-bria-api-key";

#[derive(Debug)]
pub struct OnchainAddress {
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct BriaClient {
    config: BriaClientConfig,
    proto_client: ProtoClient,
}

impl BriaClient {
    pub async fn connect(config: BriaClientConfig) -> Result<Self, BriaClientError> {
        let proto_client = ProtoClient::connect(config.url.clone())
            .await
            .map_err(|_| BriaClientError::ConnectionError(config.url.clone()))?;

        Ok(Self {
            config,
            proto_client,
        })
    }

    pub fn inject_auth_token<T>(
        &self,
        mut request: tonic::Request<T>,
    ) -> Result<tonic::Request<T>, BriaClientError> {
        let profile_api_key = &self.config.profile_api_key;
        request.metadata_mut().insert(
            PROFILE_API_KEY_HEADER,
            tonic::metadata::MetadataValue::try_from(profile_api_key)
                .map_err(|_| BriaClientError::CouldNotCreateMetadataValue)?,
        );
        Ok(request)
    }

    #[instrument(name = "bria_client.onchain_address", skip(self), err)]
    pub async fn onchain_address(&mut self) -> Result<OnchainAddress, BriaClientError> {
        let request = tonic::Request::new(proto::GetAddressRequest {
            identifier: Some(proto::get_address_request::Identifier::ExternalId(
                self.config.onchain_address_external_id.clone(),
            )),
        });

        if let Ok(response) = self
            .proto_client
            .get_address(self.inject_auth_token(request)?)
            .await
        {
            if let Some(address) = response.into_inner().address {
                return Ok(OnchainAddress { address });
            }
        }

        let request = tonic::Request::new(proto::NewAddressRequest {
            wallet_name: self.config.wallet_name.clone(),
            external_id: Some(self.config.onchain_address_external_id.clone()),
            metadata: None,
        });

        let response = self
            .proto_client
            .new_address(self.inject_auth_token(request)?)
            .await?;
        Ok(OnchainAddress {
            address: response.into_inner().address,
        })
    }

    #[instrument(name = "bria_client.send_onchain_payment", skip(self), err)]
    pub async fn send_onchain_payment(
        &mut self,
        destination: String,
        satoshis: rust_decimal::Decimal,
    ) -> Result<String, BriaClientError> {
        use rust_decimal::prelude::ToPrimitive;

        let metadata = Some(serde_json::json!({
            "deposit_amount": satoshis,
            "to": "okex",
            "from": "stablesats"
        }));

        let request = tonic::Request::new(proto::SubmitPayoutRequest {
            wallet_name: self.config.wallet_name.clone(),
            payout_queue_name: self.config.payout_queue_name.clone(),
            destination: Some(proto::submit_payout_request::Destination::OnchainAddress(
                destination,
            )),
            satoshis: satoshis
                .abs()
                .to_u64()
                .ok_or(BriaClientError::CouldNotConvertSatoshisToU64)?,
            external_id: None,
            metadata: metadata.map(serde_json::from_value).transpose()?,
        });

        let response = self
            .proto_client
            .submit_payout(self.inject_auth_token(request)?)
            .await?;
        Ok(response.into_inner().id)
    }
}
