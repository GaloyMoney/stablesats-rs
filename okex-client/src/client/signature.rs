use chrono::{DateTime, SecondsFormat, Utc};
use data_encoding::BASE64;
use ring::hmac;

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub enum RequestMethod {
    GET,
    POST,
}

/// Wrapper around the 'access_signature' function that generates signatures
/// matching to the appropriate request method
pub struct AccessSignature {}

impl AccessSignature {
    /// Helper method to compute the access signature
    fn access_signature(
        method: &str,
        request_path: String,
        body: Option<String>,
        secret_key: String,
        request_timestamp: DateTime<Utc>,
    ) -> String {
        let timestamp = request_timestamp.to_rfc3339_opts(SecondsFormat::Millis, true);
        let body = body.unwrap_or_else(|| "".to_string());
        let pre_hash_str = format!("{}{}{}{}", timestamp, method, request_path, body);

        let key = hmac::Key::new(hmac::HMAC_SHA256, secret_key.as_bytes());
        let signature = hmac::sign(&key, pre_hash_str.as_bytes());
        let sign_base64 = BASE64.encode(signature.as_ref());

        sign_base64
    }

    /// Generate the access signature for each authenticated request to
    /// OKEX
    pub fn generate(
        method: RequestMethod,
        request_path: String,
        body: Option<String>,
        secret_key: String,
        request_timestamp: DateTime<Utc>,
    ) -> String {
        match method {
            RequestMethod::GET => {
                let method = "GET";
                Self::access_signature(method, request_path, body, secret_key, request_timestamp)
            }
            RequestMethod::POST => {
                let method = "POST";
                Self::access_signature(method, request_path, body, secret_key, request_timestamp)
            }
        }
    }
}

mod tests {
    #[test]
    #[allow(dead_code)]
    fn generate_access_signature() {
        use chrono::{DateTime, SecondsFormat, Utc};
        use data_encoding::BASE64;
        use ring::hmac;

        use crate::client::signature::{AccessSignature, RequestMethod};

        let current_time = Utc::now();
        let pre_hash = format!(
            "{}{}{}{}",
            current_time.to_rfc3339_opts(SecondsFormat::Millis, true),
            "GET",
            "user/self/verify",
            ""
        );

        let key = hmac::Key::new(hmac::HMAC_SHA256, "secret".as_bytes());
        let signature = hmac::sign(&key, pre_hash.as_bytes());
        let expected_sign = BASE64.encode(signature.as_ref());

        let method = RequestMethod::GET;
        let request_path = "user/self/verify".to_string();
        let body = None;
        let secret_key = "secret".to_string();

        let access_sign =
            AccessSignature::generate(method, request_path, body, secret_key, current_time);

        println!("Access sign: {}", access_sign);

        assert_eq!(access_sign, expected_sign);
    }
}
