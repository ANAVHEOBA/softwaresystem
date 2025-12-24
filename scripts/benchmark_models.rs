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

async fn test_model(client: &Client, api_key: &str, model: &str) -> Result<(String, u128, String), String> {
    let prompt = "Write a function in Python to check if a number is prime. Keep it short.";

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        max_tokens: 200,
    };

    let start = Instant::now();

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let elapsed = start.elapsed().as_millis();

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(format!("API error: {}", error));
    }

    let chat_response: ChatResponse = response.json().await.map_err(|e| e.to_string())?;

    let content = chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    Ok((model.to_string(), elapsed, content))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set");
    let client = Client::new();

    let models = vec![
        "xiaomi/mimo-v2-flash:free",
        "nvidia/nemotron-3-nano-30b-a3b:free",
        "mistralai/devstral-2512:free",
        "nex-agi/deepseek-v3.1-nex-n1:free",
        "kwaipilot/kat-coder-pro:free",
        "qwen/qwen3-30b-a3b:free",
        "qwen/qwen-2.5-coder-32b:free",
        "deepseek/deepseek-chat-v3-0324:free",
        "google/gemma-3-27b-it:free",
        "meta-llama/llama-4-maverick:free",
    ];

    println!("\n🚀 Benchmarking LLM Models for Speed\n");
    println!("Prompt: \"Write a function in Python to check if a number is prime\"\n");
    println!("{:-<80}", "");

    let mut results: Vec<(String, u128, String)> = Vec::new();

    for model in &models {
        print!("Testing {}... ", model);
        match test_model(&client, &api_key, model).await {
            Ok((name, time, response)) => {
                println!("✅ {}ms", time);
                results.push((name, time, response));
            }
            Err(e) => {
                println!("❌ {}", e);
            }
        }
        // Small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Sort by time
    results.sort_by(|a, b| a.1.cmp(&b.1));

    println!("\n{:-<80}", "");
    println!("\n📊 Results (sorted by speed):\n");

    for (i, (model, time, response)) in results.iter().enumerate() {
        println!("{}. {} - {}ms", i + 1, model, time);
        println!("   Response preview: {}...\n", &response.chars().take(100).collect::<String>().replace('\n', " "));
    }

    if let Some((fastest, time, _)) = results.first() {
        println!("\n🏆 Fastest model: {} ({}ms)", fastest, time);
    }
}
