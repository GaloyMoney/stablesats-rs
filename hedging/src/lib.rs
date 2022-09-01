#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod app;
mod error;
mod job;

pub use app::*;
pub use error::*;
