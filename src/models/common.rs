//! Common types shared across different API models

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Generation options that can be applied to various requests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Options {
    /// Number of tokens to predict
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,

    /// Sets the random number seed to use for generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,

    /// The temperature of the model. Increasing the temperature will make the model answer more creatively. (0.1 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Sets the size of the context window used to generate the next token (default: 2048)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<i32>,

    /// Works together with top_p. A higher value (e.g. 100) will give more diverse answers, whereas a lower value (e.g. 10) will be more conservative. (1 to 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,

    /// Works together with top_k. A higher value (e.g. 0.95) will lead to more diverse text, while a lower value (e.g. 0.5) will generate more focused and conservative text. (0.1 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    /// Tail free sampling is used to reduce the impact of less probable tokens from the output. (0.1 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tfs_z: Option<f64>,

    /// Typical P is used to reduce the impact of less probable tokens from the output. (0.1 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typical_p: Option<f64>,

    /// Sets how far back for the model to look back to prevent repetition. (0 = disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_last_n: Option<i32>,

    /// Sets how strongly to penalize repetitions. (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f64>,

    /// Positive values penalize new tokens based on whether they appear in the text so far
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    /// Positive values penalize new tokens based on their existing frequency in the text so far
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    /// Enable Mirostat sampling for controlling perplexity. (0 = disabled, 1 = Mirostat, 2 = Mirostat 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirostat: Option<i32>,

    /// Controls the balance between coherence and diversity of the output. (1.0 to 10.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirostat_tau: Option<f64>,

    /// Influences how quickly the algorithm responds to feedback from the generated text. (0.1 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirostat_eta: Option<f64>,

    /// Penalize newlines in the output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub penalize_newline: Option<bool>,

    /// Sequences where the API will stop generating further tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Enable or disable the use of a GPU for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numa: Option<bool>,

    /// Sets the number of threads to use during computation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_thread: Option<i32>,

    /// Sets the number of tokens to keep from the initial prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_keep: Option<i32>,

    /// Sets the batch size for prompt processing (default 512)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_batch: Option<i32>,

    /// The number of GPUs to use for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_gpu: Option<i32>,

    /// The main GPU to use for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_gpu: Option<i32>,

    /// Enable low VRAM mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_vram: Option<bool>,

    /// Enable F16 key-value cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f16_kv: Option<bool>,

    /// Return logits for all tokens in the vocabulary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logits_all: Option<bool>,

    /// Load only the vocabulary, not the weights
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocab_only: Option<bool>,

    /// Use memory mapping for file access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_mmap: Option<bool>,

    /// Use memory locking to keep the model in RAM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_mlock: Option<bool>,
}

impl Options {
    /// Create a new Options struct with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set temperature (0.1 to 2.0)
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set top_k (1 to 100)
    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set top_p (0.1 to 1.0)
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set the number of tokens to predict
    pub fn num_predict(mut self, num_predict: i32) -> Self {
        self.num_predict = Some(num_predict);
        self
    }

    /// Set the context window size
    pub fn num_ctx(mut self, num_ctx: i32) -> Self {
        self.num_ctx = Some(num_ctx);
        self
    }

    /// Set the random seed
    pub fn seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set stop sequences
    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }
}

/// Tool function definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    /// Function name
    pub name: String,

    /// Function description
    pub description: String,

    /// Function parameters schema
    pub parameters: serde_json::Value,
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool type (currently only "function" is supported)
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Function definition
    pub function: ToolFunction,
}

impl Tool {
    /// Create a new function tool
    pub fn function(name: String, description: String, parameters: serde_json::Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name,
                description,
                parameters,
            },
        }
    }
}

/// Tool call made by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID (optional - some models don't provide this)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Tool type (optional - defaults to "function")
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,

    /// Function call details
    pub function: FunctionCall,
}

/// Function call details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,

    /// Function arguments - can be either a JSON string or object
    #[serde(with = "arguments_serde")]
    pub arguments: Value,
}

/// Custom serialization for arguments field that can be string or object
mod arguments_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_json::Value;

    pub fn serialize<S>(arguments: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        arguments.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(text) => {
                if text.trim().is_empty() {
                    Ok(Value::String(text))
                } else {
                    match serde_json::from_str(&text) {
                        Ok(parsed) => Ok(parsed),
                        Err(_) => Ok(Value::String(text)),
                    }
                }
            }
            other => Ok(other),
        }
    }
}

/// Usage statistics for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,

    /// Number of tokens in the completion
    pub completion_tokens: u32,

    /// Total number of tokens
    pub total_tokens: u32,
}

/// Format types for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// Default text format
    Text,
    /// JSON format
    Json,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self::Text
    }
}

/// Keep alive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeepAlive {
    /// Keep alive duration as string (e.g., "5m", "1h")
    Duration(String),
    /// Keep alive as seconds
    Seconds(u64),
    /// Disable keep alive
    Never,
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self::Duration("5m".to_string())
    }
}

impl From<&str> for KeepAlive {
    fn from(s: &str) -> Self {
        Self::Duration(s.to_string())
    }
}

impl From<u64> for KeepAlive {
    fn from(seconds: u64) -> Self {
        Self::Seconds(seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::chat::ToolChoice;

    #[test]
    fn test_options_builder() {
        let options = Options::new()
            .temperature(0.7)
            .top_k(40)
            .top_p(0.9)
            .num_predict(100);

        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.top_k, Some(40));
        assert_eq!(options.top_p, Some(0.9));
        assert_eq!(options.num_predict, Some(100));
    }

    #[test]
    fn test_tool_creation() {
        let tool = Tool::function(
            "get_weather".to_string(),
            "Get weather for a location".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            }),
        );

        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.function.name, "get_weather");
    }

    #[test]
    fn test_keep_alive_variants() {
        let duration = KeepAlive::from("10m");
        let seconds = KeepAlive::from(300u64);

        match duration {
            KeepAlive::Duration(d) => assert_eq!(d, "10m"),
            _ => panic!("Expected Duration variant"),
        }

        match seconds {
            KeepAlive::Seconds(s) => assert_eq!(s, 300),
            _ => panic!("Expected Seconds variant"),
        }
    }

    #[test]
    fn test_function_call_arguments_deserialization_string() {
        // Test deserialization from JSON string
        let json = r#"{"name": "test_function", "arguments": "{\"param\": \"value\"}"}"#;
        let function_call: FunctionCall = serde_json::from_str(json).unwrap();

        assert_eq!(function_call.name, "test_function");
        assert_eq!(function_call.arguments["param"], "value");
    }

    #[test]
    fn test_function_call_arguments_deserialization_object() {
        // Test deserialization from JSON object
        let json = r#"{"name": "test_function", "arguments": {"param": "value", "number": 42}}"#;
        let function_call: FunctionCall = serde_json::from_str(json).unwrap();

        assert_eq!(function_call.name, "test_function");
        assert_eq!(function_call.arguments["param"], "value");
        assert_eq!(function_call.arguments["number"], 42);
    }

    #[test]
    fn test_function_call_arguments_deserialization_array() {
        // Test deserialization from JSON array
        let json = r#"{"name": "test_function", "arguments": ["arg1", "arg2"]}"#;
        let function_call: FunctionCall = serde_json::from_str(json).unwrap();

        assert_eq!(function_call.name, "test_function");
        assert!(function_call.arguments.is_array());
        assert_eq!(function_call.arguments[0], "arg1");
        assert_eq!(function_call.arguments[1], "arg2");
    }

    #[test]
    fn test_function_call_arguments_serialization() {
        let function_call = FunctionCall {
            name: "test_function".to_string(),
            arguments: serde_json::json!({"param": "value"}),
        };

        let json = serde_json::to_string(&function_call).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "test_function");
        assert_eq!(parsed["arguments"]["param"], "value");
    }

    #[test]
    fn test_tool_call_with_optional_id() {
        // Test ToolCall without id
        let json = r#"{"function": {"name": "test_func", "arguments": "{\"key\": \"value\"}"}}"#;
        let tool_call: ToolCall = serde_json::from_str(json).unwrap();

        assert!(tool_call.id.is_none());
        assert_eq!(tool_call.function.name, "test_func");
    }

    #[test]
    fn test_tool_call_with_id() {
        // Test ToolCall with id
        let json = r#"{"id": "call_123", "function": {"name": "test_func", "arguments": "{\"key\": \"value\"}"}}"#;
        let tool_call: ToolCall = serde_json::from_str(json).unwrap();

        assert_eq!(tool_call.id, Some("call_123".to_string()));
        assert_eq!(tool_call.function.name, "test_func");
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            id: Some("call_456".to_string()),
            tool_type: Some("function".to_string()),
            function: FunctionCall {
                name: "get_weather".to_string(),
                arguments: serde_json::json!({"location": "New York"}),
            },
        };

        let json = serde_json::to_string(&tool_call).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], "call_456");
        assert_eq!(parsed["function"]["name"], "get_weather");
    }

    #[test]
    fn test_tool_choice_variants() {
        // Test ToolChoice::Auto
        let auto_choice = ToolChoice::Auto("auto".to_string());
        match auto_choice {
            ToolChoice::Auto(s) => assert_eq!(s, "auto"),
            _ => panic!("Expected Auto variant"),
        }

        // Test ToolChoice::None
        let none_choice = ToolChoice::None("none".to_string());
        match none_choice {
            ToolChoice::None(s) => assert_eq!(s, "none"),
            _ => panic!("Expected None variant"),
        }

        // Test ToolChoice::Specific
        let specific_choice = ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: crate::models::chat::FunctionChoice {
                name: "my_function".to_string(),
            },
        };
        match specific_choice {
            ToolChoice::Specific {
                tool_type,
                function,
            } => {
                assert_eq!(tool_type, "function");
                assert_eq!(function.name, "my_function");
            }
            _ => panic!("Expected Specific variant"),
        }
    }

    #[test]
    fn test_tool_choice_serialization() {
        // Test Auto serialization
        let auto_choice = ToolChoice::Auto("auto".to_string());
        let json = serde_json::to_string(&auto_choice).unwrap();
        assert_eq!(json, "\"auto\"");

        // Test None serialization
        let none_choice = ToolChoice::None("none".to_string());
        let json = serde_json::to_string(&none_choice).unwrap();
        assert_eq!(json, "\"none\"");

        // Test Specific serialization
        let specific_choice = ToolChoice::Specific {
            tool_type: "function".to_string(),
            function: crate::models::chat::FunctionChoice {
                name: "my_function".to_string(),
            },
        };
        let json = serde_json::to_string(&specific_choice).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "function");
        assert_eq!(parsed["function"]["name"], "my_function");
    }

    #[test]
    fn test_edge_cases_function_arguments() {
        // Test with empty string
        let json = r#"{"name": "test", "arguments": ""}"#;
        let function_call: FunctionCall = serde_json::from_str(json).unwrap();
        assert_eq!(function_call.arguments, Value::String(String::new()));

        // Test with null (should remain null)
        let json = r#"{"name": "test", "arguments": null}"#;
        let function_call: FunctionCall = serde_json::from_str(json).unwrap();
        assert_eq!(function_call.arguments, Value::Null);

        // Test with number (should remain number)
        let json = r#"{"name": "test", "arguments": 42}"#;
        let function_call: FunctionCall = serde_json::from_str(json).unwrap();
        assert_eq!(
            function_call.arguments,
            Value::Number(serde_json::Number::from(42))
        );

        // Test with boolean (should remain boolean)
        let json = r#"{"name": "test", "arguments": true}"#;
        let function_call: FunctionCall = serde_json::from_str(json).unwrap();
        assert_eq!(function_call.arguments, Value::Bool(true));
    }
}
