use anyhow::Context;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use futures::SinkExt;
use std::{net::SocketAddr, sync::Arc};

use shared::health::HealthChecker;

async fn health(checkers: Arc<Vec<HealthChecker>>) -> impl IntoResponse {
    for checker in checkers.iter() {
        let (snd, recv) = futures::channel::oneshot::channel();
        if let Err(e) = checker.clone().send(snd).await {
            eprintln!("Couldn't send health check: {}", e);
            return StatusCode::SERVICE_UNAVAILABLE;
        }
        match tokio::time::timeout(std::time::Duration::from_millis(100), recv).await {
            Err(_) => {
                eprintln!("Health check timed out");
                return StatusCode::SERVICE_UNAVAILABLE;
            }
            Ok(Err(e)) => {
                eprintln!("Health check failed: {}", e);
                return StatusCode::SERVICE_UNAVAILABLE;
            }
            Ok(Ok(_)) => (),
        }
    }
    StatusCode::OK
}

pub async fn run(checkers: Vec<HealthChecker>) -> anyhow::Result<()> {
    let app = Router::new().route(
        "/healthz",
        get({
            let checkers = Arc::new(checkers);
            move || health(Arc::clone(&checkers))
        }),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Bind health server")
}
