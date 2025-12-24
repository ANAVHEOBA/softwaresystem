use axum::{extract::State, http::StatusCode, Json};
use validator::Validate;

use crate::modules::ai::{
    crud::AiCrud,
    model::AiCompletion,
    schema::{
        AiModel, AiResponse, AnalyzeRequest, CompleteRequest, MessageResponse,
        ModelInfo, ModelsResponse, SuggestRequest,
    },
};
use crate::services::llm::LlmClient;
use crate::AppState;

fn create_llm_client() -> Result<LlmClient, crate::services::llm::LlmError> {
    // Try Groq first (faster), fall back to OpenRouter
    LlmClient::new_groq().or_else(|_| LlmClient::new())
}

pub async fn complete(
    State(state): State<AppState>,
    Json(payload): Json<CompleteRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<MessageResponse>)> {
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: e.to_string() }),
        ));
    }

    let llm = create_llm_client().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let model = payload.model.unwrap_or_else(|| llm.default_model().to_string());

    let result = llm
        .complete(
            &payload.prompt,
            &model,
            payload.system_prompt.as_deref(),
            payload.max_tokens,
            payload.temperature,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse { message: e.to_string() }),
            )
        })?;

    // Store in database
    let crud = AiCrud::new(&state.db);
    let completion = AiCompletion::new(
        payload.prompt,
        payload.system_prompt,
        model.clone(),
        result.content.clone(),
        result.usage.clone(),
        "complete".to_string(),
    );

    let id = crud.create(completion.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    Ok(Json(AiResponse {
        id: id.to_hex(),
        model,
        content: result.content,
        usage: result.usage,
        created_at: completion.created_at.to_rfc3339(),
    }))
}

pub async fn suggest(
    State(state): State<AppState>,
    Json(payload): Json<SuggestRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<MessageResponse>)> {
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: e.to_string() }),
        ));
    }

    let llm = create_llm_client().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let model = payload.model.unwrap_or_else(|| llm.default_model().to_string());

    let result = llm
        .suggest(&payload.context, &model, payload.suggestion_type.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse { message: e.to_string() }),
            )
        })?;

    // Store in database
    let crud = AiCrud::new(&state.db);
    let completion = AiCompletion::new(
        payload.context,
        None,
        model.clone(),
        result.content.clone(),
        result.usage.clone(),
        "suggest".to_string(),
    );

    let id = crud.create(completion.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    Ok(Json(AiResponse {
        id: id.to_hex(),
        model,
        content: result.content,
        usage: result.usage,
        created_at: completion.created_at.to_rfc3339(),
    }))
}

pub async fn analyze(
    State(state): State<AppState>,
    Json(payload): Json<AnalyzeRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<MessageResponse>)> {
    if let Err(e) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(MessageResponse { message: e.to_string() }),
        ));
    }

    let llm = create_llm_client().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let model = payload.model.unwrap_or_else(|| llm.default_model().to_string());

    let result = llm
        .analyze(&payload.text, &model, payload.analysis_type.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse { message: e.to_string() }),
            )
        })?;

    // Store in database
    let crud = AiCrud::new(&state.db);
    let completion = AiCompletion::new(
        payload.text,
        None,
        model.clone(),
        result.content.clone(),
        result.usage.clone(),
        "analyze".to_string(),
    );

    let id = crud.create(completion.clone()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    Ok(Json(AiResponse {
        id: id.to_hex(),
        model,
        content: result.content,
        usage: result.usage,
        created_at: completion.created_at.to_rfc3339(),
    }))
}

pub async fn list_models() -> Json<ModelsResponse> {
    let models = vec![
        // Groq models (fastest - ~500ms)
        ModelInfo {
            id: "llama-3.1-8b-instant".to_string(),
            name: "Llama 3.1 8B Instant (Groq)".to_string(),
            description: "Fastest model (~500ms). Great for quick coding help.".to_string(),
            context_length: 131072,
        },
        ModelInfo {
            id: "llama-3.3-70b-versatile".to_string(),
            name: "Llama 3.3 70B (Groq)".to_string(),
            description: "Larger Groq model for complex tasks. Slower but smarter.".to_string(),
            context_length: 131072,
        },
        // OpenRouter models (free tier)
        ModelInfo {
            id: AiModel::NemotronNano.as_str().to_string(),
            name: "Nemotron 3 Nano 30B".to_string(),
            description: "Fast free model (~700ms). NVIDIA's efficient MoE.".to_string(),
            context_length: 256000,
        },
        ModelInfo {
            id: "google/gemma-3-27b-it:free".to_string(),
            name: "Gemma 3 27B".to_string(),
            description: "Google's fast model (~900ms). Good quality.".to_string(),
            context_length: 131072,
        },
        ModelInfo {
            id: AiModel::KatCoderPro.as_str().to_string(),
            name: "KAT-Coder-Pro V1".to_string(),
            description: "Coding specialist (~1200ms). 73.4% on SWE-Bench.".to_string(),
            context_length: 256000,
        },
        ModelInfo {
            id: AiModel::Devstral.as_str().to_string(),
            name: "Devstral 2".to_string(),
            description: "Mistral coding model (~2300ms). 256K context.".to_string(),
            context_length: 262144,
        },
    ];

    Json(ModelsResponse { models })
}
