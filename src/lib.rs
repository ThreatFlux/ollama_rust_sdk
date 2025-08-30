//! # Ollama Rust SDK
//!
//! A comprehensive Rust SDK for interacting with the Ollama API.
//!
//! This crate provides type-safe, async-first bindings for all Ollama API endpoints
//! including text generation, chat, embeddings, and model management.
//!
//! ## Features
//!
//! - Async/await support with tokio
//! - Type-safe API with proper error handling
//! - Streaming support for real-time text generation
//! - Builder pattern for easy request configuration
//! - Comprehensive model management
//! - Embedding generation with batch processing
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ollama_rust_sdk::OllamaClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OllamaClient::new("http://localhost:11434")?;
//!     
//!     let response = client
//!         .generate()
//!         .model("qwen3:30b-a3b")
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
