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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LlmProvider {
    OpenRouter,
    Groq,
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
    provider: LlmProvider,
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
            provider: LlmProvider::OpenRouter,
        })
    }

    /// Create a Groq client for faster inference (~500ms vs ~700ms+)
    pub fn new_groq() -> Result<Self, LlmError> {
        let api_key = env::var("GROQ_API_KEY").map_err(|_| LlmError::MissingApiKey)?;
        let base_url =
            env::var("GROQ_BASE_URL").unwrap_or_else(|_| "https://api.groq.com/openai/v1".to_string());

        Ok(Self {
            client: Client::new(),
            base_url,
            api_key,
            provider: LlmProvider::Groq,
        })
    }

    /// Get the default fast model for this provider
    pub fn default_model(&self) -> &str {
        match self.provider {
            LlmProvider::Groq => "llama-3.1-8b-instant",
            LlmProvider::OpenRouter => "nvidia/nemotron-3-nano-30b-a3b:free",
        }
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

        let mut req = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        // OpenRouter requires these headers
        if self.provider == LlmProvider::OpenRouter {
            req = req
                .header("HTTP-Referer", "https://cleuly.app")
                .header("X-Title", "Cleuly");
        }

        let response = req.json(&request).send().await?;

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
            Some("interview") | Some("coding_interview") => r#"You are a real-time coding interview coach. Be EXTREMELY concise.

For coding: give optimal solution in code block, then "Time: O(?) | Space: O(?) | Pattern: [name]"
For behavioral: give 2-3 bullet points max
For system design: list 3-5 key components

NO lengthy explanations. Direct answers only."#,

            Some("leetcode") | Some("coding") => r#"You are an expert competitive programmer. Give CONCISE answers.

FORMAT:
```python
[code]
```
Time: O(?) | Space: O(?) | Pattern: [name]

NO explanations unless asked. Code only."#,

            Some("meeting") => r#"You are a meeting assistant providing real-time suggestions.

RULES:
- Give actionable responses the user can say immediately
- Keep suggestions brief (1-2 sentences each)
- Be professional but natural
- Provide 2-3 options when appropriate"#,

            _ => r#"You are Cleuly, a real-time AI assistant. Be direct, concise, and helpful. Give answers the user can use immediately."#,
        };

        let prompt = match suggestion_type {
            Some("interview") | Some("coding_interview") | Some("leetcode") | Some("coding") => {
                format!("Solve this:\n\n{}", context)
            }
            _ => format!("Help with this:\n\n{}", context),
        };

        self.complete(&prompt, model, Some(system_prompt), Some(800), Some(0.3)).await
    }

    pub async fn analyze(&self, text: &str, model: &str, analysis_type: Option<&str>) -> Result<LlmResponse, LlmError> {
        let system_prompt = match analysis_type {
            Some("sentiment") => "Analyze sentiment briefly. Format: [POSITIVE/NEGATIVE/NEUTRAL] - one line explanation.",
            Some("intent") => "Identify the speaker's intent in one sentence.",
            Some("summary") => "Summarize in 2-3 bullet points maximum.",
            Some("technical") => "Explain the technical concept concisely with a code example if relevant.",
            Some("debug") => r#"You are a debugging expert. Identify the bug, explain why it happens, and provide the fix. Be direct."#,
            _ => "Provide a brief, useful analysis.",
        };

        let prompt = format!("{}", text);

        self.complete(&prompt, model, Some(system_prompt), Some(600), Some(0.3)).await
    }
}
