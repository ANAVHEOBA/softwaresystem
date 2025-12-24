use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TranscribeResponse {
    pub id: String,
    pub text: String,
    pub language: Option<String>,
    pub duration: Option<f32>,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TranscribeWithAiResponse {
    pub id: String,
    pub transcription: String,
    pub ai_response: String,
    pub language: Option<String>,
    pub duration: Option<f32>,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TranscriptionListResponse {
    pub data: Vec<TranscribeResponse>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TranscribeQuery {
    pub language: Option<String>,
    pub session_id: Option<String>,
}
