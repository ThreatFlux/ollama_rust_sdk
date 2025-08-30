//! Basic text generation example

use ollama_rust_sdk::{OllamaClient, OllamaError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Create a client
    let client = OllamaClient::new("http://localhost:11434")?;

    // Check if the server is healthy
    if !client.health().await? {
        eprintln!("Ollama server is not healthy or not running");
        std::process::exit(1);
    }

    println!("Connected to Ollama server successfully!");

    // List available models
    match client.list_models().await {
        Ok(models) => {
            println!("Available models:");
            for model in &models.models {
                println!("  - {} ({})", model.name, model.size_string());
            }

            if models.models.is_empty() {
                println!("No models available. Please pull a model first:");
                println!("  ollama pull qwen3:30b-a3b");
                return Ok(());
            }
        }
        Err(e) => {
            eprintln!("Failed to list models: {}", e);
            return Ok(());
        }
    }

    // Use the first available model, or default to qwen3:30b-a3b
    let models = client.list_models().await?;
    let model_name = models
        .models
        .first()
        .map(|m| m.name.as_str())
        .unwrap_or("qwen3:30b-a3b");

    println!("\nUsing model: {}", model_name);

    // Generate text
    println!("\n=== Basic Generation ===");
    match client
        .generate()
        .model(model_name)
        .prompt("Why is the sky blue? Explain in simple terms.")
        .temperature(0.7)
        .max_tokens(150)
        .send()
        .await
    {
        Ok(response) => {
            println!("Response: {}", response.response);

            // Show performance metrics if available
            if let Some(rate) = response.eval_rate() {
                println!("Generation speed: {:.2} tokens/second", rate);
            }
            if let Some(total_duration) = response.total_duration {
                println!("Total time: {:.2}s", total_duration as f64 / 1e9);
            }
        }
        Err(OllamaError::ModelNotFound(model)) => {
            eprintln!("Model '{}' not found. Available models:", model);
            let models = client.list_models().await?;
            for model in models.models {
                eprintln!("  - {}", model.name);
            }
            eprintln!("\nTry pulling a model first:");
            eprintln!("  ollama pull qwen3:30b-a3b");
        }
        Err(e) => {
            eprintln!("Generation failed: {}", e);
        }
    }

    // Generate with system prompt
    println!("\n=== Generation with System Prompt ===");
    match client
        .generate()
        .model(model_name)
        .system("You are a helpful assistant that explains things like I'm 5 years old.")
        .prompt("How do computers work?")
        .temperature(0.8)
        .max_tokens(100)
        .send()
        .await
    {
        Ok(response) => {
            println!("Response: {}", response.response);
        }
        Err(e) => {
            eprintln!("Generation with system prompt failed: {}", e);
        }
    }

    Ok(())
}
