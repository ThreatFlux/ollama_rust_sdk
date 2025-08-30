//! Additional options and configurations

use serde::{Deserialize, Serialize};

/// Model-specific configuration options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelOptions {
    /// Temperature setting for this model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Top-k setting for this model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,

    /// Top-p setting for this model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    /// Custom system prompt for this model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

/// Request options for fine-tuning behavior
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestOptions {
    /// Request timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    /// Maximum retries for failed requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,

    /// Custom headers to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,

    /// Enable debug logging
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
}

/// Streaming configuration options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamOptions {
    /// Buffer size for streaming responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_size: Option<usize>,

    /// Timeout for individual stream chunks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_timeout: Option<u64>,

    /// Whether to include partial responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_partial: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_options_default() {
        let opts = ModelOptions::default();
        assert!(opts.temperature.is_none());
        assert!(opts.top_k.is_none());
        assert!(opts.top_p.is_none());
        assert!(opts.system.is_none());
    }

    #[test]
    fn test_request_options_serialization() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Custom".to_string(), "value".to_string());

        let opts = RequestOptions {
            timeout: Some(30),
            max_retries: Some(3),
            headers: Some(headers),
            debug: Some(true),
        };

        let json = serde_json::to_string(&opts).unwrap();
        let deserialized: RequestOptions = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timeout, Some(30));
        assert_eq!(deserialized.max_retries, Some(3));
        assert_eq!(deserialized.debug, Some(true));
    }
}
