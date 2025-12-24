use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiModel {
    #[serde(rename = "xiaomi/mimo-v2-flash:free")]
    MimoV2Flash,
    #[serde(rename = "nvidia/nemotron-3-nano-30b-a3b:free")]
    NemotronNano,
    #[serde(rename = "mistralai/devstral-2512:free")]
    Devstral,
    #[serde(rename = "nex-agi/deepseek-v3.1-nex-n1:free")]
    DeepSeekNex,
    #[serde(rename = "kwaipilot/kat-coder-pro:free")]
    KatCoderPro,
}

impl AiModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiModel::MimoV2Flash => "xiaomi/mimo-v2-flash:free",
            AiModel::NemotronNano => "nvidia/nemotron-3-nano-30b-a3b:free",
            AiModel::Devstral => "mistralai/devstral-2512:free",
            AiModel::DeepSeekNex => "nex-agi/deepseek-v3.1-nex-n1:free",
            AiModel::KatCoderPro => "kwaipilot/kat-coder-pro:free",
        }
    }

    pub fn all() -> Vec<AiModel> {
        vec![
            AiModel::MimoV2Flash,
            AiModel::NemotronNano,
            AiModel::Devstral,
            AiModel::DeepSeekNex,
            AiModel::KatCoderPro,
        ]
    }
}

impl Default for AiModel {
    fn default() -> Self {
        AiModel::MimoV2Flash
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CompleteRequest {
    #[validate(length(min = 1, message = "Prompt cannot be empty"))]
    pub prompt: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SuggestRequest {
    #[validate(length(min = 1, message = "Context cannot be empty"))]
    pub context: String,
    pub model: Option<String>,
    pub suggestion_type: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AnalyzeRequest {
    #[validate(length(min = 1, message = "Text cannot be empty"))]
    pub text: String,
    pub model: Option<String>,
    pub analysis_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AiResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub usage: Option<UsageInfo>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub context_length: u32,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
