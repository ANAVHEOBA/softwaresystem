use axum::http::StatusCode;
use axum::Router;
use axum_test::TestServer;
use cleuly::{config, modules, AppState};
use serde_json::json;

async fn setup_test_server() -> TestServer {
    dotenvy::dotenv().ok();

    let db = config::database::connect().await;
    let redis = config::redis::connect().await;

    let state = AppState { db, redis };

    let app = Router::new()
        .merge(modules::ai::routes::routes())
        .with_state(state);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_list_models() {
    let server = setup_test_server().await;

    let response = server.get("/api/ai/models").await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["models"].is_array());

    let models = body["models"].as_array().unwrap();
    assert_eq!(models.len(), 5);

    // Check first model has expected fields
    assert!(models[0]["id"].is_string());
    assert!(models[0]["name"].is_string());
    assert!(models[0]["description"].is_string());
    assert!(models[0]["context_length"].is_number());
}

#[tokio::test]
async fn test_complete_empty_prompt_fails() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/ai/complete")
        .json(&json!({
            "prompt": ""
        }))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_suggest_empty_context_fails() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/ai/suggest")
        .json(&json!({
            "context": ""
        }))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_analyze_empty_text_fails() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/ai/analyze")
        .json(&json!({
            "text": ""
        }))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_complete_with_valid_prompt() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/ai/complete")
        .json(&json!({
            "prompt": "Say hello in one word",
            "model": "xiaomi/mimo-v2-flash:free",
            "max_tokens": 50
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["id"].is_string());
    assert!(body["model"].is_string());
    assert!(body["content"].is_string());
    assert!(!body["content"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_suggest_with_valid_context() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/ai/suggest")
        .json(&json!({
            "context": "The interviewer asked: What is your greatest strength?",
            "suggestion_type": "interview"
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["id"].is_string());
    assert!(body["content"].is_string());
}

#[tokio::test]
async fn test_analyze_with_valid_text() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/ai/analyze")
        .json(&json!({
            "text": "I am really excited about this opportunity!",
            "analysis_type": "sentiment"
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["id"].is_string());
    assert!(body["content"].is_string());
}

#[tokio::test]
async fn test_complete_with_different_model() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/ai/complete")
        .json(&json!({
            "prompt": "What is 2+2? Answer with just the number.",
            "model": "nvidia/nemotron-3-nano-30b-a3b:free",
            "max_tokens": 10
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["model"], "nvidia/nemotron-3-nano-30b-a3b:free");
}

#[tokio::test]
async fn test_complete_with_system_prompt() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/ai/complete")
        .json(&json!({
            "prompt": "Who are you?",
            "system_prompt": "You are a helpful coding assistant named Cleuly.",
            "max_tokens": 100
        }))
        .await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["content"].is_string());
}
