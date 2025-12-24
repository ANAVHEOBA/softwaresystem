use axum::{extract::State, http::StatusCode, Json};
use std::env;
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

fn get_default_model() -> String {
    env::var("DEFAULT_MODEL").unwrap_or_else(|_| "xiaomi/mimo-v2-flash:free".to_string())
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

    let llm = LlmClient::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let model = payload.model.unwrap_or_else(get_default_model);

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

    let llm = LlmClient::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let model = payload.model.unwrap_or_else(get_default_model);

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

    let llm = LlmClient::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse { message: e.to_string() }),
        )
    })?;

    let model = payload.model.unwrap_or_else(get_default_model);

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
        ModelInfo {
            id: AiModel::MimoV2Flash.as_str().to_string(),
            name: "MiMo-V2-Flash".to_string(),
            description: "Xiaomi's 309B MoE model, excels at reasoning and coding".to_string(),
            context_length: 262144,
        },
        ModelInfo {
            id: AiModel::NemotronNano.as_str().to_string(),
            name: "Nemotron 3 Nano 30B".to_string(),
            description: "NVIDIA's efficient 30B MoE for agentic AI systems".to_string(),
            context_length: 256000,
        },
        ModelInfo {
            id: AiModel::Devstral.as_str().to_string(),
            name: "Devstral 2".to_string(),
            description: "Mistral's 123B coding specialist with 256K context".to_string(),
            context_length: 262144,
        },
        ModelInfo {
            id: AiModel::DeepSeekNex.as_str().to_string(),
            name: "DeepSeek V3.1 Nex N1".to_string(),
            description: "Nex AGI's flagship model for agent autonomy and tool use".to_string(),
            context_length: 131072,
        },
        ModelInfo {
            id: AiModel::KatCoderPro.as_str().to_string(),
            name: "KAT-Coder-Pro V1".to_string(),
            description: "KwaiKAT's advanced agentic coding model, 73.4% on SWE-Bench".to_string(),
            context_length: 256000,
        },
    ];

    Json(ModelsResponse { models })
}
