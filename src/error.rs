//! Error types for the Ollama SDK

use thiserror::Error;

/// Result type alias for Ollama operations
pub type Result<T> = std::result::Result<T, OllamaError>;

/// Comprehensive error types for Ollama SDK operations
#[derive(Error, Debug)]
pub enum OllamaError {
    /// Network-related errors (connection issues, timeouts, etc.)
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// URL parsing errors
    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),

    /// Model not found on the server
    #[error("Model '{0}' not found")]
    ModelNotFound(String),

    /// Invalid model name format
    #[error("Invalid model name: {0}")]
    InvalidModelName(String),

    /// Server returned an error response
    #[error("Server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    /// Request timeout
    #[error("Request timeout")]
    Timeout,

    /// Invalid API response format
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Streaming error
    #[error("Streaming error: {0}")]
    StreamError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid parameters provided
    #[error("Invalid parameter: {parameter} - {reason}")]
    InvalidParameter { parameter: String, reason: String },

    /// Model is currently loading
    #[error("Model '{0}' is currently loading, please try again")]
    ModelLoading(String),

    /// Insufficient system resources
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),

    /// Generic error for other cases
    #[error("Ollama error: {0}")]
    Other(String),
}

impl OllamaError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            OllamaError::NetworkError(_)
                | OllamaError::Timeout
                | OllamaError::ModelLoading(_)
                | OllamaError::ServerError {
                    status: 500..=599,
                    ..
                }
        )
    }

    /// Check if the error indicates the model is not available
    pub fn is_model_unavailable(&self) -> bool {
        matches!(
            self,
            OllamaError::ModelNotFound(_) | OllamaError::ModelLoading(_)
        )
    }

    /// Get the HTTP status code if this is a server error
    pub fn status_code(&self) -> Option<u16> {
        match self {
            OllamaError::ServerError { status, .. } => Some(*status),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest;
    use serde_json;
    use std::io;
    use url;

    #[tokio::test]
    async fn test_network_error_conversion() {
        // Create an actual reqwest error by making an invalid request
        let client = reqwest::Client::new();
        let result = client
            .get("http://invalid-domain-that-does-not-exist.test/")
            .send()
            .await;

        match result {
            Err(reqwest_error) => {
                let ollama_error: OllamaError = reqwest_error.into();
                assert!(matches!(ollama_error, OllamaError::NetworkError(_)));
                assert!(ollama_error.to_string().contains("Network error"));
                assert!(ollama_error.is_retryable());
            }
            Ok(_) => {
                // If the request somehow succeeds, just test the error type directly
                let test_error = OllamaError::Timeout;
                assert!(test_error.is_retryable());
            }
        }
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = r#"{"invalid": json}"#;
        let json_error =
            serde_json::from_str::<serde_json::Value>(&json_str[..json_str.len() - 1]).unwrap_err();
        let ollama_error: OllamaError = json_error.into();

        assert!(matches!(ollama_error, OllamaError::JsonError(_)));
        assert!(ollama_error.to_string().contains("JSON error"));
        assert!(!ollama_error.is_retryable());
    }

    #[test]
    fn test_url_error_conversion() {
        let url_error = url::ParseError::EmptyHost;
        let ollama_error: OllamaError = url_error.into();

        assert!(matches!(ollama_error, OllamaError::UrlError(_)));
        assert!(ollama_error.to_string().contains("Invalid URL"));
        assert!(!ollama_error.is_retryable());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let ollama_error: OllamaError = io_error.into();

        assert!(matches!(ollama_error, OllamaError::IoError(_)));
        assert!(ollama_error.to_string().contains("IO error"));
        assert!(!ollama_error.is_retryable());
    }

    #[test]
    fn test_model_not_found() {
        let error = OllamaError::ModelNotFound("llama3".to_string());

        assert_eq!(error.to_string(), "Model 'llama3' not found");
        assert!(!error.is_retryable());
        assert!(error.is_model_unavailable());
        assert_eq!(error.status_code(), None);
    }

    #[test]
    fn test_invalid_model_name() {
        let error = OllamaError::InvalidModelName("invalid/model".to_string());

        assert_eq!(error.to_string(), "Invalid model name: invalid/model");
        assert!(!error.is_retryable());
        assert!(!error.is_model_unavailable());
    }

    #[test]
    fn test_server_error_retryable() {
        let error = OllamaError::ServerError {
            status: 503,
            message: "Service Unavailable".to_string(),
        };

        assert!(error.to_string().contains("Server error: 503"));
        assert!(error.is_retryable());
        assert!(!error.is_model_unavailable());
        assert_eq!(error.status_code(), Some(503));
    }

    #[test]
    fn test_server_error_not_retryable() {
        let error = OllamaError::ServerError {
            status: 400,
            message: "Bad Request".to_string(),
        };

        assert!(error.to_string().contains("Server error: 400"));
        assert!(!error.is_retryable());
        assert_eq!(error.status_code(), Some(400));
    }

    #[test]
    fn test_timeout() {
        let error = OllamaError::Timeout;

        assert_eq!(error.to_string(), "Request timeout");
        assert!(error.is_retryable());
        assert!(!error.is_model_unavailable());
        assert_eq!(error.status_code(), None);
    }

    #[test]
    fn test_invalid_response() {
        let error = OllamaError::InvalidResponse("Missing required field".to_string());

        assert!(error.to_string().contains("Invalid response format"));
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_authentication_error() {
        let error = OllamaError::AuthenticationError("Invalid token".to_string());

        assert!(error.to_string().contains("Authentication failed"));
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let error = OllamaError::RateLimitExceeded;

        assert_eq!(error.to_string(), "Rate limit exceeded");
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_stream_error() {
        let error = OllamaError::StreamError("Connection lost".to_string());

        assert!(error.to_string().contains("Streaming error"));
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_config_error() {
        let error = OllamaError::ConfigError("Invalid endpoint".to_string());

        assert!(error.to_string().contains("Configuration error"));
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_invalid_parameter() {
        let error = OllamaError::InvalidParameter {
            parameter: "temperature".to_string(),
            reason: "must be between 0 and 2".to_string(),
        };

        assert!(error.to_string().contains("Invalid parameter: temperature"));
        assert!(error.to_string().contains("must be between 0 and 2"));
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_model_loading() {
        let error = OllamaError::ModelLoading("llama3".to_string());

        assert!(error
            .to_string()
            .contains("Model 'llama3' is currently loading"));
        assert!(error.is_retryable());
        assert!(error.is_model_unavailable());
    }

    #[test]
    fn test_insufficient_resources() {
        let error = OllamaError::InsufficientResources("Out of memory".to_string());

        assert!(error.to_string().contains("Insufficient resources"));
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_other_error() {
        let error = OllamaError::Other("Unexpected error".to_string());

        assert!(error.to_string().contains("Ollama error"));
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_debug_formatting() {
        let error = OllamaError::ModelNotFound("test-model".to_string());
        let debug_str = format!("{:?}", error);

        assert!(debug_str.contains("ModelNotFound"));
        assert!(debug_str.contains("test-model"));
    }
}
