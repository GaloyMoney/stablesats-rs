use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceFeederError {
    #[error("connection to websocket failed error")]
    ConnectionError(),
    #[error("Price feeder encountered a translation error")]
    TranslationError(),
    #[error("Price feeder encountered unknown error")]
    UnknownPriceFeederError(),
}
