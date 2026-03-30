# Ollama Rust SDK

[![CI](https://github.com/ThreatFlux/ollama_rust_sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/ThreatFlux/ollama_rust_sdk/actions/workflows/ci.yml)
[![Security](https://github.com/ThreatFlux/ollama_rust_sdk/actions/workflows/security.yml/badge.svg)](https://github.com/ThreatFlux/ollama_rust_sdk/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.94.0%2B-blue.svg)](https://www.rust-lang.org)

A comprehensive Rust SDK for interacting with the Ollama API. Type-safe, async-first bindings for text generation, chat, embeddings, and model management.

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
ollama_rust_sdk = "0.1.1"
tokio = { version = "1.50", features = ["full"] }
```

## Quick Start

```rust
use ollama_rust_sdk::OllamaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::new("http://localhost:11434")?;

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

```rust
use ollama_rust_sdk::{OllamaClient, ClientConfig};

let config = ClientConfig::builder()
    .base_url("http://localhost:11434")
    .timeout(std::time::Duration::from_secs(120))
    .build();

let client = OllamaClient::with_config(config)?;
```

Environment variables are also supported:

| Variable | Description | Default |
|----------|-------------|---------|
| `OLLAMA_BASE_URL` | Ollama server URL | `http://127.0.0.1:11434` |
| `OLLAMA_TIMEOUT_SECS` | Request timeout in seconds | `30` |
| `OLLAMA_USER_AGENT` | Custom user agent string | - |
| `OLLAMA_API_HEADERS` | Custom headers as JSON | - |

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

## Development

```bash
# Install development tools
make dev-setup

# Run all CI checks locally
make ci-local

# Format, lint, test
make fmt
make lint
make test

# Run with coverage
make coverage
```

## Requirements

- Rust 1.94.0 or higher
- Ollama server running locally or remotely
- At least one model pulled in Ollama

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

To report a vulnerability, see [SECURITY.md](SECURITY.md).
