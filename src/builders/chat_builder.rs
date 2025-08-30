//! Builder for chat requests

use crate::{
    api::chat::ChatApi,
    error::Result,
    models::{
        chat::{ChatMessage, ChatRequest, ChatResponse, ToolChoice},
        common::{KeepAlive, Options, ResponseFormat, Tool},
    },
    streaming::stream::ChatStream,
    utils::http::HttpClient,
};
use std::sync::Arc;

/// Builder for chat requests
#[derive(Debug, Clone)]
pub struct ChatBuilder {
    http_client: Arc<HttpClient>,
    request: ChatRequest,
}

impl ChatBuilder {
    /// Create a new chat builder
    #[must_use]
    pub fn new(http_client: Arc<HttpClient>) -> Self {
        Self {
            http_client,
            request: ChatRequest::default(),
        }
    }

    /// Set the model to use
    #[must_use]
    pub fn model<S: Into<String>>(mut self, model: S) -> Self {
        self.request.model = model.into();
        self
    }

    /// Add a message to the conversation
    pub fn add_message(mut self, message: ChatMessage) -> Self {
        self.request.messages.push(message);
        self
    }

    /// Add a system message
    pub fn add_system_message<S: Into<String>>(mut self, content: S) -> Self {
        self.request.messages.push(ChatMessage::system(content));
        self
    }

    /// Add a user message
    pub fn add_user_message<S: Into<String>>(mut self, content: S) -> Self {
        self.request.messages.push(ChatMessage::user(content));
        self
    }

    /// Add an assistant message
    pub fn add_assistant_message<S: Into<String>>(mut self, content: S) -> Self {
        self.request.messages.push(ChatMessage::assistant(content));
        self
    }

    /// Add a user message with images
    pub fn add_user_message_with_images<S: Into<String>>(
        mut self,
        content: S,
        images: Vec<String>,
    ) -> Self {
        let message = ChatMessage::user(content).with_images(images);
        self.request.messages.push(message);
        self
    }

    /// Set all messages at once
    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.request.messages = messages;
        self
    }

    /// Set generation options
    pub fn options(mut self, options: Options) -> Self {
        self.request.options = Some(options);
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temperature: f64) -> Self {
        let mut options = self.request.options.unwrap_or_default();
        options.temperature = Some(temperature);
        self.request.options = Some(options);
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        let mut options = self.request.options.unwrap_or_default();
        // Use saturating conversion to avoid wrap-around
        options.num_predict = Some(max_tokens.try_into().unwrap_or(i32::MAX));
        self.request.options = Some(options);
        self
    }

    /// Set top-k
    pub fn top_k(mut self, top_k: i32) -> Self {
        let mut options = self.request.options.unwrap_or_default();
        options.top_k = Some(top_k);
        self.request.options = Some(options);
        self
    }

    /// Set top-p
    pub fn top_p(mut self, top_p: f64) -> Self {
        let mut options = self.request.options.unwrap_or_default();
        options.top_p = Some(top_p);
        self.request.options = Some(options);
        self
    }

    /// Set response format
    pub fn format(mut self, format: ResponseFormat) -> Self {
        self.request.format = Some(format);
        self
    }

    /// Set keep alive
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.request.keep_alive = Some(keep_alive);
        self
    }

    /// Set available tools
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.request.tools = Some(tools);
        self
    }

    /// Set tool choice strategy
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.request.tool_choice = Some(choice);
        self
    }

    /// Send the request (non-streaming)
    /// 
    /// # Errors
    /// Returns an error if the request fails due to network issues, authentication problems, 
    /// or invalid parameters.
    pub async fn send(self) -> Result<ChatResponse> {
        ChatApi::chat(&self.http_client, self.request).await
    }

    /// Send the request with streaming
    /// 
    /// # Errors
    /// Returns an error if the request fails due to network issues, authentication problems, 
    /// or invalid parameters.
    pub async fn stream(self) -> Result<ChatStream> {
        let stream = ChatApi::chat_stream(&self.http_client, self.request).await?;
        Ok(ChatStream::new(Box::pin(stream)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::chat::FunctionChoice;
    use serde_json::json;

    #[test]
    fn test_chat_builder() {
        let config = crate::config::ClientConfig::default();
        let http_client = Arc::new(crate::utils::http::HttpClient::new(config).unwrap());

        let builder = ChatBuilder::new(http_client)
            .model("test-model")
            .add_system_message("You are helpful")
            .add_user_message("Hello")
            .temperature(0.7);

        assert_eq!(builder.request.model, "test-model");
        assert_eq!(builder.request.messages.len(), 2);
        assert_eq!(
            builder.request.messages[0].role,
            crate::models::chat::MessageRole::System
        );
        assert_eq!(
            builder.request.messages[1].role,
            crate::models::chat::MessageRole::User
        );

        let options = builder.request.options.unwrap();
        assert_eq!(options.temperature, Some(0.7));
    }

    #[test]
    fn test_chat_builder_with_tools() {
        let config = crate::config::ClientConfig::default();
        let http_client = Arc::new(crate::utils::http::HttpClient::new(config).unwrap());

        let tool1 = Tool::function(
            "get_weather".to_string(),
            "Get weather".to_string(),
            json!({"type": "object"}),
        );

        let tool2 = Tool::function(
            "calculate".to_string(),
            "Calculate".to_string(),
            json!({"type": "object"}),
        );

        let builder = ChatBuilder::new(http_client)
            .model("test-model")
            .add_user_message("What's the weather?")
            .tools(vec![tool1, tool2])
            .tool_choice(ToolChoice::Auto("auto".to_string()));

        assert_eq!(builder.request.model, "test-model");
        assert_eq!(builder.request.messages.len(), 1);

        let tools = builder.request.tools.unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].function.name, "get_weather");
        assert_eq!(tools[1].function.name, "calculate");

        assert!(builder.request.tool_choice.is_some());
    }

    #[test]
    fn test_chat_builder_tool_choice_variants() {
        let config = crate::config::ClientConfig::default();
        let http_client = Arc::new(crate::utils::http::HttpClient::new(config).unwrap());

        // Test auto choice
        let builder_auto = ChatBuilder::new(http_client.clone())
            .model("test")
            .tool_choice(ToolChoice::Auto("auto".to_string()));

        match builder_auto.request.tool_choice {
            Some(ToolChoice::Auto(s)) => assert_eq!(s, "auto"),
            _ => panic!("Expected Auto variant"),
        }

        // Test none choice
        let builder_none = ChatBuilder::new(http_client.clone())
            .model("test")
            .tool_choice(ToolChoice::None("none".to_string()));

        match builder_none.request.tool_choice {
            Some(ToolChoice::None(s)) => assert_eq!(s, "none"),
            _ => panic!("Expected None variant"),
        }

        // Test specific tool choice
        let builder_specific =
            ChatBuilder::new(http_client)
                .model("test")
                .tool_choice(ToolChoice::Specific {
                    tool_type: "function".to_string(),
                    function: FunctionChoice {
                        name: "my_function".to_string(),
                    },
                });

        match builder_specific.request.tool_choice {
            Some(ToolChoice::Specific {
                tool_type,
                function,
            }) => {
                assert_eq!(tool_type, "function");
                assert_eq!(function.name, "my_function");
            }
            _ => panic!("Expected Specific variant"),
        }
    }
}
