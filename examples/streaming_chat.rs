//! Streaming chat example

use ollama_rust_sdk::OllamaClient;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = OllamaClient::new("http://localhost:11434")?;

    // Check server health
    if !client.health().await? {
        eprintln!("Ollama server is not available");
        return Ok(());
    }

    // Get available model
    let models = client.list_models().await?;
    let model_name = models
        .models
        .first()
        .map(|m| m.name.as_str())
        .unwrap_or("qwen3:30b-a3b");

    println!("Using model: {}", model_name);
    println!("Starting streaming chat example..\n");

    // Create a streaming chat request
    let mut stream = client
        .chat()
        .model(model_name)
        .add_system_message(
            "You are a helpful AI assistant. Keep your responses concise but informative.",
        )
        .add_user_message("Tell me an interesting fact about space exploration.")
        .temperature(0.7)
        .stream()
        .await?;

    println!("Assistant: ");
    let mut full_response = String::new();

    // Process the stream
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(response) => {
                print!("{}", response.message.content);
                full_response.push_str(&response.message.content);

                // Flush stdout to see text appear in real-time
                use std::io::{self, Write};
                io::stdout().flush()?;

                if response.done {
                    println!("\n\n[Stream completed]");
                    if let Some(eval_count) = response.eval_count {
                        if let Some(eval_duration) = response.eval_duration {
                            let tokens_per_second =
                                eval_count as f64 / (eval_duration as f64 / 1e9);
                            println!("Tokens generated: {}", eval_count);
                            println!("Speed: {:.2} tokens/second", tokens_per_second);
                        }
                    }
                    break;
                }
            }
            Err(e) => {
                eprintln!("\nStream error: {}", e);
                break;
            }
        }
    }

    // Demonstrate multi-turn conversation
    println!("\n{}", "=".repeat(50));
    println!("Multi-turn conversation example:");
    println!("{}", "=".repeat(50));

    let response = client
        .chat()
        .model(model_name)
        .add_system_message("You are a knowledgeable science teacher.")
        .add_user_message("What is photosynthesis?")
        .add_assistant_message(&full_response) // Use previous response
        .add_user_message("Can you explain it in even simpler terms?")
        .temperature(0.6)
        .send()
        .await?;

    println!("\nTeacher: {}", response.message.content);

    Ok(())
}
