//! Example demonstrating tool/function calling with Ollama
//!
//! This example shows how to:
//! 1. Define tools/functions for the model to call
//! 2. Handle tool calls from the model
//! 3. Send tool results back to the model
//! 4. Complete a full conversation with tool use

use ollama_rust_sdk::models::common::{Tool, ToolCall};
use ollama_rust_sdk::{ChatMessage, OllamaClient};
use serde_json::json;
use std::collections::HashMap;

/// Mock weather API function
fn get_weather(location: &str, unit: &str) -> serde_json::Value {
    // In a real application, this would call an actual weather API
    let weather_data: HashMap<&str, (&str, i32, &str)> = [
        ("San Francisco", ("sunny", 72, "fahrenheit")),
        ("New York", ("cloudy", 65, "fahrenheit")),
        ("London", ("rainy", 15, "celsius")),
        ("Tokyo", ("partly cloudy", 28, "celsius")),
        ("Paris", ("sunny", 22, "celsius")),
    ]
    .iter()
    .cloned()
    .collect();

    // Convert unit if needed
    let (condition, temp, temp_unit) = weather_data
        .get(location)
        .unwrap_or(&("unknown", 20, "celsius"));

    let temperature = if unit.to_lowercase() == "celsius" && *temp_unit == "fahrenheit" {
        (*temp - 32) * 5 / 9
    } else if unit.to_lowercase() == "fahrenheit" && *temp_unit == "celsius" {
        *temp * 9 / 5 + 32
    } else {
        *temp
    };

    json!({
        "location": location,
        "temperature": temperature,
        "unit": unit,
        "condition": condition,
        "forecast": "Stable for the next 3 days"
    })
}

/// Mock stock price API function
fn get_stock_price(symbol: &str) -> serde_json::Value {
    let stock_prices: HashMap<&str, (f64, f64, &str)> = [
        ("AAPL", (178.45, 2.3, "up")),
        ("GOOGL", (142.67, -1.2, "down")),
        ("MSFT", (380.12, 0.8, "up")),
        ("TSLA", (265.89, -3.5, "down")),
        ("AMZN", (155.34, 1.5, "up")),
    ]
    .iter()
    .cloned()
    .collect();

    let (price, change, direction) = stock_prices
        .get(symbol)
        .unwrap_or(&(100.0, 0.0, "unchanged"));

    json!({
        "symbol": symbol,
        "price": price,
        "change": change,
        "change_percent": format!("{:.2}%", change),
        "direction": direction,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })
}

/// Mock calculator function
fn calculate(expression: &str) -> serde_json::Value {
    // Simple calculator - in production, use a proper expression parser
    let result = match expression {
        expr if expr.contains('+') => {
            let parts: Vec<&str> = expr.split('+').collect();
            if parts.len() == 2 {
                let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
                a + b
            } else {
                0.0
            }
        }
        expr if expr.contains('-') => {
            let parts: Vec<&str> = expr.split('-').collect();
            if parts.len() == 2 {
                let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
                a - b
            } else {
                0.0
            }
        }
        expr if expr.contains('*') => {
            let parts: Vec<&str> = expr.split('*').collect();
            if parts.len() == 2 {
                let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
                a * b
            } else {
                0.0
            }
        }
        expr if expr.contains('/') => {
            let parts: Vec<&str> = expr.split('/').collect();
            if parts.len() == 2 {
                let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let b: f64 = parts[1].trim().parse().unwrap_or(1.0);
                if b != 0.0 {
                    a / b
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }
        _ => 0.0,
    };

    json!({
        "expression": expression,
        "result": result
    })
}

/// Process a tool call and return the result
fn process_tool_call(tool_call: &ToolCall) -> String {
    println!("  🔧 Processing tool call: {}", tool_call.function.name);
    println!("     Arguments: {}", tool_call.function.arguments);

    let args = &tool_call.function.arguments;
    let result = match tool_call.function.name.as_str() {
        "get_weather" => {
            let location = args
                .get("location")
                .and_then(|value| value.as_str())
                .unwrap_or("Unknown");
            let unit = args
                .get("unit")
                .and_then(|value| value.as_str())
                .unwrap_or("celsius");
            get_weather(location, unit)
        }
        "get_stock_price" => {
            let symbol = args
                .get("symbol")
                .and_then(|value| value.as_str())
                .unwrap_or("UNKNOWN");
            get_stock_price(symbol)
        }
        "calculate" => {
            let expression = args
                .get("expression")
                .and_then(|value| value.as_str())
                .unwrap_or("0");
            calculate(expression)
        }
        _ => json!({"error": "Unknown function"}),
    };

    result.to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Ollama Tool Calling Example\n");

    // Create the client
    let client = OllamaClient::new("http://localhost:11434")?;

    // Check if server is healthy
    if !client.health().await? {
        eprintln!("❌ Ollama server is not available");
        return Ok(());
    }

    // Define available tools
    let weather_tool = Tool::function(
        "get_weather".to_string(),
        "Get the current weather for a location".to_string(),
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "The temperature unit"
                }
            },
            "required": ["location"]
        }),
    );

    let stock_tool = Tool::function(
        "get_stock_price".to_string(),
        "Get the current stock price for a symbol".to_string(),
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "The stock symbol, e.g. AAPL"
                }
            },
            "required": ["symbol"]
        }),
    );

    let calculator_tool = Tool::function(
        "calculate".to_string(),
        "Perform a mathematical calculation".to_string(),
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The mathematical expression to evaluate, e.g. '23 + 45'"
                }
            },
            "required": ["expression"]
        }),
    );

    let tools = vec![weather_tool, stock_tool, calculator_tool];

    // Test scenarios
    let test_queries = vec![
        "What's the weather like in San Francisco and New York?",
        "What's the current price of AAPL stock?",
        "Calculate 123 * 456 for me",
        "Compare the weather in London and Tokyo, and also tell me the MSFT stock price",
        "What's 1000 / 25 and what's the weather in Paris in fahrenheit?",
    ];

    // Choose the best available model for tool calling
    // Note: qwen3:30b, gpt-oss:20b, and gpt-oss:120b are known to support tool calling
    let model = match client.list_models().await {
        Ok(models) => {
            // Prioritize models known to support tool calling
            if models.models.iter().any(|m| m.name == "gpt-oss:120b") {
                "gpt-oss:120b".to_string()
            } else if models.models.iter().any(|m| m.name == "gpt-oss:20b") {
                "gpt-oss:20b".to_string()
            } else if models.models.iter().any(|m| m.name == "qwen3:30b") {
                "qwen3:30b".to_string()
            } else if models
                .models
                .iter()
                .any(|m| m.name.contains("qwen3") && m.name.contains("30b"))
            {
                models
                    .models
                    .iter()
                    .find(|m| m.name.contains("qwen3") && m.name.contains("30b"))
                    .unwrap()
                    .name
                    .clone()
            } else if models
                .models
                .iter()
                .any(|m| m.name.contains("llama3.1") || m.name.contains("llama3:70b"))
            {
                // Larger Llama models may support tools
                models
                    .models
                    .iter()
                    .find(|m| m.name.contains("llama3.1") || m.name.contains("llama3:70b"))
                    .unwrap()
                    .name
                    .clone()
            } else if !models.models.is_empty() {
                eprintln!("⚠️  No known tool-supporting models found.");
                eprintln!("   Recommended models: gpt-oss:20b, gpt-oss:120b, qwen3:30b");
                eprintln!("   Using first available model, but tool calling may not work.");
                models.models[0].name.clone()
            } else {
                eprintln!("❌ No models available. Please pull a model first.");
                eprintln!("   Recommended: ollama pull gpt-oss:20b");
                return Ok(());
            }
        }
        Err(_) => {
            eprintln!("⚠️  Could not list models. Defaulting to gpt-oss:20b");
            "gpt-oss:20b".to_string()
        }
    };

    println!("Using model: {}\n", model);

    for query in test_queries {
        println!("{}", "=".repeat(60));
        println!("📝 User Query: {}", query);
        println!("{}", "=".repeat(60));

        let mut messages = vec![
            ChatMessage::system(
                "You are a helpful assistant with access to tools. Use them when needed to answer questions accurately."
            ),
            ChatMessage::user(query),
        ];

        // First request with tools
        let response = client
            .chat()
            .model(&model)
            .messages(messages.clone())
            .tools(tools.clone())
            .send()
            .await;

        match response {
            Ok(response) => {
                println!("\n🤖 Assistant: {}", response.message.content);

                // Check if the model made any tool calls
                if let Some(tool_calls) = response.message.tool_calls {
                    println!("\n📞 Tool Calls Requested: {}", tool_calls.len());

                    // Add assistant's message with tool calls to history
                    let mut assistant_msg = ChatMessage::assistant(&response.message.content);
                    assistant_msg.tool_calls = Some(tool_calls.clone());
                    messages.push(assistant_msg);

                    // Process each tool call
                    for tool_call in &tool_calls {
                        let result = process_tool_call(tool_call);
                        println!("     Result: {}", result);

                        // Add tool response to messages
                        let tool_id = tool_call
                            .id
                            .clone()
                            .unwrap_or_else(|| format!("call_{}", uuid::Uuid::new_v4()));
                        messages.push(ChatMessage::tool(result, tool_id));
                    }

                    // Send the tool results back to the model
                    println!("\n🔄 Sending tool results back to model...");

                    match client
                        .chat()
                        .model(&model)
                        .messages(messages)
                        .tools(tools.clone())
                        .send()
                        .await
                    {
                        Ok(final_response) => {
                            println!("\n✅ Final Response: {}", final_response.message.content);
                        }
                        Err(e) => {
                            eprintln!("❌ Error in follow-up: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);

                // If tool calling is not supported, try without tools
                println!("\n⚠️  Tool calling might not be supported by this model.");
                println!("    Trying without tools...\n");

                let response = client
                    .chat()
                    .model(&model)
                    .messages(vec![
                        ChatMessage::system("You are a helpful assistant."),
                        ChatMessage::user(query),
                    ])
                    .send()
                    .await;

                if let Ok(response) = response {
                    println!("🤖 Response (without tools): {}", response.message.content);
                }
            }
        }

        println!("\n");
    }

    // Interactive tool calling session
    println!("{}", "=".repeat(60));
    println!("🎮 Interactive Tool Calling Session");
    println!("{}", "=".repeat(60));
    println!("Type your questions (or 'quit' to exit):");
    println!("Available tools: weather, stock prices, calculator\n");

    let mut conversation = vec![ChatMessage::system(
        "You are a helpful assistant with access to tools. Use them when needed.",
    )];

    loop {
        print!("> ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("👋 Goodbye!");
            break;
        }

        conversation.push(ChatMessage::user(input));

        let response = client
            .chat()
            .model(&model)
            .messages(conversation.clone())
            .tools(tools.clone())
            .send()
            .await;

        match response {
            Ok(response) => {
                println!("\n🤖 {}", response.message.content);

                let mut assistant_msg = ChatMessage::assistant(&response.message.content);

                if let Some(tool_calls) = response.message.tool_calls {
                    println!("\n🔧 Making {} tool call(s)...", tool_calls.len());
                    assistant_msg.tool_calls = Some(tool_calls.clone());
                    conversation.push(assistant_msg);

                    for tool_call in &tool_calls {
                        let result = process_tool_call(tool_call);
                        let tool_id = tool_call
                            .id
                            .clone()
                            .unwrap_or_else(|| format!("call_{}", uuid::Uuid::new_v4()));
                        conversation.push(ChatMessage::tool(result.clone(), tool_id));
                    }

                    // Get final response with tool results
                    let followup = client
                        .chat()
                        .model(&model)
                        .messages(conversation.clone())
                        .tools(tools.clone())
                        .send()
                        .await;

                    if let Ok(final_response) = followup {
                        println!("📊 {}", final_response.message.content);
                        conversation.push(ChatMessage::assistant(&final_response.message.content));
                    }
                } else {
                    conversation.push(assistant_msg);
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
            }
        }
        println!();
    }

    Ok(())
}
