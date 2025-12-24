use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::modules::stt::controller;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/stt/transcribe", post(controller::transcribe))
        .route("/api/stt/transcribe-ai", post(controller::transcribe_and_respond))
        .route("/api/stt/transcription/{id}", get(controller::get_transcription))
        .route("/api/stt/transcription/{id}", delete(controller::delete_transcription))
        .route("/api/stt/transcriptions", get(controller::list_transcriptions))
        .route("/api/stt/formats", get(controller::supported_formats))
}
