//! Chat API request and response models

use crate::models::common::{KeepAlive, Options, ResponseFormat, Tool, ToolCall};
use serde::{Deserialize, Serialize};

/// Role of a message in a chat conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message that provides context
    System,
    /// User message
    User,
    /// Assistant/AI response
    Assistant,
    /// Tool/function call result
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::Tool => write!(f, "tool"),
        }
    }
}

/// A message in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender
    pub role: MessageRole,

    /// Content of the message
    pub content: String,

    /// Images associated with the message (for multimodal models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,

    /// Tool calls made by the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Tool call ID (for tool response messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// Create a new system message
    pub fn system<S: Into<String>>(content: S) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            images: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a new user message
    pub fn user<S: Into<String>>(content: S) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            images: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant<S: Into<String>>(content: S) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            images: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a new tool message
    pub fn tool<S: Into<String>>(content: S, tool_call_id: S) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            images: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }

    /// Add images to the message
    pub fn with_images(mut self, images: Vec<String>) -> Self {
        self.images = Some(images);
        self
    }

    /// Add tool calls to the message
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }
}

/// Request for chat completion
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatRequest {
    /// Model to use for chat
    pub model: String,

    /// List of messages in the conversation
    pub messages: Vec<ChatMessage>,

    /// Enable or disable streaming
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Additional generation options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,

    /// Response format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ResponseFormat>,

    /// How long to keep the model loaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<KeepAlive>,

    /// Available tools/functions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Tool choice strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

/// Tool choice strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Automatically choose when to use tools
    Auto(String), // "auto"
    /// Never use tools
    None(String), // "none"
    /// Always use tools
    Required(String), // "required"
    /// Use a specific tool
    Specific {
        #[serde(rename = "type")]
        tool_type: String,
        function: FunctionChoice,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    pub name: String,
}

impl ChatRequest {
    /// Create a new chat request
    pub fn new<S: Into<String>>(model: S) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Add a message to the conversation
    pub fn add_message(mut self, message: ChatMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Add a system message
    pub fn add_system_message<S: Into<String>>(mut self, content: S) -> Self {
        self.messages.push(ChatMessage::system(content));
        self
    }

    /// Add a user message
    pub fn add_user_message<S: Into<String>>(mut self, content: S) -> Self {
        self.messages.push(ChatMessage::user(content));
        self
    }

    /// Add an assistant message
    pub fn add_assistant_message<S: Into<String>>(mut self, content: S) -> Self {
        self.messages.push(ChatMessage::assistant(content));
        self
    }

    /// Set whether to stream the response
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Set generation options
    pub fn options(mut self, options: Options) -> Self {
        self.options = Some(options);
        self
    }

    /// Set available tools
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set tool choice strategy
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }
}

/// Response from chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The model that was used
    pub model: String,

    /// The generated message
    pub message: ChatMessage,

    /// Whether this is the final response
    pub done: bool,

    /// Total duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,

    /// Load duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,

    /// Prompt evaluation count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,

    /// Prompt evaluation duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,

    /// Evaluation count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u32>,

    /// Evaluation duration in nanoseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

impl ChatResponse {
    /// Get the assistant's response content
    pub fn content(&self) -> &str {
        &self.message.content
    }

    /// Check if the message has tool calls
    pub fn has_tool_calls(&self) -> bool {
        self.message.tool_calls.is_some()
    }

    /// Get tool calls if any
    pub fn tool_calls(&self) -> Option<&Vec<ToolCall>> {
        self.message.tool_calls.as_ref()
    }

    /// Get tokens per second for evaluation
    pub fn eval_rate(&self) -> Option<f64> {
        match (self.eval_count, self.eval_duration) {
            (Some(count), Some(duration)) if duration > 0 => {
                Some(count as f64 / (duration as f64 / 1e9))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let system_msg = ChatMessage::system("You are a helpful assistant");
        assert_eq!(system_msg.role, MessageRole::System);
        assert_eq!(system_msg.content, "You are a helpful assistant");

        let user_msg = ChatMessage::user("Hello");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert_eq!(assistant_msg.content, "Hi there!");
    }

    #[test]
    fn test_chat_request_builder() {
        let request = ChatRequest::new("test-model")
            .add_system_message("You are helpful")
            .add_user_message("Hello")
            .stream(true);

        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[1].role, MessageRole::User);
        assert_eq!(request.stream, Some(true));
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::System.to_string(), "system");
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::Tool.to_string(), "tool");
    }
}
