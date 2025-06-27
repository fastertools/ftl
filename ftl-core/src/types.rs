use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Error)]
pub enum McpError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    
    #[error("Tool error: {0}")]
    ToolError(#[from] ToolError),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Timeout")]
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    #[serde(rename = "type")]
    pub result_type: String,
    pub content: Vec<TextContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl ToolResult {
    pub fn text(content: String) -> Self {
        Self {
            result_type: "content".to_string(),
            content: vec![TextContent {
                content_type: "text".to_string(),
                text: content,
            }],
        }
    }
}

impl JsonRpcError {
    pub fn invalid_request(message: String) -> Self {
        Self {
            code: -32600,
            message,
            data: None,
        }
    }
    
    pub fn method_not_found(method: String) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }
    
    pub fn internal_error(message: String) -> Self {
        Self {
            code: -32603,
            message,
            data: None,
        }
    }
}

impl JsonRpcResponse {
    pub fn error(id: Option<Value>, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
            id: id.unwrap_or(serde_json::Value::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: Some(json!({"key": "value"})),
            id: Some(json!(1)),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.method, "test");
        assert_eq!(deserialized.params, Some(json!({"key": "value"})));
        assert_eq!(deserialized.id, Some(json!(1)));
    }

    #[test]
    fn test_jsonrpc_response_with_result() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({"status": "ok"})),
            error: None,
            id: json!(1),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(!serialized.contains("\"error\""));
        assert!(serialized.contains("\"result\""));
    }

    #[test]
    fn test_jsonrpc_response_with_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError::method_not_found("unknown".to_string())),
            id: json!(1),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("\"error\""));
        assert!(!serialized.contains("\"result\""));
    }

    #[test]
    fn test_tool_result_creation() {
        let result = ToolResult::text("Hello, world!".to_string());
        
        assert_eq!(result.result_type, "content");
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0].content_type, "text");
        assert_eq!(result.content[0].text, "Hello, world!");
    }

    #[test]
    fn test_jsonrpc_error_codes() {
        let invalid_request = JsonRpcError::invalid_request("bad request".to_string());
        assert_eq!(invalid_request.code, -32600);

        let method_not_found = JsonRpcError::method_not_found("unknown".to_string());
        assert_eq!(method_not_found.code, -32601);

        let internal_error = JsonRpcError::internal_error("server error".to_string());
        assert_eq!(internal_error.code, -32603);
    }

    #[test]
    fn test_tool_error_display() {
        let error = ToolError::InvalidArguments("missing field".to_string());
        assert_eq!(error.to_string(), "Invalid arguments: missing field");

        let error = ToolError::ExecutionError("failed to process".to_string());
        assert_eq!(error.to_string(), "Execution error: failed to process");

        let error = ToolError::Timeout;
        assert_eq!(error.to_string(), "Timeout");
    }
}