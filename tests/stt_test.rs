use axum::http::StatusCode;
use axum::Router;
use axum_test::TestServer;
use cleuly::{config, modules, AppState};

async fn setup_test_server() -> TestServer {
    dotenvy::dotenv().ok();

    let db = config::database::connect().await;
    let redis = config::redis::connect().await;

    let state = AppState { db, redis };

    let app = Router::new()
        .merge(modules::stt::routes::routes())
        .with_state(state);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_supported_formats() {
    let server = setup_test_server().await;

    let response = server.get("/api/stt/formats").await;

    response.assert_status(StatusCode::OK);

    let formats: Vec<String> = response.json();
    assert!(formats.contains(&"mp3".to_string()));
    assert!(formats.contains(&"wav".to_string()));
    assert!(formats.contains(&"webm".to_string()));
}

#[tokio::test]
async fn test_transcribe_no_file() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/stt/transcribe")
        .await;

    // Should fail because no file provided
    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_transcriptions() {
    let server = setup_test_server().await;

    let response = server.get("/api/stt/transcriptions").await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

#[tokio::test]
async fn test_get_transcription_not_found() {
    let server = setup_test_server().await;

    let response = server.get("/api/stt/transcription/507f1f77bcf86cd799439011").await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_transcription_invalid_id() {
    let server = setup_test_server().await;

    let response = server.get("/api/stt/transcription/invalid-id").await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_transcription_not_found() {
    let server = setup_test_server().await;

    let response = server.delete("/api/stt/transcription/507f1f77bcf86cd799439011").await;

    response.assert_status(StatusCode::NOT_FOUND);
}

// Note: Full transcription tests require:
// 1. GROQ_API_KEY to be set
// 2. Actual audio file to upload
// These are integration tests that should be run manually
