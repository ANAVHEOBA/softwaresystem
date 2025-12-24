use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::modules::transcription::controller;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/transcription", post(controller::create_transcription))
        .route("/api/transcription/{id}", get(controller::get_transcription))
        .route("/api/transcription/{id}", delete(controller::delete_transcription))
        .route("/api/transcriptions", get(controller::list_transcriptions))
}
