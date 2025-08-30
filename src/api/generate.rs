//! Generate API implementation

use crate::{
    error::{OllamaError, Result},
    models::generation::{GenerateRequest, GenerateResponse},
    utils::http::HttpClient,
};
use futures_util::StreamExt;
use std::sync::Arc;

/// API implementation for text generation
pub struct GenerateApi;

impl GenerateApi {
    /// Generate text completion (non-streaming)
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the server returns an error.
    pub async fn generate(
        http_client: &Arc<HttpClient>,
        mut request: GenerateRequest,
    ) -> Result<GenerateResponse> {
        request.stream = Some(false);

        let response = http_client
            .post("api/generate")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }

        let generate_response: GenerateResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(generate_response)
    }

    /// Generate text completion with streaming
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the server returns an error.
    pub async fn generate_stream(
        http_client: &Arc<HttpClient>,
        mut request: GenerateRequest,
    ) -> Result<impl tokio_stream::Stream<Item = Result<GenerateResponse>>> {
        request.stream = Some(true);

        let response = http_client
            .post("api/generate")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OllamaError::ServerError {
                status: response.status().as_u16(),
                message: "Stream request failed".to_string(),
            });
        }

        let stream = response.bytes_stream().map(|chunk| match chunk {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    if !line.trim().is_empty() {
                        match serde_json::from_str::<GenerateResponse>(line) {
                            Ok(response) => return Ok(response),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_request_format() {
        let request = GenerateRequest::new("test-model", "test prompt")
            .stream(false)
            .system("test system");

        assert_eq!(request.model, "test-model");
        assert_eq!(request.prompt, "test prompt");
        assert_eq!(request.stream, Some(false));
        assert_eq!(request.system, Some("test system".to_string()));
    }
}
