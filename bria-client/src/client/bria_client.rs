use anyhow::Context;

use super::{config::BriaClientConfig, proto};

type ProtoClient = proto::bria_service_client::BriaServiceClient<tonic::transport::Channel>;

pub const PROFILE_API_KEY_HEADER: &str = "x-bria-api-key";

#[derive(Debug)]
pub struct OnchainAddress {
    pub address: String,
}

pub struct BriaClient {
    config: BriaClientConfig,
}

impl BriaClient {
    pub fn new(config: BriaClientConfig) -> Self {
        Self { config }
    }

    async fn connect(&self) -> anyhow::Result<ProtoClient> {
        match ProtoClient::connect(self.config.url.to_string()).await {
            Ok(client) => Ok(client),
            Err(err) => {
                eprintln!(
                    "Couldn't connect to daemon\nAre you sure its running on {}?\n",
                    self.config.url
                );
                Err(anyhow::anyhow!(err))
            }
        }
    }

    pub fn inject_auth_token<T>(
        &self,
        mut request: tonic::Request<T>,
    ) -> anyhow::Result<tonic::Request<T>> {
        if self.config.key.is_empty() {
            return Err(anyhow::anyhow!("Bria key cannot be empty"));
        }
        let key = &self.config.key;
        request.metadata_mut().insert(
            PROFILE_API_KEY_HEADER,
            tonic::metadata::MetadataValue::try_from(key)
                .context("Couldn't create MetadataValue")?,
        );
        Ok(request)
    }

    pub async fn get_address(&self) -> anyhow::Result<OnchainAddress> {
        let request = tonic::Request::new(proto::GetAddressRequest {
            identifier: Some(proto::get_address_request::Identifier::ExternalId(
                self.config.external_id.clone(),
            )),
        });
        let response = self
            .connect()
            .await?
            .get_address(self.inject_auth_token(request)?)
            .await?;
        if let Some(addr) = response.into_inner().address {
            return Ok(OnchainAddress { address: addr });
        }
        Err(anyhow::anyhow!(
            "Couldn't find address for the given external_id"
        ))
    }
}
