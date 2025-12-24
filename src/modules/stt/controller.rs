use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use bson::oid::ObjectId;
use std::env;

use crate::modules::session::crud::SessionCrud;
use crate::modules::session::model::Message;
use crate::modules::stt::{
    crud::SttCrud,
    model::SttTranscription,
    schema::{
        MessageResponse, TranscribeQuery, TranscribeResponse,
        TranscribeWithAiResponse, TranscriptionListResponse,
    },
};
use crate::services::llm::LlmClient;
use crate::services::stt::SttClient;
use crate::AppState;

fn to_response(t: &SttTranscription) -> TranscribeResponse {
    TranscribeResponse {
        id: t.id.map(|id| id.to_hex()).unwrap_or_default(),
        text: t.text.clone(),
        language: t.language.clone(),
        duration: t.duration,
        model: t.model.clone(),
        created_at: t.created_at_rfc3339(),
    }
}

pub async fn transcribe(
    State(state): State<AppState>,
    Query(query): Query<TranscribeQuery>,
    mut multipart: Multipart,
) -> Result<Json<TranscribeResponse>, (StatusCode, Json<MessageResponse>)> {
    // Extract audio file from multipart
    let mut audio_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut file_size: Option<u64> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: format!("Failed to read multipart: {}", e) }),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" || name == "audio" {
            file_name = field.file_name().map(|s| s.to_string());
            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(MessageResponse { message: format!("Failed to read file: {}", e) }),
                )
            })?;
            file_size = Some(data.len() as u64);
            audio_data = Some(data.to_vec());
        }
    }

    let audio_data = audio_data.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: "No audio file provided".to_string() }),
        )
    })?;

    let file_name = file_name.unwrap_or_else(|| "audio.wav".to_string());

    // Validate file extension
    let extension = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
    if !SttClient::supported_formats().contains(&extension.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MessageResponse {
                message: format!(
                    "Unsupported audio format. Supported: {:?}",
                    SttClient::supported_formats()
                ),
            }),
        ));
    }

    // Transcribe
    let stt = SttClient::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let result = stt
        .transcribe(audio_data, &file_name, query.language.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse { message: e.to_string() }),
            )
        })?;

    // Save to database
    let crud = SttCrud::new(&state.db);
    let transcription = SttTranscription::new(
        result.text.clone(),
        result.language.clone(),
        result.duration,
        result.model.clone(),
        Some(file_name),
        file_size,
        query.session_id.clone(),
    );

    let id = crud.create(transcription.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    // If session_id provided, add to session
    if let Some(session_id) = query.session_id {
        if let Ok(oid) = ObjectId::parse_str(&session_id) {
            let session_crud = SessionCrud::new(&state.db, state.redis.clone());
            let message = Message::user(result.text.clone());
            let _ = session_crud.add_message(&oid, message).await;
        }
    }

    Ok(Json(TranscribeResponse {
        id: id.to_hex(),
        text: result.text,
        language: result.language,
        duration: result.duration,
        model: result.model,
        created_at: transcription.created_at_rfc3339(),
    }))
}

pub async fn transcribe_and_respond(
    State(state): State<AppState>,
    Query(query): Query<TranscribeQuery>,
    mut multipart: Multipart,
) -> Result<Json<TranscribeWithAiResponse>, (StatusCode, Json<MessageResponse>)> {
    // Extract audio file from multipart
    let mut audio_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut file_size: Option<u64> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: format!("Failed to read multipart: {}", e) }),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" || name == "audio" {
            file_name = field.file_name().map(|s| s.to_string());
            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(MessageResponse { message: format!("Failed to read file: {}", e) }),
                )
            })?;
            file_size = Some(data.len() as u64);
            audio_data = Some(data.to_vec());
        }
    }

    let audio_data = audio_data.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: "No audio file provided".to_string() }),
        )
    })?;

    let file_name = file_name.unwrap_or_else(|| "audio.wav".to_string());

    // Transcribe
    let stt = SttClient::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let result = stt
        .transcribe(audio_data, &file_name, query.language.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse { message: e.to_string() }),
            )
        })?;

    // Get AI response
    let llm = LlmClient::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let model = env::var("DEFAULT_MODEL").unwrap_or_else(|_| "xiaomi/mimo-v2-flash:free".to_string());

    let ai_result = llm
        .suggest(&result.text, &model, Some("interview"))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse { message: e.to_string() }),
            )
        })?;

    // Save to database
    let crud = SttCrud::new(&state.db);
    let mut transcription = SttTranscription::new(
        result.text.clone(),
        result.language.clone(),
        result.duration,
        result.model.clone(),
        Some(file_name),
        file_size,
        query.session_id.clone(),
    );
    transcription.ai_response = Some(ai_result.content.clone());

    let id = crud.create(transcription.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    // If session_id provided, add both messages to session
    if let Some(session_id) = query.session_id {
        if let Ok(oid) = ObjectId::parse_str(&session_id) {
            let session_crud = SessionCrud::new(&state.db, state.redis.clone());
            let user_msg = Message::user(result.text.clone());
            let assistant_msg = Message::assistant(ai_result.content.clone());
            let _ = session_crud.add_message(&oid, user_msg).await;
            let _ = session_crud.add_message(&oid, assistant_msg).await;
        }
    }

    Ok(Json(TranscribeWithAiResponse {
        id: id.to_hex(),
        transcription: result.text,
        ai_response: ai_result.content,
        language: result.language,
        duration: result.duration,
        model: result.model,
        created_at: transcription.created_at_rfc3339(),
    }))
}

pub async fn get_transcription(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TranscribeResponse>, (StatusCode, Json<MessageResponse>)> {
    let oid = ObjectId::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: "Invalid ID format".to_string() }),
        )
    })?;

    let crud = SttCrud::new(&state.db);

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
    let crud = SttCrud::new(&state.db);

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

    let crud = SttCrud::new(&state.db);

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

pub async fn supported_formats() -> Json<Vec<&'static str>> {
    Json(SttClient::supported_formats())
}
