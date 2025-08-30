//! Streaming response types

use crate::{
    error::Result,
    models::{chat::ChatResponse, generation::GenerateResponse},
};
use futures_util::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::StreamExt;

/// Stream chunk that can contain either data or an error
pub type StreamChunk<T> = Result<T>;

/// Generate response stream
pub struct GenerateStream {
    inner: Pin<Box<dyn Stream<Item = StreamChunk<GenerateResponse>> + Send>>,
}

impl GenerateStream {
    /// Create a new generate stream
    pub fn new(stream: Pin<Box<dyn Stream<Item = StreamChunk<GenerateResponse>> + Send>>) -> Self {
        Self { inner: stream }
    }

    /// Collect all responses into a single response
    pub async fn collect_response(mut self) -> Result<GenerateResponse> {
        let mut final_response = None;
        let mut full_text = String::new();

        while let Some(chunk) = self.next().await {
            let response = chunk?;
            full_text.push_str(&response.response);

            if response.done {
                final_response = Some(GenerateResponse {
                    model: response.model,
                    response: full_text,
                    done: true,
                    context: response.context,
                    total_duration: response.total_duration,
                    load_duration: response.load_duration,
                    prompt_eval_count: response.prompt_eval_count,
                    prompt_eval_duration: response.prompt_eval_duration,
                    eval_count: response.eval_count,
                    eval_duration: response.eval_duration,
                });
                break;
            }
        }

        final_response.ok_or_else(|| {
            crate::error::OllamaError::StreamError(
                "Stream ended without final response".to_string(),
            )
        })
    }
}

impl Stream for GenerateStream {
    type Item = StreamChunk<GenerateResponse>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// Chat response stream
pub struct ChatStream {
    inner: Pin<Box<dyn Stream<Item = StreamChunk<ChatResponse>> + Send>>,
}

impl ChatStream {
    /// Create a new chat stream
    pub fn new(stream: Pin<Box<dyn Stream<Item = StreamChunk<ChatResponse>> + Send>>) -> Self {
        Self { inner: stream }
    }

    /// Collect all responses into a single response
    pub async fn collect_response(mut self) -> Result<ChatResponse> {
        let mut final_response = None;
        let mut full_content = String::new();

        while let Some(chunk) = self.next().await {
            let response = chunk?;
            full_content.push_str(&response.message.content);

            if response.done {
                final_response = Some(ChatResponse {
                    model: response.model,
                    message: crate::models::chat::ChatMessage {
                        role: response.message.role,
                        content: full_content,
                        images: response.message.images,
                        tool_calls: response.message.tool_calls,
                        tool_call_id: response.message.tool_call_id,
                    },
                    done: true,
                    total_duration: response.total_duration,
                    load_duration: response.load_duration,
                    prompt_eval_count: response.prompt_eval_count,
                    prompt_eval_duration: response.prompt_eval_duration,
                    eval_count: response.eval_count,
                    eval_duration: response.eval_duration,
                });
                break;
            }
        }

        final_response.ok_or_else(|| {
            crate::error::OllamaError::StreamError(
                "Stream ended without final response".to_string(),
            )
        })
    }
}

impl Stream for ChatStream {
    type Item = StreamChunk<ChatResponse>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::OllamaError;
    use crate::models::{chat::*, generation::GenerateResponse};
    use futures_util::stream;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_generate_stream_creation() {
        let mock_stream = stream::empty::<StreamChunk<GenerateResponse>>();
        let generate_stream = GenerateStream::new(Box::pin(mock_stream));

        // Just test that we can create the stream without errors
        drop(generate_stream);
    }

    #[tokio::test]
    async fn test_chat_stream_creation() {
        let mock_stream = stream::empty::<StreamChunk<ChatResponse>>();
        let chat_stream = ChatStream::new(Box::pin(mock_stream));

        // Just test that we can create the stream without errors
        drop(chat_stream);
    }

    #[tokio::test]
    async fn test_generate_stream_collect_single_response() {
        let response = GenerateResponse {
            model: "test-model".to_string(),
            response: "Hello world".to_string(),
            done: true,
            context: Some(vec![1, 2, 3]),
            total_duration: Some(1000),
            load_duration: Some(100),
            prompt_eval_count: Some(5),
            prompt_eval_duration: Some(200),
            eval_count: Some(10),
            eval_duration: Some(300),
        };

        let mock_stream = stream::iter(vec![Ok(response.clone())]);
        let generate_stream = GenerateStream::new(Box::pin(mock_stream));

        let collected = generate_stream.collect_response().await.unwrap();

        assert_eq!(collected.model, "test-model");
        assert_eq!(collected.response, "Hello world");
        assert!(collected.done);
        assert_eq!(collected.context, Some(vec![1, 2, 3]));
        assert_eq!(collected.total_duration, Some(1000));
    }

    #[tokio::test]
    async fn test_generate_stream_collect_multiple_chunks() {
        let chunk1 = GenerateResponse {
            model: "test-model".to_string(),
            response: "Hello".to_string(),
            done: false,
            context: None,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        };

        let chunk2 = GenerateResponse {
            model: "test-model".to_string(),
            response: " world".to_string(),
            done: false,
            context: None,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        };

        let final_chunk = GenerateResponse {
            model: "test-model".to_string(),
            response: "!".to_string(),
            done: true,
            context: Some(vec![1, 2, 3]),
            total_duration: Some(1000),
            load_duration: Some(100),
            prompt_eval_count: Some(5),
            prompt_eval_duration: Some(200),
            eval_count: Some(15),
            eval_duration: Some(800),
        };

        let mock_stream = stream::iter(vec![Ok(chunk1), Ok(chunk2), Ok(final_chunk)]);
        let generate_stream = GenerateStream::new(Box::pin(mock_stream));

        let collected = generate_stream.collect_response().await.unwrap();

        assert_eq!(collected.model, "test-model");
        assert_eq!(collected.response, "Hello world!");
        assert!(collected.done);
        assert_eq!(collected.context, Some(vec![1, 2, 3]));
        assert_eq!(collected.total_duration, Some(1000));
        assert_eq!(collected.eval_count, Some(15));
    }

    #[tokio::test]
    async fn test_generate_stream_collect_with_error() {
        let chunk = GenerateResponse {
            model: "test-model".to_string(),
            response: "Hello".to_string(),
            done: false,
            context: None,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        };

        let error = OllamaError::StreamError("Connection lost".to_string());

        let mock_stream = stream::iter(vec![Ok(chunk), Err(error)]);
        let generate_stream = GenerateStream::new(Box::pin(mock_stream));

        let result = generate_stream.collect_response().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OllamaError::StreamError(_)));
    }

    #[tokio::test]
    async fn test_generate_stream_collect_empty_stream() {
        let mock_stream = stream::empty::<StreamChunk<GenerateResponse>>();
        let generate_stream = GenerateStream::new(Box::pin(mock_stream));

        let result = generate_stream.collect_response().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OllamaError::StreamError(_)));
    }

    #[tokio::test]
    async fn test_chat_stream_collect_single_response() {
        let response = ChatResponse {
            model: "test-model".to_string(),
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: "Hello world".to_string(),
                images: None,
                tool_calls: None,
                tool_call_id: None,
            },
            done: true,
            total_duration: Some(1000),
            load_duration: Some(100),
            prompt_eval_count: Some(5),
            prompt_eval_duration: Some(200),
            eval_count: Some(10),
            eval_duration: Some(300),
        };

        let mock_stream = stream::iter(vec![Ok(response.clone())]);
        let chat_stream = ChatStream::new(Box::pin(mock_stream));

        let collected = chat_stream.collect_response().await.unwrap();

        assert_eq!(collected.model, "test-model");
        assert_eq!(collected.message.content, "Hello world");
        assert!(matches!(collected.message.role, MessageRole::Assistant));
        assert!(collected.done);
    }

    #[tokio::test]
    async fn test_chat_stream_collect_multiple_chunks() {
        let chunk1 = ChatResponse {
            model: "test-model".to_string(),
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: "Hello".to_string(),
                images: None,
                tool_calls: None,
                tool_call_id: None,
            },
            done: false,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        };

        let chunk2 = ChatResponse {
            model: "test-model".to_string(),
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: " world".to_string(),
                images: Some(vec!["image1".to_string()]),
                tool_calls: None,
                tool_call_id: None,
            },
            done: false,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        };

        let final_chunk = ChatResponse {
            model: "test-model".to_string(),
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: "!".to_string(),
                images: Some(vec!["image2".to_string()]),
                tool_calls: None,
                tool_call_id: Some("call123".to_string()),
            },
            done: true,
            total_duration: Some(1500),
            load_duration: Some(150),
            prompt_eval_count: Some(8),
            prompt_eval_duration: Some(300),
            eval_count: Some(20),
            eval_duration: Some(1000),
        };

        let mock_stream = stream::iter(vec![Ok(chunk1), Ok(chunk2), Ok(final_chunk.clone())]);
        let chat_stream = ChatStream::new(Box::pin(mock_stream));

        let collected = chat_stream.collect_response().await.unwrap();

        assert_eq!(collected.model, "test-model");
        assert_eq!(collected.message.content, "Hello world!");
        assert_eq!(collected.message.images, Some(vec!["image2".to_string()]));
        assert_eq!(collected.message.tool_call_id, Some("call123".to_string()));
        assert!(collected.done);
        assert_eq!(collected.total_duration, Some(1500));
    }

    #[tokio::test]
    async fn test_chat_stream_collect_with_error() {
        let chunk = ChatResponse {
            model: "test-model".to_string(),
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: "Hello".to_string(),
                images: None,
                tool_calls: None,
                tool_call_id: None,
            },
            done: false,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        };

        let error = OllamaError::StreamError("Connection lost".to_string());

        let mock_stream = stream::iter(vec![Ok(chunk), Err(error)]);
        let chat_stream = ChatStream::new(Box::pin(mock_stream));

        let result = chat_stream.collect_response().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OllamaError::StreamError(_)));
    }

    #[tokio::test]
    async fn test_chat_stream_collect_empty_stream() {
        let mock_stream = stream::empty::<StreamChunk<ChatResponse>>();
        let chat_stream = ChatStream::new(Box::pin(mock_stream));

        let result = chat_stream.collect_response().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OllamaError::StreamError(_)));
    }

    #[tokio::test]
    async fn test_generate_stream_as_stream_trait() {
        let response1 = GenerateResponse {
            model: "test-model".to_string(),
            response: "chunk1".to_string(),
            done: false,
            context: None,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        };

        let response2 = GenerateResponse {
            model: "test-model".to_string(),
            response: "chunk2".to_string(),
            done: true,
            context: Some(vec![1, 2, 3]),
            total_duration: Some(1000),
            load_duration: Some(100),
            prompt_eval_count: Some(5),
            prompt_eval_duration: Some(200),
            eval_count: Some(10),
            eval_duration: Some(300),
        };

        let mock_stream = stream::iter(vec![Ok(response1), Ok(response2)]);
        let mut generate_stream = GenerateStream::new(Box::pin(mock_stream));

        let first_item = generate_stream.next().await.unwrap().unwrap();
        assert_eq!(first_item.response, "chunk1");
        assert!(!first_item.done);

        let second_item = generate_stream.next().await.unwrap().unwrap();
        assert_eq!(second_item.response, "chunk2");
        assert!(second_item.done);

        let third_item = generate_stream.next().await;
        assert!(third_item.is_none());
    }

    #[tokio::test]
    async fn test_chat_stream_as_stream_trait() {
        let response1 = ChatResponse {
            model: "test-model".to_string(),
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: "chunk1".to_string(),
                images: None,
                tool_calls: None,
                tool_call_id: None,
            },
            done: false,
            total_duration: None,
            load_duration: None,
            prompt_eval_count: None,
            prompt_eval_duration: None,
            eval_count: None,
            eval_duration: None,
        };

        let response2 = ChatResponse {
            model: "test-model".to_string(),
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: "chunk2".to_string(),
                images: None,
                tool_calls: None,
                tool_call_id: None,
            },
            done: true,
            total_duration: Some(1500),
            load_duration: Some(150),
            prompt_eval_count: Some(8),
            prompt_eval_duration: Some(300),
            eval_count: Some(20),
            eval_duration: Some(1000),
        };

        let mock_stream = stream::iter(vec![Ok(response1), Ok(response2)]);
        let mut chat_stream = ChatStream::new(Box::pin(mock_stream));

        let first_item = chat_stream.next().await.unwrap().unwrap();
        assert_eq!(first_item.message.content, "chunk1");
        assert!(!first_item.done);

        let second_item = chat_stream.next().await.unwrap().unwrap();
        assert_eq!(second_item.message.content, "chunk2");
        assert!(second_item.done);

        let third_item = chat_stream.next().await;
        assert!(third_item.is_none());
    }
}
