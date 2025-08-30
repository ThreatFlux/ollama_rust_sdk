//! Main client for interacting with the Ollama API

use crate::{
    api::{blobs::BlobsApi, embeddings::EmbeddingsApi, models::ModelsApi},
    builders::{chat_builder::ChatBuilder, generate_builder::GenerateBuilder},
    config::ClientConfig,
    error::{OllamaError, Result},
    models::{
        embedding::EmbedRequest,
        model_info::{ModelInfo, ModelList, RunningModels},
    },
    utils::http::HttpClient,
};
use std::sync::Arc;

/// Main client for interacting with the Ollama API
#[derive(Debug, Clone)]
pub struct OllamaClient {
    /// HTTP client for making requests
    http_client: Arc<HttpClient>,
    /// Client configuration
    config: Arc<ClientConfig>,
}

impl OllamaClient {
    /// Create a new Ollama client with the default configuration
    pub fn new<U: AsRef<str>>(base_url: U) -> Result<Self> {
        let config = ClientConfig::new(base_url)?;
        Self::with_config(config)
    }

    /// Create a new Ollama client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let http_client = HttpClient::new(config.clone())?;

        Ok(Self {
            http_client: Arc::new(http_client),
            config: Arc::new(config),
        })
    }

    /// Get the client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Check if the Ollama server is healthy
    pub async fn health(&self) -> Result<bool> {
        match self.http_client.get("").await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get the Ollama server version
    pub async fn version(&self) -> Result<serde_json::Value> {
        let response = self.http_client.get("api/version").await?;
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;
        Ok(json)
    }

    // Generation API methods

    /// Create a generate request builder
    pub fn generate(&self) -> GenerateBuilder {
        GenerateBuilder::new(self.http_client.clone())
    }

    // Chat API methods

    /// Create a chat request builder
    pub fn chat(&self) -> ChatBuilder {
        ChatBuilder::new(self.http_client.clone())
    }

    // Embeddings API methods

    /// Create an embeddings request builder
    pub fn embed(&self) -> EmbedRequestBuilder {
        EmbedRequestBuilder::new(self.http_client.clone())
    }

    // Model Management API methods

    /// List all available models
    pub async fn list_models(&self) -> Result<ModelList> {
        ModelsApi::list_models(&self.http_client).await
    }

    /// Get information about a specific model
    pub async fn show_model(&self, name: &str) -> Result<ModelInfo> {
        ModelsApi::show_model(&self.http_client, name).await
    }

    /// Pull a model from the registry
    pub async fn pull_model(&self, name: &str) -> Result<()> {
        ModelsApi::pull_model(&self.http_client, name, false).await
    }

    /// Pull a model with streaming progress updates
    pub async fn pull_model_stream(
        &self,
        name: &str,
    ) -> Result<impl tokio_stream::Stream<Item = Result<serde_json::Value>>> {
        ModelsApi::pull_model_stream(&self.http_client, name).await
    }

    /// Create a new model from a Modelfile
    pub async fn create_model(&self, name: &str, modelfile: &str) -> Result<()> {
        ModelsApi::create_model(&self.http_client, name, modelfile, false).await
    }

    /// Create a model with streaming progress updates
    pub async fn create_model_stream(
        &self,
        name: &str,
        modelfile: &str,
    ) -> Result<impl tokio_stream::Stream<Item = Result<serde_json::Value>>> {
        ModelsApi::create_model_stream(&self.http_client, name, modelfile).await
    }

    /// Copy a model
    pub async fn copy_model(&self, source: &str, destination: &str) -> Result<()> {
        ModelsApi::copy_model(&self.http_client, source, destination).await
    }

    /// Delete a model
    pub async fn delete_model(&self, name: &str) -> Result<()> {
        ModelsApi::delete_model(&self.http_client, name).await
    }

    /// List currently running models
    pub async fn list_running_models(&self) -> Result<RunningModels> {
        ModelsApi::list_running_models(&self.http_client).await
    }

    // Blob Management API methods

    /// Check if a blob exists
    pub async fn blob_exists(&self, digest: &str) -> Result<bool> {
        BlobsApi::blob_exists(&self.http_client, digest).await
    }

    /// Create/upload a blob
    pub async fn create_blob(&self, digest: &str, data: Vec<u8>) -> Result<()> {
        BlobsApi::create_blob(&self.http_client, digest, data).await
    }
}

/// Builder for embedding requests
#[derive(Debug)]
pub struct EmbedRequestBuilder {
    http_client: Arc<HttpClient>,
    request: EmbedRequest,
}

impl EmbedRequestBuilder {
    fn new(http_client: Arc<HttpClient>) -> Self {
        Self {
            http_client,
            request: EmbedRequest::default(),
        }
    }

    /// Set the model to use for embeddings
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.request.model = model.into();
        self
    }

    /// Set the input text(s) to embed
    pub fn input<I>(mut self, input: I) -> Self
    where
        I: Into<crate::models::embedding::EmbedInput>,
    {
        self.request.input = input.into();
        self
    }

    /// Set additional options
    pub fn options(mut self, options: crate::models::common::Options) -> Self {
        self.request.options = Some(options);
        self
    }

    /// Set keep alive duration
    pub fn keep_alive(mut self, keep_alive: crate::models::common::KeepAlive) -> Self {
        self.request.keep_alive = Some(keep_alive);
        self
    }

    /// Enable/disable truncation
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.request.truncate = Some(truncate);
        self
    }

    /// Send the embedding request
    pub async fn send(self) -> Result<crate::models::embedding::EmbedResponse> {
        EmbeddingsApi::embed(&self.http_client, self.request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::{KeepAlive, Options};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new("http://localhost:11434");
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_with_https() {
        let client = OllamaClient::new("https://api.example.com:8080");
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_invalid_url() {
        let client = OllamaClient::new("invalid-url");
        assert!(client.is_err());
    }

    #[test]
    fn test_client_with_empty_url() {
        let client = OllamaClient::new("");
        assert!(client.is_err());
    }

    #[test]
    fn test_client_with_config() {
        let config = ClientConfig::default();
        let client = OllamaClient::with_config(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_config_access() {
        let client = OllamaClient::new("http://localhost:11434").unwrap();
        let config = client.config();

        assert_eq!(config.base_url.as_str(), "http://localhost:11434/");
        assert!(config.timeout.as_secs() > 0);
        assert!(!config.user_agent.is_empty());
    }

    #[tokio::test]
    async fn test_health_check_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(""))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(mock_server.uri()).unwrap();
        let is_healthy = client.health().await.unwrap();

        assert!(is_healthy);
    }

    #[tokio::test]
    async fn test_health_check_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(""))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(mock_server.uri()).unwrap();
        let is_healthy = client.health().await.unwrap();

        assert!(!is_healthy);
    }

    #[tokio::test]
    async fn test_health_check_network_error() {
        let client = OllamaClient::new("http://nonexistent.example.com:12345").unwrap();
        let is_healthy = client.health().await.unwrap();

        // Should return false on network errors
        assert!(!is_healthy);
    }

    #[tokio::test]
    async fn test_version_success() {
        let mock_server = MockServer::start().await;

        let version_response = r#"{"version":"0.1.0"}"#;

        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_string(version_response))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(mock_server.uri()).unwrap();
        let version = client.version().await.unwrap();

        assert_eq!(version["version"], "0.1.0");
    }

    #[tokio::test]
    async fn test_version_invalid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(mock_server.uri()).unwrap();
        let result = client.version().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OllamaError::InvalidResponse(_)
        ));
    }

    #[test]
    fn test_generate_builder_creation() {
        let client = OllamaClient::new("http://localhost:11434").unwrap();
        let builder = client.generate();

        // Should create builder without errors
        drop(builder);
    }

    #[test]
    fn test_chat_builder_creation() {
        let client = OllamaClient::new("http://localhost:11434").unwrap();
        let builder = client.chat();

        // Should create builder without errors
        drop(builder);
    }

    #[test]
    fn test_embed_builder() {
        let config = ClientConfig::default();
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let builder = EmbedRequestBuilder::new(http_client)
            .model("test-model")
            .input("test text")
            .truncate(true);

        assert_eq!(builder.request.model, "test-model");
        assert_eq!(builder.request.truncate, Some(true));
    }

    #[test]
    fn test_embed_builder_with_options() {
        let config = ClientConfig::default();
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let options = Options::default();
        let keep_alive = KeepAlive::Duration("30s".to_string());

        let builder = EmbedRequestBuilder::new(http_client)
            .model("embedding-model")
            .input(vec!["text1".to_string(), "text2".to_string()])
            .options(options)
            .keep_alive(keep_alive)
            .truncate(false);

        assert_eq!(builder.request.model, "embedding-model");
        assert_eq!(builder.request.truncate, Some(false));
        assert!(builder.request.options.is_some());
        assert!(builder.request.keep_alive.is_some());
    }

    #[test]
    fn test_embed_builder_with_different_input_types() {
        let config = ClientConfig::default();
        let http_client = Arc::new(HttpClient::new(config.clone()).unwrap());

        // Test with string input
        let builder1 = EmbedRequestBuilder::new(http_client.clone())
            .model("test-model")
            .input("single text");

        // Test with vec input
        let builder2 = EmbedRequestBuilder::new(http_client)
            .model("test-model")
            .input(vec!["text1".to_string(), "text2".to_string()]);

        assert_eq!(builder1.request.model, "test-model");
        assert_eq!(builder2.request.model, "test-model");
    }

    #[test]
    fn test_client_builder_methods() {
        let client = OllamaClient::new("http://localhost:11434").unwrap();

        // Test that builders can be created
        let _generate_builder = client.generate();
        let _chat_builder = client.chat();
        let _embed_builder = client.embed();
    }

    #[test]
    fn test_client_clone() {
        let client1 = OllamaClient::new("http://localhost:11434").unwrap();
        let client2 = client1.clone();

        // Both clients should have the same configuration
        assert_eq!(client1.config().base_url, client2.config().base_url);
        assert_eq!(client1.config().timeout, client2.config().timeout);
    }

    #[tokio::test]
    async fn test_list_models_delegation() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"models":[]}"#))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(mock_server.uri()).unwrap();
        let result = client.list_models().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_show_model_delegation() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/show"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(r#"{"modelfile": "FROM test"}"#),
            )
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(mock_server.uri()).unwrap();
        let result = client.show_model("test-model").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_blob_exists_delegation() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:test";

        Mock::given(method("HEAD"))
            .and(path(format!("/api/blobs/{digest}")))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(mock_server.uri()).unwrap();
        let result = client.blob_exists(digest).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_create_blob_delegation() {
        let mock_server = MockServer::start().await;
        let digest = "sha256:test";
        let data = b"test data".to_vec();

        Mock::given(method("PUT"))
            .and(path(format!("/api/blobs/{digest}")))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(mock_server.uri()).unwrap();
        let result = client.create_blob(digest, data).await;

        assert!(result.is_ok());
    }
}
