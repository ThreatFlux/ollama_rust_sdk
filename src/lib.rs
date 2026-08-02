//! # Ollama Rust SDK
//!
//! An async, type-safe Rust client for supported native Ollama API endpoints.
//!
//! This community-maintained crate provides builder-based generation and chat,
//! typed streaming, embeddings, and selected server, model, and blob operations.
//! It is not affiliated with, endorsed by, or maintained by Ollama.
//!
//! ## Features
//!
//! - Async/await support with tokio
//! - Type-safe API with proper error handling
//! - Streaming support for real-time text generation
//! - Builder pattern for easy request configuration
//! - Model listing, inspection, pull, create, copy, delete, and running-state operations
//! - Embedding generation with batch processing
//!
//! For the exact implemented surface and known limitations, see the
//! [API coverage matrix](https://github.com/ThreatFlux/ollama_rust_sdk/blob/main/docs/api-coverage.md).
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ollama_rust_sdk::OllamaClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OllamaClient::from_env()?;
//!     let model = std::env::var("OLLAMA_MODEL")?;
//!
//!     let response = client
//!         .generate()
//!         .model(model)
//!         .prompt("Why is the sky blue?")
//!         .send()
//!         .await?;
//!
//!     println!("Response: {}", response.response);
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod builders;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod streaming;
pub mod types;
pub mod utils;

// Re-export main types for convenience
pub use client::OllamaClient;
pub use config::{ClientConfig, ClientConfigBuilder};
pub use error::{OllamaError, Result};

// Re-export commonly used types
pub use models::{
    chat::{ChatMessage, ChatRequest, ChatResponse, MessageRole},
    common::{Options, ToolCall, ToolFunction},
    embedding::{EmbedRequest, EmbedResponse},
    generation::{GenerateRequest, GenerateResponse},
    model_info::{ModelDetails, ModelInfo, ModelList},
};

// Re-export builders
pub use builders::{chat_builder::ChatBuilder, generate_builder::GenerateBuilder};

// Re-export streaming types
pub use streaming::stream::{ChatStream, GenerateStream, StreamChunk};
