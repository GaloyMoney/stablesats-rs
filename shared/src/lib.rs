#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod macros;
pub mod payload;
pub mod pubsub;
pub mod time;
pub mod tracing;

#[derive(Debug)]
pub struct ParseIdError(pub &'static str);
