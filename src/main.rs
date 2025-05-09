mod api;
mod audio;
mod config;
mod error;
mod metrics;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use crate::{
    api::{create_router, models::AppState},
    config::Config,
    metrics::Metrics,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Logging initialized");

    // Load environment variables
    dotenv::dotenv().ok();
    info!("Environment variables loaded");

    // Load configuration
    Config::from_env()?;
    info!("Configuration loaded");

    // Initialize metrics
    metrics::setup_metrics().unwrap();

    // Initialize metrics
    let metrics = Arc::new(Metrics::new());

    // Create app state
    let state = AppState { metrics };

    // Create router
    let app = create_router(state);
    info!("Router created");

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
