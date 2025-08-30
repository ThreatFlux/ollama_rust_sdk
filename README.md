# Ollama Rust SDK

A comprehensive Rust SDK for interacting with the Ollama API. This SDK provides type-safe, async-first bindings for all Ollama API endpoints including text generation, chat, embeddings, and model management.

## Features

- **Async/await support** with `tokio`
- **Type-safe API** with proper error handling
- **Streaming support** for real-time text generation
- **Builder pattern** for easy request configuration
- **Comprehensive model management** (pull, create, delete, list)
- **Embedding generation** with batch processing
- **CLI tool** for command-line usage
- **OpenAI-compatible endpoints** support

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ollama_rust_sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use ollama_rust_sdk::{OllamaClient, GenerateRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = OllamaClient::new("http://localhost:11434")?;
    
    // Generate text
    let response = client
        .generate()
        .model("qwen3:30b-a3b")
        .prompt("Why is the sky blue?")
        .send()
        .await?;
    
    println!("Response: {}", response.response);
    Ok(())
}
```

## Examples

### Basic Text Generation

```rust
use ollama_rust_sdk::OllamaClient;

let client = OllamaClient::new("http://localhost:11434")?;

let response = client
    .generate()
    .model("qwen3:30b-a3b")
    .prompt("Explain quantum computing")
    .temperature(0.7)
    .max_tokens(200)
    .send()
    .await?;
```

### Streaming Chat

```rust
use tokio_stream::StreamExt;

let mut stream = client
    .chat()
    .model("qwen3:30b-a3b")
    .add_user_message("Tell me a story")
    .stream()
    .await?;

while let Some(chunk) = stream.next().await {
    if let Ok(response) = chunk {
        print!("{}", response.message.content);
    }
}
```

### Generate Embeddings

```rust
let embeddings = client
    .embed()
    .model("qwen3-embedding:8b")
    .input(vec!["Hello world", "How are you?"])
    .send()
    .await?;
```

## CLI Usage

The SDK includes a CLI tool for interacting with Ollama from the command line:

```bash
# Generate text
ollama-cli generate "Why is the sky blue?" --model qwen3:30b-a3b

# Start a chat session
ollama-cli chat --model qwen3:30b-a3b

# List available models
ollama-cli models list

# Pull a model
ollama-cli models pull qwen3:30b-a3b
```

## Configuration

The SDK supports various configuration options:

```rust
use ollama_rust_sdk::{OllamaClient, ClientConfig};

let config = ClientConfig::builder()
    .base_url("http://localhost:11434")
    .timeout(std::time::Duration::from_secs(120))
    .build();

let client = OllamaClient::with_config(config)?;
```

## Model Management

```rust
// List all available models
let models = client.list_models().await?;

// Pull a new model
client.pull_model("qwen3:30b-a3b").await?;

// Show model information
let model_info = client.show_model("qwen3:30b-a3b").await?;

// Delete a model
client.delete_model("old-model").await?;
```

## Error Handling

The SDK provides comprehensive error handling:

```rust
use ollama_rust_sdk::{OllamaError, OllamaClient};

match client.generate().model("invalid-model").send().await {
    Ok(response) => println!("Success: {}", response.response),
    Err(OllamaError::ModelNotFound(model)) => {
        eprintln!("Model '{}' not found", model);
    }
    Err(OllamaError::NetworkError(err)) => {
        eprintln!("Network error: {}", err);
    }
    Err(err) => eprintln!("Other error: {}", err),
}
```

## Supported Models

The SDK works with all Ollama-compatible models, including:

- **Qwen3**: `qwen3:30b-a3b`, `qwen3:7b`
- **GPT-OSS**: `gpt-oss:20b`
- **Llama**: `llama2`, `llama3`
- **Code Llama**: `codellama`
- **Embeddings**: `qwen3-embedding:8b`

## Requirements

- Rust 1.70 or higher
- Ollama server running locally or remotely
- At least one model pulled in Ollama

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.