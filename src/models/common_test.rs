//! Unit tests for tool calling functionality

#[cfg(test)]
mod tests {
    use super::super::common::*;
    use serde_json::json;

    #[test]
    fn test_tool_creation() {
        let tool = Tool::function(
            "get_weather".to_string(),
            "Get weather for a location".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"]
                    }
                },
                "required": ["location"]
            }),
        );

        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.function.name, "get_weather");
        assert_eq!(tool.function.description, "Get weather for a location");
        assert_eq!(tool.function.parameters["type"], "object");
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool::function(
            "calculate".to_string(),
            "Perform calculation".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "expression": {"type": "string"}
                }
            }),
        );

        let serialized = serde_json::to_string(&tool).unwrap();
        assert!(serialized.contains("\"type\":\"function\""));
        assert!(serialized.contains("\"name\":\"calculate\""));
        assert!(serialized.contains("\"description\":\"Perform calculation\""));
    }

    #[test]
    fn test_tool_call_deserialization() {
        let json_str = r#"{
            "id": "call_123",
            "type": "function",
            "function": {
                "name": "get_weather",
                "arguments": "{\"location\": \"San Francisco\", \"unit\": \"celsius\"}"
            }
        }"#;

        let tool_call: ToolCall = serde_json::from_str(json_str).unwrap();
        assert_eq!(tool_call.id, "call_123");
        assert_eq!(tool_call.tool_type, "function");
        assert_eq!(tool_call.function.name, "get_weather");
        assert_eq!(tool_call.function.arguments["location"], "San Francisco");
    }

    #[test]
    fn test_function_call_serialization() {
        let func_call = FunctionCall {
            name: "calculate".to_string(),
            arguments: json!({"expression": "2 + 2"}),
        };

        let serialized = serde_json::to_string(&func_call).unwrap();
        assert!(serialized.contains("\"name\":\"calculate\""));
        assert!(serialized.contains("expression"));
    }

    #[test]
    fn test_multiple_tools() {
        let tools = vec![
            Tool::function(
                "tool1".to_string(),
                "First tool".to_string(),
                json!({"type": "object"}),
            ),
            Tool::function(
                "tool2".to_string(),
                "Second tool".to_string(),
                json!({"type": "object"}),
            ),
            Tool::function(
                "tool3".to_string(),
                "Third tool".to_string(),
                json!({"type": "object"}),
            ),
        ];

        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].function.name, "tool1");
        assert_eq!(tools[1].function.name, "tool2");
        assert_eq!(tools[2].function.name, "tool3");
    }

    #[test]
    fn test_tool_with_complex_parameters() {
        let params = json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "filters": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "field": {"type": "string"},
                            "value": {"type": "string"}
                        }
                    }
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["query"]
        });

        let tool = Tool::function(
            "search".to_string(),
            "Advanced search".to_string(),
            params.clone(),
        );

        assert_eq!(tool.function.parameters, params);
        assert_eq!(tool.function.parameters["properties"]["query"]["type"], "string");
        assert_eq!(tool.function.parameters["properties"]["filters"]["type"], "array");
    }

    #[test]
    fn test_tool_call_response_parsing() {
        let response_json = json!({
            "tool_calls": [
                {
                    "id": "call_001",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": "{\"location\": \"London\"}"
                    }
                },
                {
                    "id": "call_002",
                    "type": "function",
                    "function": {
                        "name": "get_stock_price",
                        "arguments": "{\"symbol\": \"AAPL\"}"
                    }
                }
            ]
        });

        let tool_calls: Vec<ToolCall> = response_json["tool_calls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .collect();

        assert_eq!(tool_calls.len(), 2);
        assert_eq!(tool_calls[0].function.name, "get_weather");
        assert_eq!(tool_calls[1].function.name, "get_stock_price");
    }

    #[test]
    fn test_tool_function_equality() {
        let tool1 = Tool::function(
            "test".to_string(),
            "Test function".to_string(),
            json!({"type": "object"}),
        );

        let tool2 = Tool::function(
            "test".to_string(),
            "Test function".to_string(),
            json!({"type": "object"}),
        );

        // Tools with same properties should be equal in structure
        assert_eq!(tool1.tool_type, tool2.tool_type);
        assert_eq!(tool1.function.name, tool2.function.name);
        assert_eq!(tool1.function.description, tool2.function.description);
    }
}
