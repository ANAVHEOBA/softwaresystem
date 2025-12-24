use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: bson::DateTime,
}

impl Message {
    pub fn new(role: String, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: bson::DateTime::now(),
        }
    }

    pub fn user(content: String) -> Self {
        Self::new("user".to_string(), content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new("assistant".to_string(), content)
    }

    pub fn system(content: String) -> Self {
        Self::new("system".to_string(), content)
    }

    pub fn timestamp_rfc3339(&self) -> String {
        self.timestamp.try_to_rfc3339_string().unwrap_or_default()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: Option<String>,
    pub session_type: String,
    pub messages: Vec<Message>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
}

impl Session {
    pub fn new(title: Option<String>, session_type: Option<String>, metadata: Option<serde_json::Value>) -> Self {
        let now = bson::DateTime::now();
        Self {
            id: None,
            title,
            session_type: session_type.unwrap_or_else(|| "general".to_string()),
            messages: Vec::new(),
            metadata,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = bson::DateTime::now();
    }

    pub fn get_context_messages(&self, limit: usize) -> Vec<&Message> {
        let len = self.messages.len();
        if len <= limit {
            self.messages.iter().collect()
        } else {
            self.messages.iter().skip(len - limit).collect()
        }
    }

    pub fn created_at_rfc3339(&self) -> String {
        self.created_at.try_to_rfc3339_string().unwrap_or_default()
    }

    pub fn updated_at_rfc3339(&self) -> String {
        self.updated_at.try_to_rfc3339_string().unwrap_or_default()
    }
}
