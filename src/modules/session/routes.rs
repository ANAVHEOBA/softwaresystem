use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::modules::session::controller;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/session", post(controller::create_session))
        .route("/api/session/{id}", get(controller::get_session))
        .route("/api/session/{id}", delete(controller::delete_session))
        .route("/api/session/{id}/message", post(controller::add_message))
        .route("/api/session/{id}/chat", post(controller::chat))
        .route("/api/sessions", get(controller::list_sessions))
}
