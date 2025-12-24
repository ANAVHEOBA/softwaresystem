use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use bson::oid::ObjectId;
use validator::Validate;

use crate::modules::transcription::{
    crud::TranscriptionCrud,
    model::Transcription,
    schema::{CreateTranscriptionRequest, MessageResponse, TranscriptionListResponse, TranscriptionResponse},
};
use crate::AppState;

fn to_response(t: &Transcription) -> TranscriptionResponse {
    TranscriptionResponse {
        id: t.id.map(|id| id.to_hex()).unwrap_or_default(),
        text: t.text.clone(),
        source: t.source.clone(),
        ai_response: t.ai_response.clone(),
        created_at: t.created_at.to_rfc3339(),
    }
}

pub async fn create_transcription(
    State(state): State<AppState>,
    Json(payload): Json<CreateTranscriptionRequest>,
) -> Result<(StatusCode, Json<TranscriptionResponse>), (StatusCode, Json<MessageResponse>)> {
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: e.to_string() }),
        ));
    }

    let crud = TranscriptionCrud::new(&state.db);
    let transcription = Transcription::new(payload.text, payload.source);

    match crud.create(transcription.clone()).await {
        Ok(id) => {
            let mut response = to_response(&transcription);
            response.id = id.to_hex();
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )),
    }
}

pub async fn get_transcription(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TranscriptionResponse>, (StatusCode, Json<MessageResponse>)> {
    let oid = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: "Invalid ID format".to_string() }),
        )
    })?;

    let crud = TranscriptionCrud::new(&state.db);

    match crud.find_by_id(&oid).await {
        Ok(Some(t)) => Ok(Json(to_response(&t))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(MessageResponse { message: "Transcription not found".to_string() }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )),
    }
}

pub async fn list_transcriptions(
    State(state): State<AppState>,
) -> Result<Json<TranscriptionListResponse>, (StatusCode, Json<MessageResponse>)> {
    let crud = TranscriptionCrud::new(&state.db);

    let transcriptions = crud.find_all(50).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let total = crud.count().await.unwrap_or(0);

    Ok(Json(TranscriptionListResponse {
        data: transcriptions.iter().map(to_response).collect(),
        total,
    }))
}

pub async fn delete_transcription(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<MessageResponse>)> {
    let oid = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: "Invalid ID format".to_string() }),
        )
    })?;

    let crud = TranscriptionCrud::new(&state.db);

    match crud.delete(&oid).await {
        Ok(true) => Ok(Json(MessageResponse { message: "Deleted successfully".to_string() })),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(MessageResponse { message: "Transcription not found".to_string() }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )),
    }
}
