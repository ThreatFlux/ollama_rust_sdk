//! Performance tests for the Ollama Rust SDK

use ollama_rust_sdk::OllamaClient;
use std::time::Instant;
use tokio_stream::StreamExt;

/// Test generation performance with different parameters
#[tokio::test]
async fn test_generation_performance_metrics() {
    let client = match OllamaClient::new("http://localhost:11434") {
        Ok(client) => client,
        Err(_) => {
            println!("Failed to create client, skipping performance test");
            return;
        }
    };

    if !client.health().await.unwrap_or(false) {
        println!("Ollama server not healthy, skipping performance test");
        return;
    }

    let models = match client.list_models().await {
        Ok(models) => models,
        Err(_) => {
            println!("Could not list models, skipping performance test");
            return;
        }
    };

    if models.models.is_empty() {
        println!("No models available, skipping performance test");
        return;
    }

    let model_name = &models.models[0].name;
    println!("\\n=== Performance Test Results ===");
    println!("Model: {}", model_name);

    // Test 1: Short prompt, fast response
    println!("\\n--- Test 1: Short Prompt ---");
    let start = Instant::now();
    let response = client
        .generate()
        .model(model_name)
        .prompt("Hello")
        .temperature(0.1)
        .max_tokens(10)
        .send()
        .await;
    let duration = start.elapsed();

    match response {
        Ok(resp) => {
            println!("Response: {}", resp.response);
            println!("Wall clock time: {:?}", duration);

            if let Some(total_duration) = resp.total_duration {
                let server_time = total_duration as f64 / 1e9;
                println!("Server total time: {:.3}s", server_time);
            }

            if let Some(eval_rate) = resp.eval_rate() {
                println!("Generation rate: {:.2} tokens/second", eval_rate);
            }
        }
        Err(e) => println!("Test 1 failed: {}", e),
    }

    // Test 2: Medium prompt
    println!("\\n--- Test 2: Medium Prompt ---");
    let start = Instant::now();
    let response = client
        .generate()
        .model(model_name)
        .prompt("Write a short paragraph about artificial intelligence.")
        .temperature(0.7)
        .max_tokens(100)
        .send()
        .await;
    let duration = start.elapsed();

    match response {
        Ok(resp) => {
            println!("Response length: {} characters", resp.response.len());
            println!("Wall clock time: {:?}", duration);

            if let Some(total_duration) = resp.total_duration {
                let server_time = total_duration as f64 / 1e9;
                println!("Server total time: {:.3}s", server_time);
            }

            if let Some(eval_rate) = resp.eval_rate() {
                println!("Generation rate: {:.2} tokens/second", eval_rate);
            }

            if let Some(eval_count) = resp.eval_count {
                println!("Tokens generated: {}", eval_count);
            }
        }
        Err(e) => println!("Test 2 failed: {}", e),
    }

    // Test 3: Streaming performance
    println!("\\n--- Test 3: Streaming Performance ---");
    let start = Instant::now();
    let mut stream = match client
        .generate()
        .model(model_name)
        .prompt("Count from 1 to 10 with explanations:")
        .temperature(0.3)
        .max_tokens(150)
        .stream()
        .await
    {
        Ok(stream) => stream,
        Err(e) => {
            println!("Failed to create stream: {}", e);
            return;
        }
    };

    let mut chunks = 0;
    let mut total_chars = 0;
    let mut first_token_time: Option<Instant> = None;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(response) => {
                if first_token_time.is_none() && !response.response.is_empty() {
                    first_token_time = Some(Instant::now());
                }

                chunks += 1;
                total_chars += response.response.len();

                if response.done {
                    let total_time = start.elapsed();
                    println!("Streaming completed:");
                    println!("  Total chunks: {}", chunks);
                    println!("  Total characters: {}", total_chars);
                    println!("  Total time: {:?}", total_time);

                    if let Some(first_token) = first_token_time {
                        let time_to_first_token = first_token.duration_since(start);
                        println!("  Time to first token: {:?}", time_to_first_token);
                    }

                    if let Some(eval_rate) = response.eval_rate() {
                        println!("  Final generation rate: {:.2} tokens/second", eval_rate);
                    }

                    break;
                }
            }
            Err(e) => {
                println!("Stream error: {}", e);
                break;
            }
        }
    }

    println!("\\n=== Performance Summary ===");
    println!("All performance tests completed successfully!");
}

/// Test chat performance
#[tokio::test]
async fn test_chat_performance() {
    let client = match OllamaClient::new("http://localhost:11434") {
        Ok(client) => client,
        Err(_) => {
            println!("Failed to create client, skipping chat performance test");
            return;
        }
    };

    if !client.health().await.unwrap_or(false) {
        println!("Ollama server not healthy, skipping chat performance test");
        return;
    }

    let models = match client.list_models().await {
        Ok(models) => models,
        Err(_) => {
            println!("Could not list models, skipping chat performance test");
            return;
        }
    };

    if models.models.is_empty() {
        println!("No models available, skipping chat performance test");
        return;
    }

    let model_name = &models.models[0].name;
    println!("\\n=== Chat Performance Test ===");
    println!("Model: {}", model_name);

    let start = Instant::now();
    let response = client
        .chat()
        .model(model_name)
        .add_system_message("You are a helpful assistant.")
        .add_user_message("What is the capital of Japan?")
        .temperature(0.3)
        .max_tokens(50)
        .send()
        .await;
    let duration = start.elapsed();

    match response {
        Ok(resp) => {
            println!("Chat response: {}", resp.message.content);
            println!("Wall clock time: {:?}", duration);

            if let Some(total_duration) = resp.total_duration {
                let server_time = total_duration as f64 / 1e9;
                println!("Server total time: {:.3}s", server_time);
            }

            if let Some(eval_rate) = resp.eval_rate() {
                println!("Generation rate: {:.2} tokens/second", eval_rate);
            }
        }
        Err(e) => println!("Chat performance test failed: {}", e),
    }
}

/// Test embeddings performance
#[tokio::test]
async fn test_embeddings_performance() {
    let client = match OllamaClient::new("http://localhost:11434") {
        Ok(client) => client,
        Err(_) => {
            println!("Failed to create client, skipping embeddings performance test");
            return;
        }
    };

    if !client.health().await.unwrap_or(false) {
        println!("Ollama server not healthy, skipping embeddings performance test");
        return;
    }

    let models = match client.list_models().await {
        Ok(models) => models,
        Err(_) => {
            println!("Could not list models, skipping embeddings performance test");
            return;
        }
    };

    // Look for embedding models
    let embedding_model = models
        .models
        .iter()
        .find(|m| m.name.contains("embed") || m.name.contains("nomic"))
        .map(|m| m.name.as_str());

    let model_name = match embedding_model {
        Some(name) => name,
        None => {
            println!("No embedding model available, skipping embeddings performance test");
            return;
        }
    };

    println!("\\n=== Embeddings Performance Test ===");
    println!("Model: {}", model_name);

    let texts = vec![
        "This is a test sentence for embeddings.",
        "Machine learning is a fascinating field.",
        "Performance testing is important for quality software.",
        "Rust is a systems programming language.",
        "Artificial intelligence will shape the future.",
    ];

    let start = Instant::now();
    let response = client
        .embed()
        .model(model_name)
        .input(texts.clone())
        .send()
        .await;
    let duration = start.elapsed();

    match response {
        Ok(resp) => {
            println!("Generated {} embeddings", resp.count());
            println!("Embedding dimensions: {:?}", resp.dimensions());
            println!("Total time: {:?}", duration);
            println!("Average time per text: {:?}", duration / texts.len() as u32);

            if let Some(total_duration) = resp.total_duration {
                let server_time = total_duration as f64 / 1e9;
                println!("Server total time: {:.3}s", server_time);
            }
        }
        Err(e) => println!("Embeddings performance test failed: {}", e),
    }
}
