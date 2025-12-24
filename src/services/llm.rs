use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

use crate::modules::ai::schema::UsageInfo;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Missing API key")]
    MissingApiKey,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    id: String,
    choices: Vec<ChatChoice>,
    usage: Option<ApiUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

pub struct LlmResponse {
    pub id: String,
    pub content: String,
    pub usage: Option<UsageInfo>,
}

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl LlmClient {
    pub fn new() -> Result<Self, LlmError> {
        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| LlmError::MissingApiKey)?;
        let base_url =
            env::var("OPENROUTER_BASE_URL").unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        Ok(Self {
            client: Client::new(),
            base_url,
            api_key,
        })
    }

    pub async fn complete(
        &self,
        prompt: &str,
        model: &str,
        system_prompt: Option<&str>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<LlmResponse, LlmError> {
        let mut messages = Vec::new();

        if let Some(sys) = system_prompt {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: sys.to_string(),
            });
        }

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            max_tokens,
            temperature,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://cleuly.app")
            .header("X-Title", "Cleuly")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                return Err(LlmError::ApiError(error_response.error.message));
            }
            return Err(LlmError::ApiError(error_text));
        }

        let chat_response: ChatResponse = response.json().await?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::InvalidResponse("No choices in response".to_string()))?;

        let usage = chat_response.usage.map(|u| UsageInfo {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LlmResponse {
            id: chat_response.id,
            content,
            usage,
        })
    }

    pub async fn suggest(&self, context: &str, model: &str, suggestion_type: Option<&str>) -> Result<LlmResponse, LlmError> {
        let system_prompt = match suggestion_type {
            Some("interview") => "You are an expert interview coach. Based on the conversation context, provide concise, helpful response suggestions. Be direct and actionable.",
            Some("meeting") => "You are a meeting assistant. Analyze the conversation and suggest relevant talking points, questions, or responses.",
            Some("coding") => "You are a coding assistant. Provide code suggestions, fixes, or explanations based on the context.",
            _ => "You are a helpful AI assistant. Analyze the context and provide useful suggestions for how to respond or proceed.",
        };

        let prompt = format!(
            "Based on this context, provide 2-3 helpful suggestions:\n\n{}",
            context
        );

        self.complete(&prompt, model, Some(system_prompt), Some(500), Some(0.7)).await
    }

    pub async fn analyze(&self, text: &str, model: &str, analysis_type: Option<&str>) -> Result<LlmResponse, LlmError> {
        let system_prompt = match analysis_type {
            Some("sentiment") => "Analyze the sentiment and emotional tone of the text. Provide a brief analysis.",
            Some("intent") => "Identify the speaker's intent and underlying goals in this text.",
            Some("summary") => "Provide a concise summary of the key points.",
            Some("technical") => "Analyze the technical content and provide insights.",
            _ => "Analyze this text and provide useful insights about its content, tone, and meaning.",
        };

        let prompt = format!("Analyze the following:\n\n{}", text);

        self.complete(&prompt, model, Some(system_prompt), Some(500), Some(0.5)).await
    }
}
