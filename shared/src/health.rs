use futures::channel::{mpsc::*, oneshot};

pub type HealthCheckResponse = Result<(), String>;
pub type HealthCheckTrigger = UnboundedReceiver<oneshot::Sender<HealthCheckResponse>>;
pub type HealthChecker = UnboundedSender<oneshot::Sender<HealthCheckResponse>>;
