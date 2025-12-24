use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSessionRequest {
    #[validate(length(max = 100, message = "Title too long"))]
    pub title: Option<String>,
    pub session_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddMessageRequest {
    #[validate(length(min = 1, message = "Role cannot be empty"))]
    pub role: String,
    #[validate(length(min = 1, message = "Content cannot be empty"))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChatRequest {
    #[validate(length(min = 1, message = "Message cannot be empty"))]
    pub message: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub title: Option<String>,
    pub session_type: String,
    pub messages: Vec<MessageResponse>,
    pub message_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MessageResponse {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub data: Vec<SessionSummary>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct SessionSummary {
    pub id: String,
    pub title: Option<String>,
    pub session_type: String,
    pub message_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub session_id: String,
    pub message: MessageResponse,
    pub response: MessageResponse,
    pub model: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse2 {
    pub message: String,
}
