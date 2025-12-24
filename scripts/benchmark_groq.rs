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

async fn test_groq_model(client: &Client, api_key: &str, model: &str) -> Result<(String, u128, String), String> {
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
        .post("https://api.groq.com/openai/v1/chat/completions")
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

    let api_key = std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
    let client = Client::new();

    // Groq models (all free with rate limits)
    let models = vec![
        "llama-3.3-70b-versatile",
        "llama-3.1-8b-instant",
        "llama3-8b-8192",
        "llama3-70b-8192",
        "mixtral-8x7b-32768",
        "gemma2-9b-it",
    ];

    println!("\n🚀 Benchmarking GROQ Models (Known for Speed)\n");
    println!("Prompt: \"Write a function in Python to check if a number is prime\"\n");
    println!("{:-<80}", "");

    let mut results: Vec<(String, u128, String)> = Vec::new();

    for model in &models {
        print!("Testing {}... ", model);
        match test_groq_model(&client, &api_key, model).await {
            Ok((name, time, response)) => {
                println!("✅ {}ms", time);
                results.push((name, time, response));
            }
            Err(e) => {
                println!("❌ {}", e);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }

    results.sort_by(|a, b| a.1.cmp(&b.1));

    println!("\n{:-<80}", "");
    println!("\n📊 GROQ Results (sorted by speed):\n");

    for (i, (model, time, response)) in results.iter().enumerate() {
        println!("{}. {} - {}ms", i + 1, model, time);
        println!("   Response preview: {}...\n", &response.chars().take(100).collect::<String>().replace('\n', " "));
    }

    if let Some((fastest, time, _)) = results.first() {
        println!("\n🏆 Fastest GROQ model: {} ({}ms)", fastest, time);
    }
}
