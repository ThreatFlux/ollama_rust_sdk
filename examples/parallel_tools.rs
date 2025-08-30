//! Example demonstrating parallel tool calling
//!
//! This example shows how to handle multiple tool calls in parallel,
//! which is useful when the model needs to gather information from
//! multiple sources simultaneously.

use ollama_rust_sdk::models::common::Tool;
use ollama_rust_sdk::{ChatMessage, OllamaClient};
use serde_json::json;
use std::time::Instant;

/// Simulated async API call with delay
async fn fetch_data(source: &str, query: &str) -> serde_json::Value {
    // Simulate network delay
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    match source {
        "database" => {
            json!({
                "source": "database",
                "query": query,
                "results": [
                    {"id": 1, "name": "Item 1", "value": 100},
                    {"id": 2, "name": "Item 2", "value": 200},
                ],
                "count": 2,
                "execution_time_ms": 123
            })
        }
        "api" => {
            json!({
                "source": "external_api",
                "endpoint": query,
                "data": {
                    "status": "success",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "values": [42, 84, 126]
                },
                "latency_ms": 245
            })
        }
        "cache" => {
            json!({
                "source": "cache",
                "key": query,
                "value": "cached_data_123",
                "ttl_seconds": 3600,
                "hit": true
            })
        }
        _ => json!({"error": "Unknown source"}),
    }
}

/// Search across multiple data sources
async fn search_all(query: &str) -> serde_json::Value {
    let start = Instant::now();

    // Launch parallel searches
    let (db_result, api_result, cache_result) = tokio::join!(
        fetch_data("database", query),
        fetch_data("api", query),
        fetch_data("cache", query)
    );

    let elapsed = start.elapsed();

    json!({
        "query": query,
        "results": {
            "database": db_result,
            "api": api_result,
            "cache": cache_result
        },
        "total_time_ms": elapsed.as_millis(),
        "parallel_execution": true
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ollama Parallel Tool Calling Example\n");

    let client = OllamaClient::new("http://localhost:11434")?;

    // Check server health
    if !client.health().await? {
        eprintln!("❌ Ollama server is not available");
        return Ok(());
    }

    // Define tools that can be called in parallel
    let search_tool = Tool::function(
        "search_all".to_string(),
        "Search across multiple data sources in parallel".to_string(),
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                }
            },
            "required": ["query"]
        }),
    );

    let fetch_tool = Tool::function(
        "fetch_data".to_string(),
        "Fetch data from a specific source".to_string(),
        json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "enum": ["database", "api", "cache"],
                    "description": "The data source"
                },
                "query": {
                    "type": "string",
                    "description": "The query or key"
                }
            },
            "required": ["source", "query"]
        }),
    );

    let tools = vec![search_tool, fetch_tool];

    // Get available model
    let models = client.list_models().await?;
    let model = models
        .models
        .first()
        .map(|m| m.name.as_str())
        .unwrap_or("llama3:latest");

    println!("Using model: {}\n", model);

    // Example 1: Single parallel search
    println!("{}", "=".repeat(60));
    println!("Example 1: Parallel Search Across Sources");
    println!("{}", "=".repeat(60));

    let messages = vec![
        ChatMessage::system("You are a helpful assistant with access to parallel search tools."),
        ChatMessage::user("Search for 'user_data' across all available sources"),
    ];

    match client
        .chat()
        .model(model)
        .messages(messages.clone())
        .tools(tools.clone())
        .send()
        .await
    {
        Ok(response) => {
            println!("Assistant: {}\n", response.message.content);

            if let Some(tool_calls) = response.message.tool_calls {
                for tool_call in tool_calls {
                    println!("🔧 Tool Call: {}", tool_call.function.name);
                    let args: serde_json::Value =
                        serde_json::from_str(&tool_call.function.arguments)?;

                    let result = match tool_call.function.name.as_str() {
                        "search_all" => {
                            let query = args["query"].as_str().unwrap_or("default");
                            search_all(query).await
                        }
                        "fetch_data" => {
                            let source = args["source"].as_str().unwrap_or("cache");
                            let query = args["query"].as_str().unwrap_or("default");
                            fetch_data(source, query).await
                        }
                        _ => json!({"error": "Unknown tool"}),
                    };

                    println!("📊 Result: {}\n", serde_json::to_string_pretty(&result)?);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            // Try without tools if tool calling is not supported
            println!("\nTrying without tools...");
            let response = client.chat().model(model).messages(messages).send().await?;
            println!("Response: {}", response.message.content);
        }
    }

    // Example 2: Benchmarking parallel vs sequential
    println!("\n{}", "=".repeat(60));
    println!("Example 2: Parallel vs Sequential Performance");
    println!("{}", "=".repeat(60));

    let queries = vec!["query1", "query2", "query3"];

    // Sequential execution
    println!("🐌 Sequential Execution:");
    let start = Instant::now();
    for query in &queries {
        let _ = fetch_data("database", query).await;
        println!("  - Fetched: {}", query);
    }
    let sequential_time = start.elapsed();
    println!("  Time: {:?}\n", sequential_time);

    // Parallel execution
    println!("🚀 Parallel Execution:");
    let start = Instant::now();
    let results = tokio::join!(
        fetch_data("database", queries[0]),
        fetch_data("database", queries[1]),
        fetch_data("database", queries[2])
    );

    println!("  - Fetched: {}", queries[0]);
    println!("  - Fetched: {}", queries[1]);
    println!("  - Fetched: {}", queries[2]);
    let parallel_time = start.elapsed();
    println!("  Time: {:?}\n", parallel_time);

    println!(
        "⚡ Speedup: {:.2}x faster",
        sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
    );

    // Show the results
    println!("\n📦 Parallel Results:");
    println!(
        "  Result 1: {} bytes",
        serde_json::to_string(&results.0)?.len()
    );
    println!(
        "  Result 2: {} bytes",
        serde_json::to_string(&results.1)?.len()
    );
    println!(
        "  Result 3: {} bytes",
        serde_json::to_string(&results.2)?.len()
    );

    Ok(())
}
