mod error;

pub use error::BrokerAppError;

pub struct BrokerApp {}

impl BrokerApp {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create_user_trade(&self) -> Result<(), BrokerAppError> {
        Ok(())
    }
}
