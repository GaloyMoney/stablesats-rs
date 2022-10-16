use thiserror::Error;

#[derive(Error, Debug)]
pub enum PayloadError {
    #[error("PayloadError: CheckSumValidation - Can't validate accuracy of depth data")]
    CheckSumValidation,
}
