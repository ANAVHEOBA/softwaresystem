use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SttError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Missing API key")]
    MissingApiKey,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    duration: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

pub struct SttResponse {
    pub text: String,
    pub language: Option<String>,
    pub duration: Option<f32>,
    pub model: String,
}

#[derive(Clone)]
pub struct SttClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl SttClient {
    pub fn new() -> Result<Self, SttError> {
        let api_key = env::var("GROQ_API_KEY").map_err(|_| SttError::MissingApiKey)?;

        if api_key.is_empty() {
            return Err(SttError::MissingApiKey);
        }

        let base_url = env::var("GROQ_BASE_URL")
            .unwrap_or_else(|_| "https://api.groq.com/openai/v1".to_string());
        let model = env::var("STT_MODEL")
            .unwrap_or_else(|_| "whisper-large-v3-turbo".to_string());

        Ok(Self {
            client: Client::new(),
            base_url,
            api_key,
            model,
        })
    }

    pub async fn transcribe(
        &self,
        audio_data: Vec<u8>,
        file_name: &str,
        language: Option<&str>,
    ) -> Result<SttResponse, SttError> {
        let mime_type = Self::get_mime_type(file_name);

        let file_part = Part::bytes(audio_data)
            .file_name(file_name.to_string())
            .mime_str(&mime_type)
            .map_err(|e| SttError::InvalidResponse(e.to_string()))?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("model", self.model.clone())
            .text("response_format", "verbose_json");

        if let Some(lang) = language {
            form = form.text("language", lang.to_string());
        }

        let response = self
            .client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                return Err(SttError::ApiError(error_response.error.message));
            }
            return Err(SttError::ApiError(error_text));
        }

        let whisper_response: WhisperResponse = response.json().await?;

        Ok(SttResponse {
            text: whisper_response.text,
            language: whisper_response.language,
            duration: whisper_response.duration,
            model: self.model.clone(),
        })
    }

    fn get_mime_type(file_name: &str) -> String {
        let extension = file_name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "webm" => "audio/webm",
            "ogg" => "audio/ogg",
            "m4a" => "audio/m4a",
            "flac" => "audio/flac",
            "mp4" => "audio/mp4",
            _ => "application/octet-stream",
        }
        .to_string()
    }

    pub fn supported_formats() -> Vec<&'static str> {
        vec!["mp3", "wav", "webm", "ogg", "m4a", "flac", "mp4"]
    }
}
