use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTranscriptionRequest {
    #[validate(length(min = 1, message = "Text cannot be empty"))]
    pub text: String,
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TranscriptionResponse {
    pub id: String,
    pub text: String,
    pub source: Option<String>,
    pub ai_response: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TranscriptionListResponse {
    pub data: Vec<TranscriptionResponse>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
