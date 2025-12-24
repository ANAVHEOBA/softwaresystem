use axum::{
    routing::{get, post},
    Router,
};

use crate::modules::ai::controller;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/ai/complete", post(controller::complete))
        .route("/api/ai/suggest", post(controller::suggest))
        .route("/api/ai/analyze", post(controller::analyze))
        .route("/api/ai/models", get(controller::list_models))
}
