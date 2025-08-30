//! Generation API request and response models

use crate::models::common::{KeepAlive, Options, ResponseFormat};
use serde::{Deserialize, Serialize};

/// Request for text generation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerateRequest {
    /// Model to use for generation
    pub model: String,

    /// Text prompt for generation
    pub prompt: String,

    /// Enable or disable streaming
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// System message to provide context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Template string for formatting the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Context from previous requests for conversation continuity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,

    /// Additional generation options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,

    /// Response format (json, text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ResponseFormat>,

    /// Use raw prompt without formatting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,

    /// How long to keep the model loaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,

    /// Images to include with the prompt (for multimodal models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

impl GenerateRequest {
    /// Create a new generate request
    pub fn new<S: Into<String>>(model: S, prompt: S) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    /// Set whether to stream the response
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Set the system message
    pub fn system<S: Into<String>>(mut self, system: S) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set the generation options
    pub fn options(mut self, options: Options) -> Self {
        self.options = Some(options);
        self
    }

    /// Set the response format
    pub fn format(mut self, format: ResponseFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set keep alive duration
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }
}

/// Response from text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    /// The model that was used
    pub model: String,

    /// The generated response text
    pub response: String,

    /// Whether this is the final response
    pub done: bool,

    /// Context for conversation continuity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i32>>,

    /// Total duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,

    /// Load duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,

    /// Prompt evaluation count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,

    /// Prompt evaluation duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,

    /// Evaluation count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u32>,

    /// Evaluation duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

impl GenerateResponse {
    /// Get tokens per second for prompt evaluation
    pub fn prompt_eval_rate(&self) -> Option<f64> {
        match (self.prompt_eval_count, self.prompt_eval_duration) {
            (Some(count), Some(duration)) if duration > 0 => {
                Some(count as f64 / (duration as f64 / 1e9))
            }
            _ => None,
        }
    }

    /// Get tokens per second for generation
    pub fn eval_rate(&self) -> Option<f64> {
        match (self.eval_count, self.eval_duration) {
            (Some(count), Some(duration)) if duration > 0 => {
                Some(count as f64 / (duration as f64 / 1e9))
            }
            _ => None,
        }
    }

    /// Get total tokens per second
    pub fn total_rate(&self) -> Option<f64> {
        match (self.prompt_eval_count, self.eval_count, self.total_duration) {
            (Some(prompt_count), Some(eval_count), Some(duration)) if duration > 0 => {
                Some((prompt_count + eval_count) as f64 / (duration as f64 / 1e9))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_request_creation() {
        let request = GenerateRequest::new("test-model", "test prompt");
        assert_eq!(request.model, "test-model");
        assert_eq!(request.prompt, "test prompt");
        assert!(request.stream.is_none());
    }

    #[test]
    fn test_generate_request_builder() {
        let request = GenerateRequest::new("test-model", "test prompt")
            .stream(true)
            .system("You are a helpful assistant")
            .format(ResponseFormat::Json);

        assert_eq!(request.stream, Some(true));
        assert_eq!(
            request.system,
            Some("You are a helpful assistant".to_string())
        );
        matches!(request.format, Some(ResponseFormat::Json));
    }

    #[test]
    fn test_generate_response_rates() {
        let response = GenerateResponse {
            model: "test".to_string(),
            response: "test".to_string(),
            done: true,
            context: None,
            total_duration: Some(2_000_000_000), // 2 seconds
            load_duration: None,
            prompt_eval_count: Some(10),
            prompt_eval_duration: Some(1_000_000_000), // 1 second
            eval_count: Some(20),
            eval_duration: Some(1_000_000_000), // 1 second
        };

        assert_eq!(response.prompt_eval_rate(), Some(10.0));
        assert_eq!(response.eval_rate(), Some(20.0));
        assert_eq!(response.total_rate(), Some(15.0)); // (10 + 20) / 2
    }
}
