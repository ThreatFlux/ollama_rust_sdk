//! Common types used throughout the SDK

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP method types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
        }
    }
}

/// Request/response metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Additional metadata fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Progress information for long-running operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Current step in the operation
    pub step: u32,
    /// Total number of steps
    pub total: u32,
    /// Human-readable status message
    pub status: String,
    /// Optional detailed message
    pub detail: Option<String>,
}

/// API version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// API version string
    pub version: String,
    /// Build information
    pub build: Option<String>,
    /// Git commit hash
    pub commit: Option<String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Service status
    pub status: String,
    /// Timestamp of the check
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional health information
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
    /// Response metadata
    #[serde(default)]
    pub metadata: Metadata,
}

/// Generic error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: Option<String>,
    /// Error message
    pub message: String,
    /// Detailed error information
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// File upload information
#[derive(Debug, Clone)]
pub struct FileUpload {
    /// File name
    pub filename: String,
    /// File content
    pub content: Vec<u8>,
    /// MIME type
    pub mime_type: Option<String>,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Number of items per page
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Cursor-based pagination
    pub cursor: Option<String>,
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Response items
    pub items: Vec<T>,
    /// Total number of items
    pub total: Option<u32>,
    /// Next page cursor
    pub next_cursor: Option<String>,
    /// Whether there are more items
    pub has_more: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Put.as_str(), "PUT");
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
        assert_eq!(HttpMethod::Head.as_str(), "HEAD");
    }

    #[test]
    fn test_metadata_default() {
        let metadata = Metadata::default();
        assert!(metadata.request_id.is_none());
        assert!(metadata.extra.is_empty());
    }

    #[test]
    fn test_progress_serialization() {
        let progress = Progress {
            step: 5,
            total: 10,
            status: "Processing".to_string(),
            detail: Some("Working on step 5 of 10".to_string()),
        };

        let json = serde_json::to_string(&progress).unwrap();
        let deserialized: Progress = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.step, 5);
        assert_eq!(deserialized.total, 10);
        assert_eq!(deserialized.status, "Processing");
        assert_eq!(
            deserialized.detail,
            Some("Working on step 5 of 10".to_string())
        );
    }
}
