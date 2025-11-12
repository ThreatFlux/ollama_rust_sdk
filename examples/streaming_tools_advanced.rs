//! Advanced streaming with tool calls - complete conversation flow
//!
//! This example shows:
//! 1. Streaming request with tools
//! 2. Processing tool calls
//! 3. Sending tool results back
//! 4. Getting final streaming response

use ollama_rust_sdk::models::common::Tool;
use ollama_rust_sdk::{ChatMessage, OllamaClient};
use serde_json::json;
use std::time::Instant;
use tokio_stream::StreamExt;

/// Mock weather function
fn get_weather(location: &str) -> String {
    match location.to_lowercase().as_str() {
        "london" => "London: 15°C, Rainy with occasional sunshine".to_string(),
        "paris" => "Paris: 22°C, Clear skies".to_string(),
        "tokyo" => "Tokyo: 28°C, Humid with clouds".to_string(),
        "new york" => "New York: 18°C, Partly cloudy".to_string(),
        _ => format!("{}: Weather data unavailable", location),
    }
}

/// Mock calculator
fn calculate(expr: &str) -> String {
    // Simple parsing for demo
    if let Some(pos) = expr.find('*') {
        let (a, b) = expr.split_at(pos);
        let a: f64 = a.trim().parse().unwrap_or(0.0);
        let b: f64 = b[1..].trim().parse().unwrap_or(0.0);
        return format!("{} = {}", expr, a * b);
    } else if let Some(pos) = expr.find('+') {
        let (a, b) = expr.split_at(pos);
        let a: f64 = a.trim().parse().unwrap_or(0.0);
        let b: f64 = b[1..].trim().parse().unwrap_or(0.0);
        return format!("{} = {}", expr, a + b);
    }
    format!("Cannot calculate: {}", expr)
}

async fn stream_and_collect(
    mut stream: impl StreamExt<
            Item = Result<
                ollama_rust_sdk::models::chat::ChatResponse,
                ollama_rust_sdk::error::OllamaError,
            >,
        > + Unpin,
) -> Result<(String, Vec<ollama_rust_sdk::models::common::ToolCall>), Box<dyn std::error::Error>> {
    let mut content = String::new();
    let mut tool_calls = Vec::new();
    let mut chunk_count = 0;
    let start = Instant::now();

    while let Some(chunk) = stream.next().await {
        let response = chunk?;
        chunk_count += 1;

        if !response.message.content.is_empty() {
            print!("{}", response.message.content);
            content.push_str(&response.message.content);
        }

        if let Some(calls) = response.message.tool_calls {
            tool_calls.extend(calls);
        }

        if response.done {
            let elapsed = start.elapsed();
            println!(
                "\n  [Streamed {} chunks in {:.2}s]",
                chunk_count,
                elapsed.as_secs_f64()
            );
            break;
        }
    }

    Ok((content, tool_calls))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Streaming + Tool Calls Demo");
    println!("{}", "=".repeat(50));
    println!();

    let client = OllamaClient::new("http://localhost:11434")?;

    // Define tools
    let tools = vec![
        Tool::function(
            "get_weather".to_string(),
            "Get current weather for a city".to_string(),
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
        ),
        Tool::function(
            "calculate".to_string(),
            "Perform mathematical calculations".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Math expression like '2+2' or '10*5'"
                    }
                },
                "required": ["expression"]
            }),
        ),
    ];

    // Test scenario: Multi-step conversation with streaming
    println!("📝 User: What's the weather in London and Paris, and calculate 42 * 17?");
    println!("{}", "-".repeat(50));

    let mut messages = vec![
        ChatMessage::system("You are a helpful assistant with weather and calculation tools. Always use tools when asked about weather or math."),
        ChatMessage::user("What's the weather in London and Paris, and calculate 42 * 17?"),
    ];

    // Step 1: Initial streaming request
    println!("\n🤖 Assistant (streaming with tools):");
    let stream = client
        .chat()
        .model("gpt-oss:20b")
        .messages(messages.clone())
        .tools(tools.clone())
        .stream()
        .await?;

    let (content, tool_calls) = stream_and_collect(stream).await?;

    if !tool_calls.is_empty() {
        println!("\n\n🔧 Processing {} tool call(s):", tool_calls.len());

        // Add assistant message with tool calls
        let mut assistant_msg = ChatMessage::assistant(&content);
        assistant_msg.tool_calls = Some(tool_calls.clone());
        messages.push(assistant_msg);

        // Process each tool call
        for call in &tool_calls {
            let args = &call.function.arguments;
            let result = match call.function.name.as_str() {
                "get_weather" => {
                    let location = args
                        .get("location")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Unknown");
                    println!("  📍 Getting weather for: {}", location);
                    get_weather(location)
                }
                "calculate" => {
                    let expr = args
                        .get("expression")
                        .and_then(|value| value.as_str())
                        .unwrap_or("0");
                    println!("  🧮 Calculating: {}", expr);
                    calculate(expr)
                }
                _ => "Unknown function".to_string(),
            };

            println!("     → {}", result);

            // Add tool result to conversation
            let tool_id = call.id.clone().unwrap_or_else(|| "tool_call".to_string());
            messages.push(ChatMessage::tool(result, tool_id));
        }

        // Step 2: Stream final response with tool results
        println!("\n🤖 Final response (streaming):");
        let final_stream = client
            .chat()
            .model("gpt-oss:20b")
            .messages(messages)
            .tools(tools.clone())
            .stream()
            .await?;

        let (final_content, _) = stream_and_collect(final_stream).await?;

        if final_content.is_empty() {
            println!("  [No additional response needed]");
        }
    }

    println!("\n");

    // Test scenario 2: Real-time streaming interaction
    println!("📝 User: Just calculate 123 + 456 quickly");
    println!("{}", "-".repeat(50));

    let messages = vec![
        ChatMessage::system("You are a calculator assistant. Use the calculate tool for math."),
        ChatMessage::user("Just calculate 123 + 456 quickly"),
    ];

    println!("\n🤖 Assistant:");
    let stream = client
        .chat()
        .model("gpt-oss:20b")
        .messages(messages)
        .tools(tools)
        .stream()
        .await?;

    let start = Instant::now();
    let (_, tool_calls) = stream_and_collect(stream).await?;

    if !tool_calls.is_empty() {
        println!("\n🔧 Tool call executed:");
        for call in tool_calls {
            if call.function.name == "calculate" {
                let expr = call
                    .function
                    .arguments
                    .get("expression")
                    .and_then(|value| value.as_str())
                    .unwrap_or("0");
                let result = calculate(expr);
                println!("  {} ", result);
            }
        }
    }

    let total_time = start.elapsed();
    println!("\n⏱️  Total time: {:.2}s", total_time.as_secs_f64());

    println!("\n✨ Advanced streaming demo complete!");

    Ok(())
}
