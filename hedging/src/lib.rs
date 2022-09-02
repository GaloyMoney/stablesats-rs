#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod app;
mod error;
mod job;
mod synth_usd_liability;

pub use app::*;
pub use error::*;
