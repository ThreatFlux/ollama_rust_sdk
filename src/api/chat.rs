//! Chat API implementation

use crate::{
    error::{OllamaError, Result},
    models::chat::{ChatRequest, ChatResponse},
    utils::http::HttpClient,
};
use futures_util::StreamExt;
use std::sync::Arc;

/// API implementation for chat completions
pub struct ChatApi;

impl ChatApi {
    /// Send a chat completion request (non-streaming)
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the model is not found, or the server returns an error.
    pub async fn chat(
        http_client: &Arc<HttpClient>,
        mut request: ChatRequest,
    ) -> Result<ChatResponse> {
        request.stream = Some(false);

        let response = http_client.post("api/chat").json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();

            return Err(match status {
                404 => OllamaError::ModelNotFound(request.model),
                _ => OllamaError::ServerError { status, message },
            });
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::InvalidResponse(e.to_string()))?;

        Ok(chat_response)
    }

    /// Send a chat completion request with streaming
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, the model is not found, or the server returns an error.
    pub async fn chat_stream(
        http_client: &Arc<HttpClient>,
        mut request: ChatRequest,
    ) -> Result<impl tokio_stream::Stream<Item = Result<ChatResponse>>> {
        request.stream = Some(true);

        let response = http_client.post("api/chat").json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(match status {
                404 => OllamaError::ModelNotFound(request.model),
                _ => OllamaError::ServerError {
                    status,
                    message: "Stream request failed".to_string(),
                },
            });
        }

        let stream = response.bytes_stream().map(|chunk| match chunk {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        match serde_json::from_str::<ChatResponse>(line) {
                            Ok(response) => return Ok(response),
                            Err(e) => {
                                return Err(OllamaError::InvalidResponse(format!(
                                    "Failed to parse chunk: {e} - Line: {line}"
                                )))
                            }
                        }
                    }
                }
                Err(OllamaError::InvalidResponse(
                    "Empty or invalid chunk".to_string(),
                ))
            }
            Err(e) => Err(OllamaError::StreamError(e.to_string())),
        });

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::chat::MessageRole;

    #[test]
    fn test_chat_request_creation() {
        let request = ChatRequest::new("test-model")
            .add_system_message("You are helpful")
            .add_user_message("Hello")
            .stream(false);

        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[1].role, MessageRole::User);
        assert_eq!(request.stream, Some(false));
    }
}
