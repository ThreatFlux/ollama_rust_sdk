//! Quick test of tool calling with gpt-oss:20b

use ollama_rust_sdk::models::common::Tool;
use ollama_rust_sdk::{ChatMessage, OllamaClient};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Tool Calling with gpt-oss:20b\n");

    let client = OllamaClient::new("http://localhost:11434")?;

    // Define a simple calculator tool
    let calc_tool = Tool::function(
        "calculate".to_string(),
        "Perform a calculation".to_string(),
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression like '2+2' or '10*5'"
                }
            },
            "required": ["expression"]
        }),
    );

    let messages = vec![
        ChatMessage::system("You are a helpful assistant with calculator access."),
        ChatMessage::user("What is 123 * 456?"),
    ];

    println!("Sending request with tool...");
    let response = client
        .chat()
        .model("gpt-oss:20b")
        .messages(messages)
        .tools(vec![calc_tool])
        .send()
        .await?;

    println!("Response content: {}", response.message.content);

    if let Some(tool_calls) = response.message.tool_calls {
        println!("\n✅ Tool calls detected: {}", tool_calls.len());
        for (i, call) in tool_calls.iter().enumerate() {
            println!("\nTool Call #{}:", i + 1);
            println!("  Function: {}", call.function.name);
            println!("  Arguments: {}", call.function.arguments);

            if let Some(expr) = call.function.arguments.get("expression") {
                println!("  Parsed expression: {}", expr);
            }
        }
    } else {
        println!("\n❌ No tool calls in response");
    }

    Ok(())
}
