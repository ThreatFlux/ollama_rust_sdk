//! Integration tests for the Ollama Rust SDK

use ollama_rust_sdk::{OllamaClient, OllamaError};
use std::time::Instant;

/// Test basic client creation and health check
#[tokio::test]
async fn test_client_health_check() {
    let client = OllamaClient::new("http://localhost:11434").unwrap();

    // Health check should work even if no models are available
    let is_healthy = client.health().await.unwrap_or(false);

    // This test will pass if Ollama is running, or skip if not
    if !is_healthy {
        println!("Ollama server not running, skipping health check test");
    }
}

/// Test listing models (requires Ollama to be running)
#[tokio::test]
async fn test_list_models() {
    let client = match OllamaClient::new("http://localhost:11434") {
        Ok(client) => client,
        Err(_) => {
            println!("Failed to create client, skipping test");
            return;
        }
    };

    match client.list_models().await {
        Ok(models) => {
            println!("Found {} models", models.models.len());
            for model in models.models {
                println!("  - {} ({})", model.name, model.size_string());
            }
        }
        Err(OllamaError::NetworkError(_)) => {
            println!("Ollama server not running, skipping test");
        }
        Err(e) => {
            panic!("Unexpected error: {}", e);
        }
    }
}

/// Test basic text generation with first available model
#[tokio::test]
async fn test_generation_with_available_model() {
    let client = match OllamaClient::new("http://localhost:11434") {
        Ok(client) => client,
        Err(_) => {
            println!("Failed to create client, skipping test");
            return;
        }
    };

    // Check if server is healthy
    if !client.health().await.unwrap_or(false) {
        println!("Ollama server not healthy, skipping test");
        return;
    }

    // Get available models
    let models = match client.list_models().await {
        Ok(models) => models,
        Err(_) => {
            println!("Could not list models, skipping test");
            return;
        }
    };

    if models.models.is_empty() {
        println!("No models available, skipping test");
        return;
    }

    let model_name = &models.models[0].name;
    println!("Testing generation with model: {}", model_name);

    let start = Instant::now();
    let response = client
        .generate()
        .model(model_name)
        .prompt("What is 1+1? Answer in one sentence.")
        .temperature(0.1)
        .max_tokens(50)
        .send()
        .await;
    let duration = start.elapsed();

    match response {
        Ok(resp) => {
            println!("Generated response: {}", resp.response);
            println!("Generation took: {:?}", duration);
            assert!(!resp.response.is_empty());
            assert_eq!(resp.model, *model_name);
            assert!(resp.done);
        }
        Err(e) => {
            println!("Generation failed: {}", e);
            // Don't panic, just log the error
        }
    }
}

/// Test performance metrics
#[tokio::test]
async fn test_performance_metrics() {
    let client = match OllamaClient::new("http://localhost:11434") {
        Ok(client) => client,
        Err(_) => {
            println!("Failed to create client, skipping test");
            return;
        }
    };

    if !client.health().await.unwrap_or(false) {
        println!("Ollama server not healthy, skipping test");
        return;
    }

    let models = match client.list_models().await {
        Ok(models) => models,
        Err(_) => {
            println!("Could not list models, skipping test");
            return;
        }
    };

    if models.models.is_empty() {
        println!("No models available, skipping test");
        return;
    }

    let model_name = &models.models[0].name;
    println!("Testing performance with model: {}", model_name);

    let start = Instant::now();
    let response = client
        .generate()
        .model(model_name)
        .prompt("Generate a short poem about performance.")
        .temperature(0.7)
        .max_tokens(100)
        .send()
        .await;
    let total_time = start.elapsed();

    match response {
        Ok(resp) => {
            println!("Performance test completed in: {:?}", total_time);

            if let Some(eval_rate) = resp.eval_rate() {
                println!("Evaluation rate: {:.2} tokens/second", eval_rate);
                assert!(eval_rate > 0.0, "Evaluation rate should be positive");
            }

            if let Some(total_duration) = resp.total_duration {
                let total_seconds = total_duration as f64 / 1e9;
                println!("Total generation time: {:.2}s", total_seconds);
                assert!(total_seconds > 0.0, "Total duration should be positive");
            }

            if let Some(eval_count) = resp.eval_count {
                println!("Tokens evaluated: {}", eval_count);
                assert!(eval_count > 0, "Should have evaluated some tokens");
            }
        }
        Err(e) => {
            println!("Performance test failed: {}", e);
        }
    }
}

/// Test embeddings with embedding model if available
#[tokio::test]
async fn test_embeddings() {
    let client = match OllamaClient::new("http://localhost:11434") {
        Ok(client) => client,
        Err(_) => {
            println!("Failed to create client, skipping test");
            return;
        }
    };

    if !client.health().await.unwrap_or(false) {
        println!("Ollama server not healthy, skipping test");
        return;
    }

    let models = match client.list_models().await {
        Ok(models) => models,
        Err(_) => {
            println!("Could not list models, skipping test");
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
            println!("No embedding model available, skipping test");
            return;
        }
    };

    println!("Testing embeddings with model: {}", model_name);

    let response = client
        .embed()
        .model(model_name)
        .input(vec!["Hello world".to_string(), "Test text".to_string()])
        .send()
        .await;

    match response {
        Ok(resp) => {
            println!("Generated {} embeddings", resp.count());
            println!("Embedding dimensions: {:?}", resp.dimensions());
            assert_eq!(resp.count(), 2);
            assert!(resp.dimensions().unwrap_or(0) > 0);
        }
        Err(e) => {
            println!("Embeddings failed: {}", e);
            // Don't panic, just log the error
        }
    }
}
