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
        .merge(modules::session::routes::routes())
        .with_state(state);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_create_session() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/session")
        .json(&json!({
            "title": "Test Session",
            "session_type": "interview"
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let body: serde_json::Value = response.json();
    assert!(!body["id"].as_str().unwrap().is_empty());
    assert_eq!(body["title"], "Test Session");
    assert_eq!(body["session_type"], "interview");
    assert_eq!(body["message_count"], 0);
}

#[tokio::test]
async fn test_create_session_with_defaults() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/session")
        .json(&json!({}))
        .await;

    response.assert_status(StatusCode::CREATED);

    let body: serde_json::Value = response.json();
    assert_eq!(body["session_type"], "general");
}

#[tokio::test]
async fn test_get_session() {
    let server = setup_test_server().await;

    // Create session
    let create_response = server
        .post("/api/session")
        .json(&json!({
            "title": "Get Test Session"
        }))
        .await;

    create_response.assert_status(StatusCode::CREATED);
    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // Get session
    let get_response = server.get(&format!("/api/session/{}", id)).await;

    get_response.assert_status(StatusCode::OK);
    let fetched: serde_json::Value = get_response.json();
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["title"], "Get Test Session");
}

#[tokio::test]
async fn test_get_session_not_found() {
    let server = setup_test_server().await;

    let response = server.get("/api/session/507f1f77bcf86cd799439011").await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_session_invalid_id() {
    let server = setup_test_server().await;

    let response = server.get("/api/session/invalid-id").await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_sessions() {
    let server = setup_test_server().await;

    let response = server.get("/api/sessions").await;

    response.assert_status(StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

#[tokio::test]
async fn test_delete_session() {
    let server = setup_test_server().await;

    // Create session
    let create_response = server
        .post("/api/session")
        .json(&json!({
            "title": "To Be Deleted"
        }))
        .await;

    create_response.assert_status(StatusCode::CREATED);
    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // Delete
    let delete_response = server.delete(&format!("/api/session/{}", id)).await;
    delete_response.assert_status(StatusCode::OK);

    // Verify deleted
    let get_response = server.get(&format!("/api/session/{}", id)).await;
    get_response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_add_message_to_session() {
    let server = setup_test_server().await;

    // Create session
    let create_response = server
        .post("/api/session")
        .json(&json!({}))
        .await;

    create_response.assert_status(StatusCode::CREATED);
    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // Add message
    let message_response = server
        .post(&format!("/api/session/{}/message", id))
        .json(&json!({
            "role": "user",
            "content": "Hello, this is a test message"
        }))
        .await;

    message_response.assert_status(StatusCode::OK);

    let message: serde_json::Value = message_response.json();
    assert_eq!(message["role"], "user");
    assert_eq!(message["content"], "Hello, this is a test message");

    // Verify session has message
    let get_response = server.get(&format!("/api/session/{}", id)).await;
    let session: serde_json::Value = get_response.json();
    assert_eq!(session["message_count"], 1);
}

#[tokio::test]
async fn test_add_message_empty_content_fails() {
    let server = setup_test_server().await;

    // Create session
    let create_response = server
        .post("/api/session")
        .json(&json!({}))
        .await;

    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // Add empty message
    let message_response = server
        .post(&format!("/api/session/{}/message", id))
        .json(&json!({
            "role": "user",
            "content": ""
        }))
        .await;

    message_response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_chat_with_session() {
    let server = setup_test_server().await;

    // Create session
    let create_response = server
        .post("/api/session")
        .json(&json!({
            "title": "Chat Test",
            "session_type": "interview"
        }))
        .await;

    create_response.assert_status(StatusCode::CREATED);
    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // Chat
    let chat_response = server
        .post(&format!("/api/session/{}/chat", id))
        .json(&json!({
            "message": "Hello, can you help me prepare for an interview?"
        }))
        .await;

    chat_response.assert_status(StatusCode::OK);

    let chat: serde_json::Value = chat_response.json();
    assert_eq!(chat["session_id"], id);
    assert_eq!(chat["message"]["role"], "user");
    assert_eq!(chat["response"]["role"], "assistant");
    assert!(!chat["response"]["content"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_chat_empty_message_fails() {
    let server = setup_test_server().await;

    // Create session
    let create_response = server
        .post("/api/session")
        .json(&json!({}))
        .await;

    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // Empty chat
    let chat_response = server
        .post(&format!("/api/session/{}/chat", id))
        .json(&json!({
            "message": ""
        }))
        .await;

    chat_response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_chat_session_not_found() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/session/507f1f77bcf86cd799439011/chat")
        .json(&json!({
            "message": "Hello"
        }))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_multi_turn_conversation() {
    let server = setup_test_server().await;

    // Create session
    let create_response = server
        .post("/api/session")
        .json(&json!({
            "title": "Multi-turn Test"
        }))
        .await;

    let created: serde_json::Value = create_response.json();
    let id = created["id"].as_str().unwrap();

    // First message
    let chat1 = server
        .post(&format!("/api/session/{}/chat", id))
        .json(&json!({
            "message": "My name is Alice"
        }))
        .await;

    chat1.assert_status(StatusCode::OK);

    // Second message - should remember context
    let chat2 = server
        .post(&format!("/api/session/{}/chat", id))
        .json(&json!({
            "message": "What is my name?"
        }))
        .await;

    chat2.assert_status(StatusCode::OK);

    // Verify session has 4 messages (2 user + 2 assistant)
    let session = server.get(&format!("/api/session/{}", id)).await;
    let session_data: serde_json::Value = session.json();
    assert_eq!(session_data["message_count"], 4);
}
