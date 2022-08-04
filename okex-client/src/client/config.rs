use shared::string_wrapper;

string_wrapper! { ApiKey }
string_wrapper! { SecretKey }
string_wrapper! { PassPhrase }

pub struct ApiConfig {
    pub api_key: ApiKey,
    pub secret_key: SecretKey,
    pub passphrase: PassPhrase,
}
