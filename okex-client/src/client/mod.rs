mod config;
mod constants;
mod error;
mod headers;
mod signature;

use chrono::Utc;
use reqwest::Client as ReqwestClient;
use serde::Deserialize;

pub use config::*;
pub use constants::*;
pub use error::*;
pub use headers::*;
use shared::string_wrapper;
use signature::*;

string_wrapper!(CodeRaw);
string_wrapper!(MessageRaw);
string_wrapper!(ChainName);
string_wrapper!(DepositAddressRaw);
string_wrapper!(CurrencyRaw);
string_wrapper!(BeneficiaryAccountRaw);
string_wrapper!(ContractAddressRaw);

/// Response struct from OKEX
#[derive(Debug, Deserialize, PartialEq)]
pub struct OkexResponse {
    pub code: CodeRaw,
    pub msg: MessageRaw,
    pub data: Vec<DepositAddressData>,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressData {
    pub chain: ChainName,
    pub ct_addr: ContractAddressRaw,
    pub ccy: CurrencyRaw,
    pub to: BeneficiaryAccountRaw,
    pub addr: DepositAddressRaw,
    pub selected: bool,
}

#[derive(Debug)]
pub struct OkexClient {
    client: ReqwestClient,
    config: ApiConfig,
}

impl OkexClient {
    pub fn new(config: ApiConfig) -> Self {
        Self {
            client: ReqwestClient::new(),
            config,
        }
    }

    pub async fn get_funding_deposit_address(&self) -> Result<OkexResponse, OkexClientError> {
        let request_path = "/api/v5/asset/deposit-address?ccy=BTC";
        let request_timestamp = Utc::now();

        let access_signature = AccessSignature::generate(
            RequestMethod::GET,
            request_path.to_string(),
            None,
            self.config.secret_key.clone(),
            request_timestamp,
        );

        let headers =
            AuthHeaders::create(access_signature, self.config.clone(), request_timestamp)?.headers;
        let url = format!("{}{}", self.config.base_url, request_path);

        let response = self.client.get(url).headers(headers).send().await?;
        let response_text = response.text().await?;

        let response_object = serde_json::from_str::<OkexResponse>(&response_text)?;

        Ok(response_object)
    }
}
