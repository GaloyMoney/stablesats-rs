use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrokerAppError {
    #[error("BrokerAppError")]
    UnknownError,
}
