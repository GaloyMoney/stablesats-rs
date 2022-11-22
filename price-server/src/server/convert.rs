use crate::app::PriceAppError;

impl From<PriceAppError> for tonic::Status {
    fn from(err: PriceAppError) -> Self {
        use PriceAppError::*;
        match err {
            CurrencyError(err) => tonic::Status::new(tonic::Code::Unknown, format!("{}", err)),
            SubscriberError(err) => tonic::Status::new(tonic::Code::Unknown, format!("{}", err)),
            ExchangePriceCacheError(err) => {
                tonic::Status::new(tonic::Code::Unknown, format!("{}", err))
            }
        }
    }
}
