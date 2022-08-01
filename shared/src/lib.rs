#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod exchange;
pub mod money;
pub mod payload;
pub mod pubsub;
pub mod time;
