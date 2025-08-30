//! Embedding API request and response models

use crate::models::common::{KeepAlive, Options};
use serde::{Deserialize, Serialize};

/// Input for embedding requests - can be a single string or array of strings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbedInput {
    /// Single text input
    Single(String),
    /// Multiple text inputs
    Multiple(Vec<String>),
}

impl From<String> for EmbedInput {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<&str> for EmbedInput {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<Vec<String>> for EmbedInput {
    fn from(v: Vec<String>) -> Self {
        Self::Multiple(v)
    }
}

impl From<Vec<&str>> for EmbedInput {
    fn from(v: Vec<&str>) -> Self {
        Self::Multiple(v.into_iter().map(|s| s.to_string()).collect())
    }
}

impl Default for EmbedInput {
    fn default() -> Self {
        Self::Single(String::new())
    }
}

/// Request for generating embeddings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbedRequest {
    /// Model to use for embeddings
    pub model: String,

    /// Input text(s) to embed
    pub input: EmbedInput,

    /// Additional options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,

    /// How long to keep the model loaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,

    /// Whether to truncate inputs that are too long
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<bool>,
}

impl EmbedRequest {
    /// Create a new embedding request
    pub fn new<S: Into<String>, I: Into<EmbedInput>>(model: S, input: I) -> Self {
        Self {
            model: model.into(),
            input: input.into(),
            ..Default::default()
        }
    }

    /// Set additional options
    pub fn options(mut self, options: Options) -> Self {
        self.options = Some(options);
        self
    }

    /// Set keep alive duration
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }

    /// Set whether to truncate inputs
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = Some(truncate);
        self
    }

    /// Get the number of inputs
    pub fn input_count(&self) -> usize {
        match &self.input {
            EmbedInput::Single(_) => 1,
            EmbedInput::Multiple(v) => v.len(),
        }
    }

    /// Get the inputs as a vector of strings
    pub fn inputs_as_vec(&self) -> Vec<String> {
        match &self.input {
            EmbedInput::Single(s) => vec![s.clone()],
            EmbedInput::Multiple(v) => v.clone(),
        }
    }
}

/// Response from embedding generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    /// The model that was used
    pub model: String,

    /// Generated embeddings (array of arrays of floats)
    pub embeddings: Vec<Vec<f64>>,

    /// Total duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,

    /// Load duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,

    /// Prompt evaluation count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,
}

impl EmbedResponse {
    /// Get the number of embeddings
    pub fn count(&self) -> usize {
        self.embeddings.len()
    }

    /// Get the dimensionality of the embeddings (assumes all have same dimensions)
    pub fn dimensions(&self) -> Option<usize> {
        self.embeddings.first().map(|emb| emb.len())
    }

    /// Get a specific embedding by index
    pub fn get_embedding(&self, index: usize) -> Option<&Vec<f64>> {
        self.embeddings.get(index)
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> Option<f64> {
        if a.len() != b.len() {
            return None;
        }

        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Some(0.0);
        }

        Some(dot_product / (norm_a * norm_b))
    }

    /// Calculate Euclidean distance between two embeddings
    pub fn euclidean_distance(a: &[f64], b: &[f64]) -> Option<f64> {
        if a.len() != b.len() {
            return None;
        }

        let distance: f64 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt();

        Some(distance)
    }
}

/// Legacy embedding request format (deprecated but still supported)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyEmbeddingRequest {
    /// Model to use
    pub model: String,

    /// Text prompt to embed
    pub prompt: String,

    /// Additional options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,

    /// How long to keep the model loaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,
}

/// Legacy embedding response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyEmbeddingResponse {
    /// Generated embedding
    pub embedding: Vec<f64>,

    /// The model used
    pub model: String,

    /// Total duration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,

    /// Load duration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,

    /// Prompt evaluation count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_input_from_string() {
        let input: EmbedInput = "test".into();
        match input {
            EmbedInput::Single(s) => assert_eq!(s, "test"),
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn test_embed_input_from_vec() {
        let input: EmbedInput = vec!["test1", "test2"].into();
        match input {
            EmbedInput::Multiple(v) => assert_eq!(v, vec!["test1", "test2"]),
            _ => panic!("Expected Multiple variant"),
        }
    }

    #[test]
    fn test_embed_request_creation() {
        let request = EmbedRequest::new("test-model", "test text");
        assert_eq!(request.model, "test-model");
        assert_eq!(request.input_count(), 1);
        assert_eq!(request.inputs_as_vec(), vec!["test text"]);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let c = vec![1.0, 0.0, 0.0];

        assert_eq!(EmbedResponse::cosine_similarity(&a, &b), Some(0.0));
        assert_eq!(EmbedResponse::cosine_similarity(&a, &c), Some(1.0));
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];

        assert_eq!(EmbedResponse::euclidean_distance(&a, &b), Some(5.0));
    }
}
