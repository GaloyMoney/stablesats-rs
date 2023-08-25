use super::{config::BriaClientConfig, proto};
use crate::error::BriaClientError;

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

        if config.key.is_empty() {
            return Err(BriaClientError::EmptyKeyError);
        }

        Ok(Self {
            config,
            proto_client,
        })
    }

    pub fn inject_auth_token<T>(
        &self,
        mut request: tonic::Request<T>,
    ) -> Result<tonic::Request<T>, BriaClientError> {
        let key = &self.config.key;
        request.metadata_mut().insert(
            PROFILE_API_KEY_HEADER,
            tonic::metadata::MetadataValue::try_from(key)
                .map_err(|_| BriaClientError::CouldNotCreateMetadataValue)?,
        );
        Ok(request)
    }

    pub async fn onchain_address(&mut self) -> Result<OnchainAddress, BriaClientError> {
        let request = tonic::Request::new(proto::GetAddressRequest {
            identifier: Some(proto::get_address_request::Identifier::ExternalId(
                self.config.external_id.clone(),
            )),
        });

        self.proto_client
            .get_address(self.inject_auth_token(request)?)
            .await
            .ok()
            .and_then(|res| {
                res.into_inner()
                    .address
                    .map(|addr| OnchainAddress { address: addr })
            })
            .ok_or(BriaClientError::AddressNotFoundError)
    }
}
