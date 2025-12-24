use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use bson::oid::ObjectId;
use std::env;
use validator::Validate;

use crate::modules::session::{
    crud::SessionCrud,
    model::{Message, Session},
    schema::{
        AddMessageRequest, ChatRequest, ChatResponse, CreateSessionRequest,
        MessageResponse, MessageResponse2, SessionListResponse, SessionResponse,
        SessionSummary,
    },
};
use crate::services::llm::LlmClient;
use crate::AppState;

fn to_message_response(m: &Message) -> MessageResponse {
    MessageResponse {
        role: m.role.clone(),
        content: m.content.clone(),
        timestamp: m.timestamp_rfc3339(),
    }
}

fn to_session_response(s: &Session) -> SessionResponse {
    SessionResponse {
        id: s.id.map(|id| id.to_hex()).unwrap_or_default(),
        title: s.title.clone(),
        session_type: s.session_type.clone(),
        messages: s.messages.iter().map(to_message_response).collect(),
        message_count: s.messages.len(),
        created_at: s.created_at_rfc3339(),
        updated_at: s.updated_at_rfc3339(),
    }
}

fn to_session_summary(s: &Session) -> SessionSummary {
    SessionSummary {
        id: s.id.map(|id| id.to_hex()).unwrap_or_default(),
        title: s.title.clone(),
        session_type: s.session_type.clone(),
        message_count: s.messages.len(),
        created_at: s.created_at_rfc3339(),
        updated_at: s.updated_at_rfc3339(),
    }
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), (StatusCode, Json<MessageResponse2>)> {
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MessageResponse2 { message: e.to_string() }),
        ));
    }

    let crud = SessionCrud::new(&state.db, state.redis.clone());
    let session = Session::new(payload.title, payload.session_type, payload.metadata);

    match crud.create(session.clone()).await {
        Ok(id) => {
            let mut response = to_session_response(&session);
            response.id = id.to_hex();
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )),
    }
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionResponse>, (StatusCode, Json<MessageResponse2>)> {
    let oid = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse2 { message: "Invalid ID format".to_string() }),
        )
    })?;

    let crud = SessionCrud::new(&state.db, state.redis.clone());

    match crud.find_by_id(&oid).await {
        Ok(Some(s)) => Ok(Json(to_session_response(&s))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(MessageResponse2 { message: "Session not found".to_string() }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )),
    }
}

pub async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<SessionListResponse>, (StatusCode, Json<MessageResponse2>)> {
    let crud = SessionCrud::new(&state.db, state.redis.clone());

    let sessions = crud.find_all(50).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )
    })?;

    let total = crud.count().await.unwrap_or(0);

    Ok(Json(SessionListResponse {
        data: sessions.iter().map(to_session_summary).collect(),
        total,
    }))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MessageResponse2>, (StatusCode, Json<MessageResponse2>)> {
    let oid = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse2 { message: "Invalid ID format".to_string() }),
        )
    })?;

    let crud = SessionCrud::new(&state.db, state.redis.clone());

    match crud.delete(&oid).await {
        Ok(true) => Ok(Json(MessageResponse2 { message: "Deleted successfully".to_string() })),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(MessageResponse2 { message: "Session not found".to_string() }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )),
    }
}

pub async fn add_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AddMessageRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<MessageResponse2>)> {
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MessageResponse2 { message: e.to_string() }),
        ));
    }

    let oid = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse2 { message: "Invalid ID format".to_string() }),
        )
    })?;

    let crud = SessionCrud::new(&state.db, state.redis.clone());
    let message = Message::new(payload.role, payload.content);

    match crud.add_message(&oid, message.clone()).await {
        Ok(true) => Ok(Json(to_message_response(&message))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(MessageResponse2 { message: "Session not found".to_string() }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )),
    }
}

pub async fn chat(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<MessageResponse2>)> {
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MessageResponse2 { message: e.to_string() }),
        ));
    }

    let oid = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse2 { message: "Invalid ID format".to_string() }),
        )
    })?;

    let crud = SessionCrud::new(&state.db, state.redis.clone());

    // Get session
    let session = crud.find_by_id(&oid).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )
    })?;

    let session = session.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(MessageResponse2 { message: "Session not found".to_string() }),
        )
    })?;

    // Build context from previous messages
    let context_messages = session.get_context_messages(10);
    let context = context_messages
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = if context.is_empty() {
        payload.message.clone()
    } else {
        format!("Previous conversation:\n{}\n\nUser: {}", context, payload.message)
    };

    // Get AI response
    let llm = LlmClient::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )
    })?;

    let model = payload.model.unwrap_or_else(|| {
        env::var("DEFAULT_MODEL").unwrap_or_else(|_| "xiaomi/mimo-v2-flash:free".to_string())
    });

    let system_prompt = payload.system_prompt.as_deref().unwrap_or(
        "You are Cleuly, a helpful AI assistant. Provide concise, helpful responses."
    );

    let result = llm
        .complete(&prompt, &model, Some(system_prompt), Some(1000), Some(0.7))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse2 { message: e.to_string() }),
            )
        })?;

    // Save user message and AI response
    let user_message = Message::user(payload.message);
    let assistant_message = Message::assistant(result.content.clone());

    crud.add_message(&oid, user_message.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )
    })?;

    crud.add_message(&oid, assistant_message.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse2 { message: e.to_string() }),
        )
    })?;

    Ok(Json(ChatResponse {
        session_id: id,
        message: to_message_response(&user_message),
        response: to_message_response(&assistant_message),
        model,
    }))
}
