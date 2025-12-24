use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
    let client = Client::new();

    let prompt = "Write a Python function to solve two sum leetcode problem. Be concise.";

    println!("\n🧪 Testing Groq Speed for LeetCode-style Questions\n");
    println!("Prompt: \"{}\"\n", prompt);

    let request = ChatRequest {
        model: "llama-3.1-8b-instant".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a coding assistant. Provide concise, correct solutions.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        max_tokens: 300,
    };

    let start = Instant::now();

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .expect("Request failed");

    let elapsed = start.elapsed().as_millis();

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        println!("❌ Error: {}", error);
        return;
    }

    let chat_response: ChatResponse = response.json().await.expect("Parse failed");
    let content = chat_response.choices.first().map(|c| c.message.content.clone()).unwrap_or_default();

    println!("⏱️  Response time: {}ms\n", elapsed);
    println!("📝 Response:\n{}\n", content);
    println!("{:-<60}", "");

    if elapsed < 1000 {
        println!("✅ Fast enough for real-time assistance!");
    } else {
        println!("⚠️  Response might feel slow for real-time use.");
    }
}
