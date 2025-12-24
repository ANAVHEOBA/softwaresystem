use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transcription {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub text: String,
    pub source: Option<String>,
    pub ai_response: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transcription {
    pub fn new(text: String, source: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            text,
            source,
            ai_response: None,
            created_at: now,
            updated_at: now,
        }
    }
}
