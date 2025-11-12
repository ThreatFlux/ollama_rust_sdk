//! Example demonstrating streaming responses with tool calls
//!
//! This tests how tool calls work with streaming enabled

use ollama_rust_sdk::models::common::Tool;
use ollama_rust_sdk::{ChatMessage, OllamaClient};
use serde_json::json;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Streaming + Tool Calls Test\n");

    let client = OllamaClient::new("http://localhost:11434")?;

    // Define a simple weather tool
    let weather_tool = Tool::function(
        "get_weather".to_string(),
        "Get weather for a city".to_string(),
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name"
                }
            },
            "required": ["location"]
        }),
    );

    let calc_tool = Tool::function(
        "calculate".to_string(),
        "Perform calculation".to_string(),
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Math expression"
                }
            },
            "required": ["expression"]
        }),
    );

    let tools = vec![weather_tool, calc_tool];

    // Test 1: Simple streaming with tool call
    println!("Test 1: Streaming with single tool call");
    println!("{}", "-".repeat(40));

    let messages = vec![
        ChatMessage::system("You are a helpful assistant with weather and calculator tools."),
        ChatMessage::user("What's the weather in Paris?"),
    ];

    println!("Sending streaming request with tools...");
    let mut stream = client
        .chat()
        .model("gpt-oss:20b")
        .messages(messages)
        .tools(tools.clone())
        .stream()
        .await?;

    let mut full_response = String::new();
    let mut tool_calls = Vec::new();

    println!("Streaming response:");
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(response) => {
                // Print content as it streams
                if !response.message.content.is_empty() {
                    print!("{}", response.message.content);
                    full_response.push_str(&response.message.content);
                }

                // Collect tool calls
                if let Some(calls) = response.message.tool_calls {
                    tool_calls.extend(calls);
                }

                // Check if done
                if response.done {
                    println!("\n✅ Stream complete");
                    if !tool_calls.is_empty() {
                        println!("🔧 Tool calls received: {}", tool_calls.len());
                        for call in &tool_calls {
                            println!("  - Function: {}", call.function.name);
                            println!("    Args: {}", call.function.arguments);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Stream error: {}", e);
                break;
            }
        }
    }

    println!("\n");

    // Test 2: Streaming with multiple tool calls
    println!("Test 2: Streaming with multiple tool calls");
    println!("{}", "-".repeat(40));

    let messages = vec![
        ChatMessage::system("You are a helpful assistant. Use tools when needed."),
        ChatMessage::user("Tell me the weather in London and calculate 25 * 4"),
    ];

    println!("Sending request for multiple tool calls...");
    let mut stream = client
        .chat()
        .model("gpt-oss:20b")
        .messages(messages)
        .tools(tools.clone())
        .stream()
        .await?;

    let mut chunks_received = 0;
    let mut tool_calls = Vec::new();

    while let Some(chunk) = stream.next().await {
        if let Ok(response) = chunk {
            chunks_received += 1;

            if !response.message.content.is_empty() {
                print!("{}", response.message.content);
            }

            if let Some(calls) = response.message.tool_calls {
                tool_calls.extend(calls);
            }

            if response.done {
                break;
            }
        }
    }

    println!("\n📊 Streaming stats:");
    println!("  - Chunks received: {}", chunks_received);
    println!("  - Tool calls: {}", tool_calls.len());

    for (i, call) in tool_calls.iter().enumerate() {
        println!("\n  Tool Call #{}:", i + 1);
        println!("    Function: {}", call.function.name);

        let args = &call.function.arguments;
        if call.function.name == "get_weather" {
            let location = args
                .get("location")
                .and_then(|value| value.as_str())
                .unwrap_or("?");
            println!("    Location: {}", location);
        } else if call.function.name == "calculate" {
            let expression = args
                .get("expression")
                .and_then(|value| value.as_str())
                .unwrap_or("?");
            println!("    Expression: {}", expression);
        }
    }

    println!("\n");

    // Test 3: Streaming without tools (comparison)
    println!("Test 3: Regular streaming (no tools)");
    println!("{}", "-".repeat(40));

    let messages = vec![ChatMessage::user("Tell me a very short joke")];

    let mut stream = client
        .chat()
        .model("gpt-oss:20b")
        .messages(messages)
        .stream()
        .await?;

    print!("Response: ");
    while let Some(chunk) = stream.next().await {
        if let Ok(response) = chunk {
            print!("{}", response.message.content);
            if response.done {
                println!();
                break;
            }
        }
    }

    println!("\n✨ All streaming tests complete!");

    Ok(())
}
