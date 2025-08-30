//! Models API implementation

use crate::{
    error::{OllamaError, Result},
    models::model_info::{
        CopyRequest, CreateRequest, DeleteRequest, ModelInfo, ModelList, PullRequest,
        RunningModels, ShowRequest,
    },
    utils::http::HttpClient,
};
use futures_util::StreamExt;
use std::sync::Arc;

/// API implementation for model management
pub struct ModelsApi;

impl ModelsApi {
    /// List all available models
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the server returns an error.
    pub async fn list_models(http_client: &Arc<HttpClient>) -> Result<ModelList> {
        let response = http_client.get("api/tags").await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let model_list: ModelList = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(model_list)
    }

    /// Get information about a specific model
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the model is not found, or the server returns an error.
    pub async fn show_model(http_client: &Arc<HttpClient>, name: &str) -> Result<ModelInfo> {
        let request = ShowRequest {
            name: name.to_string(),
            verbose: Some(false),
        };

        let response = http_client.post("api/show").json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(match status {
                404 => OllamaError::ModelNotFound(name.to_string()),
                _ => OllamaError::ServerError {
                    status,
                    message: response.text().await.unwrap_or_default(),
                },
            });
        }

        let model_info: ModelInfo = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(model_info)
    }

    /// Pull a model from the registry
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the server returns an error.
    pub async fn pull_model(http_client: &Arc<HttpClient>, name: &str, stream: bool) -> Result<()> {
        let request = PullRequest {
            name: name.to_string(),
            stream: Some(stream),
            insecure: None,
        };

        let response = http_client.post("api/pull").json(&request).send().await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        if !stream {
            let _: serde_json::Value = response
                .json()
                .await
                .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;
        }

        Ok(())
    }

    /// Pull a model with streaming progress
    pub async fn pull_model_stream(
        http_client: &Arc<HttpClient>,
        name: &str,
    ) -> Result<impl tokio_stream::Stream<Item = Result<serde_json::Value>>> {
        let request = PullRequest {
            name: name.to_string(),
            stream: Some(true),
            insecure: None,
        };

        let response = http_client.post("api/pull").json(&request).send().await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: "Pull stream request failed".to_string(),
            });
        }

        let stream = response.bytes_stream().map(|chunk| match chunk {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    if !line.trim().is_empty() {
                        match serde_json::from_str::<serde_json::Value>(line) {
                            Ok(progress) => return Ok(progress),
                            Err(e) => return Err(OllamaError::InvalidResponse(e.to_string())),
                        }
                    }
                }
                Err(OllamaError::InvalidResponse("Empty chunk".to_string()))
            }
            Err(e) => Err(OllamaError::StreamError(e.to_string())),
        });

        Ok(stream)
    }

    /// Create a new model
    pub async fn create_model(
        http_client: &Arc<HttpClient>,
        name: &str,
        modelfile: &str,
        stream: bool,
    ) -> Result<()> {
        let request = CreateRequest {
            name: name.to_string(),
            modelfile: modelfile.to_string(),
            stream: Some(stream),
            quantize: None,
        };

        let response = http_client.post("api/create").json(&request).send().await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    /// Create a model with streaming progress
    pub async fn create_model_stream(
        http_client: &Arc<HttpClient>,
        name: &str,
        modelfile: &str,
    ) -> Result<impl tokio_stream::Stream<Item = Result<serde_json::Value>>> {
        let request = CreateRequest {
            name: name.to_string(),
            modelfile: modelfile.to_string(),
            stream: Some(true),
            quantize: None,
        };

        let response = http_client.post("api/create").json(&request).send().await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: "Create stream request failed".to_string(),
            });
        }

        let stream = response.bytes_stream().map(|chunk| match chunk {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    if !line.trim().is_empty() {
                        match serde_json::from_str::<serde_json::Value>(line) {
                            Ok(progress) => return Ok(progress),
                            Err(e) => return Err(OllamaError::InvalidResponse(e.to_string())),
                        }
                    }
                }
                Err(OllamaError::InvalidResponse("Empty chunk".to_string()))
            }
            Err(e) => Err(OllamaError::StreamError(e.to_string())),
        });

        Ok(stream)
    }

    /// Copy a model
    pub async fn copy_model(
        http_client: &Arc<HttpClient>,
        source: &str,
        destination: &str,
    ) -> Result<()> {
        let request = CopyRequest {
            source: source.to_string(),
            destination: destination.to_string(),
        };

        let response = http_client.post("api/copy").json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(match status {
                404 => OllamaError::ModelNotFound(source.to_string()),
                _ => OllamaError::ServerError {
                    status,
                    message: response.text().await.unwrap_or_default(),
                },
            });
        }

        Ok(())
    }

    /// Delete a model
    pub async fn delete_model(http_client: &Arc<HttpClient>, name: &str) -> Result<()> {
        let request = DeleteRequest {
            name: name.to_string(),
        };

        let response = http_client
            .delete("api/delete")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(match status {
                404 => OllamaError::ModelNotFound(name.to_string()),
                _ => OllamaError::ServerError {
                    status,
                    message: response.text().await.unwrap_or_default(),
                },
            });
        }

        Ok(())
    }

    /// List running models
    pub async fn list_running_models(http_client: &Arc<HttpClient>) -> Result<RunningModels> {
        let response = http_client.get("api/ps").await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let running_models: RunningModels = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(running_models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClientConfig;
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn test_pull_request_creation() {
        let request = PullRequest {
            name: "test-model".to_string(),
            stream: Some(true),
            insecure: None,
        };

        assert_eq!(request.name, "test-model");
        assert_eq!(request.stream, Some(true));
    }

    #[tokio::test]
    async fn test_list_models_success() {
        let mock_server = MockServer::start().await;

        let model_list_response = r#"{
            "models": [
                {
                    "name": "llama3:latest",
                    "model": "llama3:latest",
                    "modified_at": "2024-01-01T00:00:00Z",
                    "size": 4661100923,
                    "digest": "sha256:abcd1234",
                    "details": {
                        "parent_model": "",
                        "format": "gguf",
                        "family": "llama",
                        "families": null,
                        "parameter_size": "7B",
                        "quantization_level": "Q4_0"
                    }
                }
            ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_string(model_list_response))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::list_models(&http_client).await.unwrap();
        assert_eq!(result.models.len(), 1);
        assert_eq!(result.models[0].name, "llama3:latest");
    }

    #[tokio::test]
    async fn test_list_models_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::list_models(&http_client).await;
        assert!(result.is_err());

        if let Err(OllamaError::ServerError { status, message }) = result {
            assert_eq!(status, 500);
            assert_eq!(message, "Internal Server Error");
        } else {
            panic!("Expected ServerError");
        }
    }

    #[tokio::test]
    async fn test_list_models_invalid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::list_models(&http_client).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OllamaError::InvalidResponse(_)
        ));
    }

    #[tokio::test]
    async fn test_show_model_success() {
        let mock_server = MockServer::start().await;

        let model_info_response = r#"{
            "modelfile": "FROM llama3:latest",
            "parameters": "temperature 0.7",
            "template": "{{ .Prompt }}",
            "details": {
                "parent_model": "",
                "format": "gguf",
                "family": "llama",
                "families": null,
                "parameter_size": "7B",
                "quantization_level": "Q4_0"
            }
        }"#;

        let expected_request = ShowRequest {
            name: "llama3:latest".to_string(),
            verbose: Some(false),
        };

        Mock::given(method("POST"))
            .and(path("/api/show"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_string(model_info_response))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::show_model(&http_client, "llama3:latest")
            .await
            .unwrap();
        assert_eq!(result.modelfile, Some("FROM llama3:latest".to_string()));
        assert_eq!(result.parameters, Some("temperature 0.7".to_string()));
    }

    #[tokio::test]
    async fn test_show_model_not_found() {
        let mock_server = MockServer::start().await;

        let expected_request = ShowRequest {
            name: "nonexistent:model".to_string(),
            verbose: Some(false),
        };

        Mock::given(method("POST"))
            .and(path("/api/show"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(404).set_body_string("model not found"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::show_model(&http_client, "nonexistent:model").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OllamaError::ModelNotFound(_)));
    }

    #[tokio::test]
    async fn test_pull_model_success() {
        let mock_server = MockServer::start().await;

        let expected_request = PullRequest {
            name: "llama3:latest".to_string(),
            stream: Some(false),
            insecure: None,
        };

        Mock::given(method("POST"))
            .and(path("/api/pull"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::pull_model(&http_client, "llama3:latest", false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pull_model_stream() {
        let mock_server = MockServer::start().await;

        let expected_request = PullRequest {
            name: "llama3:latest".to_string(),
            stream: Some(true),
            insecure: None,
        };

        Mock::given(method("POST"))
            .and(path("/api/pull"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::pull_model(&http_client, "llama3:latest", true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pull_model_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/pull"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::pull_model(&http_client, "invalid-model", false).await;
        assert!(result.is_err());

        if let Err(OllamaError::ServerError { status, message }) = result {
            assert_eq!(status, 400);
            assert_eq!(message, "Bad Request");
        } else {
            panic!("Expected ServerError");
        }
    }

    #[tokio::test]
    async fn test_create_model_success() {
        let mock_server = MockServer::start().await;

        let expected_request = CreateRequest {
            name: "custom-model".to_string(),
            modelfile: "FROM llama3:latest\nTEMPERATURE 0.5".to_string(),
            stream: Some(false),
            quantize: None,
        };

        Mock::given(method("POST"))
            .and(path("/api/create"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::create_model(
            &http_client,
            "custom-model",
            "FROM llama3:latest\nTEMPERATURE 0.5",
            false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_copy_model_success() {
        let mock_server = MockServer::start().await;

        let expected_request = CopyRequest {
            source: "llama3:latest".to_string(),
            destination: "llama3:backup".to_string(),
        };

        Mock::given(method("POST"))
            .and(path("/api/copy"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::copy_model(&http_client, "llama3:latest", "llama3:backup").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_copy_model_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/copy"))
            .respond_with(ResponseTemplate::new(404).set_body_string("model not found"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::copy_model(&http_client, "nonexistent:model", "backup").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OllamaError::ModelNotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_model_success() {
        let mock_server = MockServer::start().await;

        let expected_request = DeleteRequest {
            name: "llama3:backup".to_string(),
        };

        Mock::given(method("DELETE"))
            .and(path("/api/delete"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::delete_model(&http_client, "llama3:backup").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_model_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/delete"))
            .respond_with(ResponseTemplate::new(404).set_body_string("model not found"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::delete_model(&http_client, "nonexistent:model").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OllamaError::ModelNotFound(_)));
    }

    #[tokio::test]
    async fn test_list_running_models_success() {
        let mock_server = MockServer::start().await;

        let running_models_response = r#"{
            "models": [
                {
                    "name": "llama3:latest",
                    "model": "llama3:latest",
                    "size": 4661100923,
                    "digest": "sha256:abcd1234",
                    "expires_at": "2024-01-01T01:00:00Z"
                }
            ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/api/ps"))
            .respond_with(ResponseTemplate::new(200).set_body_string(running_models_response))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::list_running_models(&http_client).await.unwrap();
        assert_eq!(result.models.len(), 1);
        assert_eq!(result.models[0].name, "llama3:latest");
    }

    #[tokio::test]
    async fn test_list_running_models_empty() {
        let mock_server = MockServer::start().await;

        let empty_response = r#"{"models": []}"#;

        Mock::given(method("GET"))
            .and(path("/api/ps"))
            .respond_with(ResponseTemplate::new(200).set_body_string(empty_response))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::list_running_models(&http_client).await.unwrap();
        assert_eq!(result.models.len(), 0);
    }

    #[tokio::test]
    async fn test_list_running_models_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ps"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: mock_server.uri().parse().unwrap(),
            ..ClientConfig::default()
        };
        let http_client = Arc::new(HttpClient::new(config).unwrap());

        let result = ModelsApi::list_running_models(&http_client).await;
        assert!(result.is_err());

        if let Err(OllamaError::ServerError { status, message }) = result {
            assert_eq!(status, 503);
            assert_eq!(message, "Service Unavailable");
        } else {
            panic!("Expected ServerError");
        }
    }

    #[test]
    fn test_create_request_creation() {
        let request = CreateRequest {
            name: "test-model".to_string(),
            modelfile: "FROM llama3:latest".to_string(),
            stream: Some(false),
            quantize: None,
        };

        assert_eq!(request.name, "test-model");
        assert_eq!(request.modelfile, "FROM llama3:latest");
        assert_eq!(request.stream, Some(false));
    }

    #[test]
    fn test_copy_request_creation() {
        let request = CopyRequest {
            source: "source-model".to_string(),
            destination: "dest-model".to_string(),
        };

        assert_eq!(request.source, "source-model");
        assert_eq!(request.destination, "dest-model");
    }

    #[test]
    fn test_delete_request_creation() {
        let request = DeleteRequest {
            name: "model-to-delete".to_string(),
        };

        assert_eq!(request.name, "model-to-delete");
    }

    #[test]
    fn test_show_request_creation() {
        let request = ShowRequest {
            name: "model-to-show".to_string(),
            verbose: Some(true),
        };

        assert_eq!(request.name, "model-to-show");
        assert_eq!(request.verbose, Some(true));
    }
}
