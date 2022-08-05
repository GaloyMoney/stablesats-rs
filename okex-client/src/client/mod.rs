mod config;
mod error;

use config::*;
use data_encoding::BASE64;
use reqwest::Client as ReqwestClient;
use ring::hmac;
use shared::string_wrapper;
use shared::time::TimeStamp;

use self::error::OkexClientError;

string_wrapper! { ApiKey }
string_wrapper! { PassPhrase }
string_wrapper! { AccessSignature }
string_wrapper! { SecretKey }

fn generate_signature(
    method: String,
    request_path: String,
    body: Option<String>,
    secret_key: SecretKey,
) -> String {
    let timestamp = TimeStamp::now().to_string();
    let body = body.unwrap_or("".to_string());
    let pre_hash_str = format!("{}{}{}{}", timestamp, method, request_path, body);

    let key = hmac::Key::new(hmac::HMAC_SHA256, secret_key.0.as_bytes());
    let signature = hmac::sign(&key, pre_hash_str.as_bytes());
    let sign_base64 = BASE64.encode(signature.as_ref());

    sign_base64
}

#[derive(Debug)]
pub struct AuthHeaders {
    pub ok_access_key: ApiKey,
    pub ok_access_sign: AccessSignature,
    pub ok_access_passphrase: PassPhrase,
    pub ok_access_timestamp: TimeStamp,
}

impl TryFrom<ApiConfig> for AuthHeaders {
    type Error = OkexClientError;

    fn try_from(config: ApiConfig) -> Result<Self, Self::Error> {
        if let (Some(api_key), Some(passphrase), Some(secret_key)) =
            (config.api_key, config.passphrase, config.secret_key)
        {
            let ok_access_key = ApiKey::from(api_key);
            let ok_access_passphrase = PassPhrase::from(passphrase);
            let ok_access_timestamp = TimeStamp::now();
            let ok_access_sign = AccessSignature(generate_signature(
                "GET".to_string(),
                "/users/self/verify".to_string(),
                None,
                SecretKey::from(secret_key),
            ));

            return Ok(Self {
                ok_access_key,
                ok_access_sign,
                ok_access_passphrase,
                ok_access_timestamp,
            });
        }
        Err(OkexClientError::AuthHeadersError)
    }
}

#[derive(Debug)]
pub struct OkexClient {
    pub client: ReqwestClient,
    pub headers: Result<AuthHeaders, OkexClientError>,
}

impl OkexClient {
    pub fn new(config: ApiConfig) -> Self {
        Self {
            client: ReqwestClient::new(),
            headers: AuthHeaders::try_from(config),
        }
    }
}
