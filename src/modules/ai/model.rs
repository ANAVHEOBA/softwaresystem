use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::schema::UsageInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiCompletion {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub model: String,
    pub response: String,
    pub usage: Option<UsageInfo>,
    pub request_type: String,
    pub created_at: DateTime<Utc>,
}

impl AiCompletion {
    pub fn new(
        prompt: String,
        system_prompt: Option<String>,
        model: String,
        response: String,
        usage: Option<UsageInfo>,
        request_type: String,
    ) -> Self {
        Self {
            id: None,
            prompt,
            system_prompt,
            model,
            response,
            usage,
            request_type,
            created_at: Utc::now(),
        }
    }
}
