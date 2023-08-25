#[allow(clippy::all)]
pub mod proto {
    tonic::include_proto!("services.bria.v1");
}

mod bria_client;
mod config;

pub use bria_client::*;
pub use config::*;
