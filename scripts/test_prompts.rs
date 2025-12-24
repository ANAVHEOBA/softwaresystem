use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize)]
struct SuggestRequest {
    context: String,
    suggestion_type: String,
}

#[derive(Debug, Deserialize)]
struct AiResponse {
    content: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = Client::new();

    println!("\n🧪 Testing Improved Prompts\n");

    // Test 1: LeetCode problem
    let leetcode_problem = r#"Given an array of integers nums and an integer target, return indices of the two numbers such that they add up to target. You may assume that each input would have exactly one solution."#;

    println!("📝 Test 1: LeetCode Two Sum");
    println!("Problem: {}\n", leetcode_problem);

    let request = SuggestRequest {
        context: leetcode_problem.to_string(),
        suggestion_type: "leetcode".to_string(),
    };

    let start = Instant::now();

    let response = client
        .post("http://127.0.0.1:8080/api/ai/suggest")
        .json(&request)
        .send()
        .await;

    let elapsed = start.elapsed().as_millis();

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let ai_response: AiResponse = resp.json().await.unwrap();
                println!("⏱️  Response time: {}ms\n", elapsed);
                println!("📝 Response:\n{}\n", ai_response.content);
            } else {
                println!("❌ Error: {}", resp.text().await.unwrap_or_default());
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}. Is the server running?", e);
            println!("\nTesting directly with Groq API instead...\n");
            test_groq_direct().await;
        }
    }
}

async fn test_groq_direct() {
    let api_key = std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
    let client = Client::new();

    let problem = r#"Given an array of integers nums and an integer target, return indices of the two numbers such that they add up to target."#;

    let system_prompt = r#"You are an expert competitive programmer. Give CONCISE answers.

FORMAT:
```python
[code]
```
Time: O(?) | Space: O(?) | Pattern: [name]

NO explanations unless asked. Code only."#;

    #[derive(Serialize)]
    struct ChatRequest {
        model: String,
        messages: Vec<ChatMessage>,
        max_tokens: u32,
        temperature: f32,
    }

    #[derive(Serialize)]
    struct ChatMessage {
        role: String,
        content: String,
    }

    #[derive(Deserialize)]
    struct ChatResponse {
        choices: Vec<ChatChoice>,
    }

    #[derive(Deserialize)]
    struct ChatChoice {
        message: ChatMessageResponse,
    }

    #[derive(Deserialize)]
    struct ChatMessageResponse {
        content: String,
    }

    let request = ChatRequest {
        model: "llama-3.1-8b-instant".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Solve this:\n\n{}", problem),
            },
        ],
        max_tokens: 800,
        temperature: 0.3,
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

    if response.status().is_success() {
        let chat_response: ChatResponse = response.json().await.unwrap();
        let content = chat_response.choices.first().map(|c| c.message.content.clone()).unwrap_or_default();

        println!("⏱️  Response time: {}ms\n", elapsed);
        println!("📝 Response:\n{}\n", content);
    } else {
        println!("❌ Error: {}", response.text().await.unwrap_or_default());
    }
}
