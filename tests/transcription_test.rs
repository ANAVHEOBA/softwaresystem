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
        .merge(modules::transcription::routes::routes())
        .with_state(state);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_create_transcription_success() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/transcription")
        .json(&json!({
            "text": "Hello, this is a test transcription",
            "source": "microphone"
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let body: serde_json::Value = response.json();
    assert!(!body["id"].as_str().unwrap().is_empty());
    assert_eq!(body["text"], "Hello, this is a test transcription");
    assert_eq!(body["source"], "microphone");
}

#[tokio::test]
async fn test_create_transcription_empty_text_fails() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/transcription")
        .json(&json!({
            "text": "",
            "source": "microphone"
        }))
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_transcription_not_found() {
    let server = setup_test_server().await;

    let response = server.get("/api/transcription/507f1f77bcf86cd799439011").await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_transcription_invalid_id() {
    let server = setup_test_server().await;

    let response = server.get("/api/transcription/invalid-id").await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_and_get_transcription() {
    let server = setup_test_server().await;

    // Create
    let create_response = server
        .post("/api/transcription")
        .json(&json!({
            "text": "Test get transcription",
            "source": "test"
        }))
        .await;

    create_response.assert_status(StatusCode::CREATED);
    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // Get
    let get_response = server.get(&format!("/api/transcription/{}", id)).await;

    get_response.assert_status(StatusCode::OK);
    let fetched: serde_json::Value = get_response.json();
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["text"], "Test get transcription");
}

#[tokio::test]
async fn test_list_transcriptions() {
    let server = setup_test_server().await;

    let response = server.get("/api/transcriptions").await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

#[tokio::test]
async fn test_delete_transcription() {
    let server = setup_test_server().await;

    // Create first
    let create_response = server
        .post("/api/transcription")
        .json(&json!({
            "text": "To be deleted",
            "source": "test"
        }))
        .await;

    create_response.assert_status(StatusCode::CREATED);
    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // Delete
    let delete_response = server.delete(&format!("/api/transcription/{}", id)).await;

    delete_response.assert_status(StatusCode::OK);

    // Verify deleted
    let get_response = server.get(&format!("/api/transcription/{}", id)).await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_transcription_not_found() {
    let server = setup_test_server().await;

    let response = server.delete("/api/transcription/507f1f77bcf86cd799439011").await;

    response.assert_status(StatusCode::NOT_FOUND);
}
