use crate::error::QuotesAppError;

impl From<QuotesAppError> for tonic::Status {
    fn from(_err: QuotesAppError) -> Self {
        tonic::Status::new(tonic::Code::Unknown, "Unknown error")
    }
}
