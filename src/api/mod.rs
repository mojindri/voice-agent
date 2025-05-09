use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::api::{
    handlers::{health_check, metrics, process_voice_agent},
    models::AppState,
};

pub mod handlers;
pub mod models;

pub fn  create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/process_voice_agent", post(process_voice_agent))
        .route("/metrics", get(metrics))
        .nest_service("/static", ServeDir::new("static"))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::AllowOrigin::any())
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
