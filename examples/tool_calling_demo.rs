//! Comprehensive demonstration of tool calling with Ollama
//!
//! This example shows real tool calling working with gpt-oss:20b and qwen3:30b models

use ollama_rust_sdk::models::common::Tool;
use ollama_rust_sdk::{ChatMessage, OllamaClient};
use serde_json::json;
use std::collections::HashMap;

/// Simple weather data
fn get_weather(location: &str) -> String {
    let weather: HashMap<&str, &str> = [
        ("San Francisco", "Sunny, 72°F"),
        ("New York", "Cloudy, 65°F"),
        ("London", "Rainy, 59°F"),
        ("Tokyo", "Clear, 82°F"),
        ("Paris", "Partly cloudy, 68°F"),
    ]
    .iter()
    .cloned()
    .collect();

    weather
        .get(location)
        .map(|w| format!("Weather in {}: {}", location, w))
        .unwrap_or_else(|| format!("Weather data not available for {}", location))
}

/// Simple calculator
fn calculate(expr: &str) -> String {
    // Very basic calculator for demo
    if expr.contains('+') {
        let parts: Vec<&str> = expr.split('+').collect();
        if parts.len() == 2 {
            let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
            let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
            return format!("{} = {}", expr, a + b);
        }
    } else if expr.contains('*') {
        let parts: Vec<&str> = expr.split('*').collect();
        if parts.len() == 2 {
            let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
            let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
            return format!("{} = {}", expr, a * b);
        }
    }
    format!("Cannot calculate: {}", expr)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ollama Tool Calling Demo");
    println!("{}", "=".repeat(50));
    println!();

    let client = OllamaClient::new("http://localhost:11434")?;

    // Check for tool-supporting models
    let models = client.list_models().await?;
    let available_tool_models: Vec<String> = models
        .models
        .iter()
        .filter(|m| {
            m.name == "gpt-oss:20b"
                || m.name == "gpt-oss:120b"
                || m.name == "qwen3:30b"
                || m.name.contains("qwen3") && m.name.contains("30b")
        })
        .map(|m| m.name.clone())
        .collect();

    if available_tool_models.is_empty() {
        println!("⚠️  No tool-supporting models found!");
        println!("Please install one of these models:");
        println!("  ollama pull gpt-oss:20b");
        println!("  ollama pull gpt-oss:120b");
        println!("  ollama pull qwen3:30b");
        return Ok(());
    }

    let model = &available_tool_models[0];
    println!("✅ Using model: {}", model);
    println!();

    // Define tools
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
        "Perform simple calculations".to_string(),
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
    );

    let tools = vec![weather_tool, calc_tool];

    // Test scenarios
    let scenarios = vec![
        "What's the weather in San Francisco?",
        "Calculate 42 * 17 for me",
        "Tell me the weather in London and calculate 100 + 250",
    ];

    for query in scenarios {
        println!("📝 Query: {}", query);
        println!("{}", "-".repeat(40));

        let messages = vec![
            ChatMessage::system(
                "You are a helpful assistant with access to weather and calculator tools.",
            ),
            ChatMessage::user(query),
        ];

        let response = client
            .chat()
            .model(model)
            .messages(messages)
            .tools(tools.clone())
            .send()
            .await?;

        if !response.message.content.is_empty() {
            println!("💭 Thinking: {}", response.message.content);
        }

        if let Some(tool_calls) = response.message.tool_calls {
            println!("🔧 Tool Calls: {}", tool_calls.len());

            for call in tool_calls {
                println!("  → Function: {}", call.function.name);

                // Parse arguments and execute
                let args: serde_json::Value = serde_json::from_str(&call.function.arguments)?;

                let result = match call.function.name.as_str() {
                    "get_weather" => {
                        let location = args["location"].as_str().unwrap_or("Unknown");
                        get_weather(location)
                    }
                    "calculate" => {
                        let expr = args["expression"].as_str().unwrap_or("0");
                        calculate(expr)
                    }
                    _ => "Unknown function".to_string(),
                };

                println!("  ← Result: {}", result);
            }
        } else {
            println!("📢 Direct Response: {}", response.message.content);
        }

        println!();
    }

    println!("✨ Demo complete!");
    Ok(())
}
