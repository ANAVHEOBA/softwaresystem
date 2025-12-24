use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SttTranscription {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub text: String,
    pub language: Option<String>,
    pub duration: Option<f32>,
    pub model: String,
    pub file_name: Option<String>,
    pub file_size: Option<u64>,
    pub session_id: Option<String>,
    pub ai_response: Option<String>,
    pub created_at: bson::DateTime,
}

impl SttTranscription {
    pub fn new(
        text: String,
        language: Option<String>,
        duration: Option<f32>,
        model: String,
        file_name: Option<String>,
        file_size: Option<u64>,
        session_id: Option<String>,
    ) -> Self {
        Self {
            id: None,
            text,
            language,
            duration,
            model,
            file_name,
            file_size,
            session_id,
            ai_response: None,
            created_at: bson::DateTime::now(),
        }
    }

    pub fn created_at_rfc3339(&self) -> String {
        self.created_at.try_to_rfc3339_string().unwrap_or_default()
    }
}
