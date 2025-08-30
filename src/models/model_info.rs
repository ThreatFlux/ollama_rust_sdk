//! Model information and management structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about a single model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Model name
    pub name: String,

    /// Model size in bytes
    pub size: u64,

    /// Digest/hash of the model
    pub digest: String,

    /// When the model was last modified
    #[serde(default)]
    pub modified_at: Option<DateTime<Utc>>,

    /// Model details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ModelDetails>,
}

/// List of models response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelList {
    /// Available models
    pub models: Vec<Model>,
}

/// Detailed model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// License information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Modelfile content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modelfile: Option<String>,

    /// Model parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<String>,

    /// Template used by the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// System message/prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Model details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ModelDetails>,

    /// Model messages (conversation examples)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<crate::models::chat::ChatMessage>>,
}

/// Detailed technical information about a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    /// Model family (e.g., "qwen", "llama")
    pub family: String,

    /// Model format (e.g., "gguf")
    pub format: String,

    /// Parameter size (e.g., "30B")
    pub parameter_size: String,

    /// Quantization level (e.g., "Q4_K_M")
    pub quantization_level: String,

    /// Families this model belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub families: Option<Vec<String>>,

    /// Parent model if this is a variant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_model: Option<String>,
}

/// Information about a running model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningModel {
    /// Model name
    pub name: String,

    /// Model size in bytes
    pub size: u64,

    /// Digest/hash of the model
    pub digest: String,

    /// Model details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ModelDetails>,

    /// When the model expires from memory
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,

    /// Size in memory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_vram: Option<u64>,
}

/// List of running models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningModels {
    /// Currently running models
    pub models: Vec<RunningModel>,
}

/// Model pull progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullProgress {
    /// Status of the pull operation
    pub status: String,

    /// Current digest being processed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,

    /// Total bytes to download
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,

    /// Bytes completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<u64>,
}

impl PullProgress {
    /// Calculate progress percentage (0.0 to 100.0)
    pub fn percentage(&self) -> Option<f64> {
        match (self.completed, self.total) {
            (Some(completed), Some(total)) if total > 0 => {
                Some((completed as f64 / total as f64) * 100.0)
            }
            _ => None,
        }
    }

    /// Check if the pull is complete
    pub fn is_complete(&self) -> bool {
        self.status.to_lowercase().contains("success")
            || self.status.to_lowercase().contains("complete")
            || (self.completed.is_some() && self.total.is_some() && self.completed == self.total)
    }
}

/// Model creation progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProgress {
    /// Status of the creation
    pub status: String,

    /// Progress details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Model copy request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyRequest {
    /// Source model name
    pub source: String,

    /// Destination model name
    pub destination: String,
}

/// Model delete request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRequest {
    /// Model name to delete
    pub name: String,
}

/// Model show request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowRequest {
    /// Model name to show
    pub name: String,

    /// Include verbose information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,
}

/// Model pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// Model name to pull
    pub name: String,

    /// Whether to stream progress updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Insecure mode (skip TLS verification)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure: Option<bool>,
}

/// Model create request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequest {
    /// Name for the new model
    pub name: String,

    /// Modelfile content
    pub modelfile: String,

    /// Whether to stream progress updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Quantization method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantize: Option<String>,
}

impl Model {
    /// Get a human-readable size string
    pub fn size_string(&self) -> String {
        format_bytes(self.size)
    }

    /// Check if this model is a fine-tune or custom model
    pub fn is_custom(&self) -> bool {
        self.name.contains(':') && !self.name.contains("latest")
    }

    /// Get the base model name (without tags)
    pub fn base_name(&self) -> &str {
        self.name.split(':').next().unwrap_or(&self.name)
    }

    /// Get the model tag (part after ':')
    pub fn tag(&self) -> Option<&str> {
        self.name.split(':').nth(1)
    }
}

impl RunningModel {
    /// Get a human-readable size string
    pub fn size_string(&self) -> String {
        format_bytes(self.size)
    }

    /// Get VRAM usage as a string
    pub fn vram_string(&self) -> String {
        self.size_vram
            .map(format_bytes)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Check if the model is about to expire soon (within 1 minute)
    pub fn expires_soon(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => {
                let now = Utc::now();
                let time_until_expiry = expires_at.signed_duration_since(now);
                time_until_expiry.num_minutes() <= 1
            }
            None => false,
        }
    }
}

/// Helper function to format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_model_methods() {
        let model = Model {
            name: "qwen3:30b-a3b".to_string(),
            size: 1073741824, // 1GB
            digest: "sha256:abc123".to_string(),
            modified_at: None,
            details: None,
        };

        assert_eq!(model.base_name(), "qwen3");
        assert_eq!(model.tag(), Some("30b-a3b"));
        assert!(model.is_custom());
        assert_eq!(model.size_string(), "1.0 GB");
    }

    #[test]
    fn test_pull_progress() {
        let progress = PullProgress {
            status: "downloading".to_string(),
            digest: Some("sha256:abc".to_string()),
            total: Some(1000),
            completed: Some(500),
        };

        assert_eq!(progress.percentage(), Some(50.0));
        assert!(!progress.is_complete());

        let complete_progress = PullProgress {
            status: "success".to_string(),
            digest: None,
            total: None,
            completed: None,
        };

        assert!(complete_progress.is_complete());
    }
}
