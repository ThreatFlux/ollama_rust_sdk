//! Embeddings API implementation

use crate::{
    error::{OllamaError, Result},
    models::embedding::{
        EmbedRequest, EmbedResponse, LegacyEmbeddingRequest, LegacyEmbeddingResponse,
    },
    utils::http::HttpClient,
};
use std::sync::Arc;

/// API implementation for embeddings
pub struct EmbeddingsApi;

impl EmbeddingsApi {
    /// Generate embeddings using the new API
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the model is not found, or the server returns an error.
    pub async fn embed(
        http_client: &Arc<HttpClient>,
        request: EmbedRequest,
    ) -> Result<EmbedResponse> {
        let response = http_client.post("api/embed").json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();

            return Err(match status {
                404 => OllamaError::ModelNotFound(request.model),
                _ => OllamaError::ServerError { status, message },
            });
        }

        let embed_response: EmbedResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(embed_response)
    }

    /// Generate embeddings using the legacy API (deprecated)
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the model is not found, or the server returns an error.
    pub async fn embed_legacy(
        http_client: &Arc<HttpClient>,
        request: LegacyEmbeddingRequest,
    ) -> Result<LegacyEmbeddingResponse> {
        let response = http_client
            .post("api/embeddings")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();

            return Err(match status {
                404 => OllamaError::ModelNotFound(request.model),
                _ => OllamaError::ServerError { status, message },
            });
        }

        let embed_response: LegacyEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(embed_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_request_creation() {
        let request = EmbedRequest::new("test-model", "test text");
        assert_eq!(request.model, "test-model");
        assert_eq!(request.input_count(), 1);
    }

    #[test]
    fn test_embed_request_multiple_inputs() {
        let inputs = vec!["text1", "text2", "text3"];
        let request = EmbedRequest::new("test-model", inputs.clone());
        assert_eq!(request.model, "test-model");
        assert_eq!(request.input_count(), 3);
        assert_eq!(request.inputs_as_vec(), vec!["text1", "text2", "text3"]);
    }
}
