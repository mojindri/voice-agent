mod api;
mod audio;
mod config;
mod error;
mod metrics;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    api::{
        create_router,
        models::AppState,
    },
    audio::processor::AudioProcessor,
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
    let config = Config::from_env()?;
    info!("Configuration loaded");

    // Initialize metrics
    metrics::setup_metrics().unwrap();

    // Initialize audio processor
    let audio_processor = Arc::new(Mutex::new(AudioProcessor::new(
        48000, // sample rate
        1,     // channels (mono)
    )));

    // Initialize metrics
    let metrics = Arc::new(Metrics::new());

    // Create app state
    let state = AppState {
        audio_processor,
        metrics,
    };

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
