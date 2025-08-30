//! Configuration for the Ollama client

use crate::error::{OllamaError, Result};
use std::time::Duration;
use url::Url;

/// Configuration for the Ollama client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL for the Ollama API
    pub base_url: Url,
    /// Request timeout duration
    pub timeout: Duration,
    /// User agent string
    pub user_agent: String,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Delay between retries
    pub retry_delay: Duration,
    /// Whether to follow HTTP redirects
    pub follow_redirects: bool,
    /// Custom headers to include in requests
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("http://localhost:11434").expect("Default URL should be valid"),
            timeout: Duration::from_secs(120),
            user_agent: format!("ollama-rust-sdk/{}", env!("CARGO_PKG_VERSION")),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            follow_redirects: true,
            headers: std::collections::HashMap::new(),
        }
    }
}

impl ClientConfig {
    /// Create a new client configuration with the specified base URL
    pub fn new<U: AsRef<str>>(base_url: U) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())
            .map_err(|e| OllamaError::ConfigError(format!("Invalid base URL: {}", e)))?;

        Ok(Self {
            base_url,
            ..Default::default()
        })
    }

    /// Create a builder for client configuration
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }

    /// Get the full URL for an API endpoint
    pub fn endpoint_url(&self, path: &str) -> Result<Url> {
        let path = if path.starts_with('/') {
            path
        } else {
            &format!("/{}", path)
        };

        self.base_url.join(path).map_err(|e| {
            OllamaError::ConfigError(format!("Invalid endpoint path '{}': {}", path, e))
        })
    }
}

/// Builder for client configuration
#[derive(Debug, Default)]
pub struct ClientConfigBuilder {
    base_url: Option<String>,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    max_retries: Option<u32>,
    retry_delay: Option<Duration>,
    follow_redirects: Option<bool>,
    headers: std::collections::HashMap<String, String>,
}

impl ClientConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL for the Ollama API
    pub fn base_url<U: Into<String>>(mut self, url: U) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the user agent string
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set the maximum number of retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Set the delay between retries
    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = Some(delay);
        self
    }

    /// Set whether to follow HTTP redirects
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = Some(follow);
        self
    }

    /// Add a custom header
    pub fn header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Build the client configuration
    pub fn build(self) -> Result<ClientConfig> {
        let base_url = match self.base_url {
            Some(url) => Url::parse(&url)
                .map_err(|e| OllamaError::ConfigError(format!("Invalid base URL: {}", e)))?,
            None => ClientConfig::default().base_url,
        };

        Ok(ClientConfig {
            base_url,
            timeout: self
                .timeout
                .unwrap_or_else(|| ClientConfig::default().timeout),
            user_agent: self
                .user_agent
                .unwrap_or_else(|| ClientConfig::default().user_agent),
            max_retries: self
                .max_retries
                .unwrap_or_else(|| ClientConfig::default().max_retries),
            retry_delay: self
                .retry_delay
                .unwrap_or_else(|| ClientConfig::default().retry_delay),
            follow_redirects: self
                .follow_redirects
                .unwrap_or_else(|| ClientConfig::default().follow_redirects),
            headers: self.headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert_eq!(config.base_url.as_str(), "http://localhost:11434/");
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert!(config.user_agent.contains("ollama-rust-sdk"));
    }

    #[test]
    fn test_config_builder() {
        let config = ClientConfig::builder()
            .base_url("http://example.com:8080")
            .timeout(Duration::from_secs(60))
            .max_retries(5)
            .header("X-Custom", "value")
            .build()
            .unwrap();

        assert_eq!(config.base_url.as_str(), "http://example.com:8080/");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_endpoint_url() {
        let config = ClientConfig::default();

        let url = config.endpoint_url("api/generate").unwrap();
        assert_eq!(url.as_str(), "http://localhost:11434/api/generate");

        let url = config.endpoint_url("/api/chat").unwrap();
        assert_eq!(url.as_str(), "http://localhost:11434/api/chat");
    }
}
