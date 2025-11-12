//! HTTP client utilities

use crate::{
    config::ClientConfig,
    error::{OllamaError, Result},
};
use reqwest::{Client, RequestBuilder, Response};
use serde::Serialize;

/// HTTP client wrapper for Ollama API requests
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: ClientConfig,
}

impl HttpClient {
    /// Create a new HTTP client with the given configuration
    pub fn new(config: ClientConfig) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent);

        if config.follow_redirects {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::limited(10));
        } else {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::none());
        }

        let client = client_builder.build().map_err(|e| {
            OllamaError::ConfigError(format!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self { client, config })
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Result<Response> {
        let url = self.config.endpoint_url(path)?;
        let request = self.client.get(url);
        self.send_request(request).await
    }

    /// Make a POST request
    pub fn post(&self, path: &str) -> PostRequestBuilder<'_> {
        let url = self.config.endpoint_url(path).expect("Valid URL");
        PostRequestBuilder {
            request: self.client.post(url),
            http_client: self,
        }
    }

    /// Make a PUT request
    pub fn put(&self, path: &str) -> PutRequestBuilder<'_> {
        let url = self.config.endpoint_url(path).expect("Valid URL");
        PutRequestBuilder {
            request: self.client.put(url),
            http_client: self,
        }
    }

    /// Make a DELETE request
    pub fn delete(&self, path: &str) -> DeleteRequestBuilder<'_> {
        let url = self.config.endpoint_url(path).expect("Valid URL");
        DeleteRequestBuilder {
            request: self.client.delete(url),
            http_client: self,
        }
    }

    /// Make a HEAD request
    pub async fn head(&self, path: &str) -> Result<Response> {
        let url = self.config.endpoint_url(path)?;
        let request = self.client.head(url);
        self.send_request(request).await
    }

    /// Send a request with common headers and error handling
    async fn send_request(&self, mut request: RequestBuilder) -> Result<Response> {
        // Add custom headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        // Add content type for JSON requests
        request = request.header("Content-Type", "application/json");

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                OllamaError::Timeout
            } else {
                OllamaError::NetworkError(e)
            }
        })?;

        Ok(response)
    }
}

/// Builder for POST requests
pub struct PostRequestBuilder<'a> {
    request: RequestBuilder,
    http_client: &'a HttpClient,
}

impl<'a> PostRequestBuilder<'a> {
    /// Set JSON body
    pub fn json<T: Serialize>(mut self, json: &T) -> Self {
        self.request = self.request.json(json);
        self
    }

    /// Set raw body
    pub fn body<T: Into<reqwest::Body>>(mut self, body: T) -> Self {
        self.request = self.request.body(body);
        self
    }

    /// Add a header
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.request = self.request.header(key.as_ref(), value.as_ref());
        self
    }

    /// Send the request
    pub async fn send(self) -> Result<Response> {
        self.http_client.send_request(self.request).await
    }
}

/// Builder for PUT requests
pub struct PutRequestBuilder<'a> {
    request: RequestBuilder,
    http_client: &'a HttpClient,
}

/// Builder for DELETE requests
pub struct DeleteRequestBuilder<'a> {
    request: RequestBuilder,
    http_client: &'a HttpClient,
}

impl<'a> PutRequestBuilder<'a> {
    /// Set raw body
    pub fn body<T: Into<reqwest::Body>>(mut self, body: T) -> Self {
        self.request = self.request.body(body);
        self
    }

    /// Add a header
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.request = self.request.header(key.as_ref(), value.as_ref());
        self
    }

    /// Send the request
    pub async fn send(self) -> Result<Response> {
        self.http_client.send_request(self.request).await
    }
}

impl<'a> DeleteRequestBuilder<'a> {
    /// Set JSON body
    pub fn json<T: Serialize>(mut self, json: &T) -> Self {
        self.request = self.request.json(json);
        self
    }

    /// Add a header
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.request = self.request.header(key.as_ref(), value.as_ref());
        self
    }

    /// Send the request
    pub async fn send(self) -> Result<Response> {
        self.http_client.send_request(self.request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let config = ClientConfig::default();
        let client = HttpClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_config_with_custom_headers() {
        let mut config = ClientConfig::default();
        config
            .headers
            .insert("X-Test".to_string(), "value".to_string());

        let client = HttpClient::new(config);
        assert!(client.is_ok());
    }
}
