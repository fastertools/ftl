use crate::tool::{Tool, ToolInfo};
use crate::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpError, ToolError};
use serde_json::{json, Value};

pub struct McpServer<T: Tool> {
    tool: T,
}

impl<T: Tool> McpServer<T> {
    pub fn new(tool: T) -> Self {
        Self { tool }
    }

    pub fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let request_id = request.id.clone().unwrap_or(serde_json::Value::Null);
        match self.process_request(request) {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: request_id,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(self.error_to_jsonrpc(e)),
                id: request_id,
            },
        }
    }

    fn process_request(&self, request: JsonRpcRequest) -> Result<Value, McpError> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(request.params),
            method => Err(McpError::MethodNotFound(method.to_string())),
        }
    }

    fn handle_initialize(&self, _params: Option<Value>) -> Result<Value, McpError> {
        Ok(json!({
            "protocolVersion": "2025-03-26",
            "serverInfo": {
                "name": self.tool.server_name(),
                "version": self.tool.server_version()
            },
            "capabilities": self.tool.capabilities()
        }))
    }

    fn handle_tools_list(&self) -> Result<Value, McpError> {
        let tool_info = ToolInfo::from(&self.tool);
        Ok(json!({
            "tools": [{
                "name": tool_info.name,
                "description": tool_info.description,
                "inputSchema": tool_info.input_schema
            }]
        }))
    }

    fn handle_tools_call(&self, params: Option<Value>) -> Result<Value, McpError> {
        let params = params.ok_or_else(|| {
            McpError::InvalidRequest("Missing params for tools/call".to_string())
        })?;

        let tool_name = params["name"]
            .as_str()
            .ok_or_else(|| McpError::InvalidRequest("Missing tool name".to_string()))?;

        if tool_name != self.tool.name() {
            return Err(McpError::ToolError(ToolError::InvalidArguments(
                format!("Unknown tool: {}", tool_name),
            )));
        }

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        match self.tool.call(&arguments) {
            Ok(result) => Ok(json!(result)),
            Err(e) => Err(McpError::ToolError(e)),
        }
    }

    fn error_to_jsonrpc(&self, error: McpError) -> JsonRpcError {
        match error {
            McpError::InvalidRequest(msg) => JsonRpcError::invalid_request(msg),
            McpError::MethodNotFound(method) => JsonRpcError::method_not_found(method),
            McpError::ToolError(e) => JsonRpcError::internal_error(e.to_string()),
            McpError::JsonError(e) => JsonRpcError::internal_error(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::Tool;
    use crate::types::{ToolError, ToolResult};
    use serde_json::json;

    #[derive(Clone)]
    struct TestTool;

    impl Tool for TestTool {
        fn name(&self) -> &'static str {
            "test_tool"
        }

        fn description(&self) -> &'static str {
            "A test tool"
        }

        fn input_schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                },
                "required": ["input"]
            })
        }

        fn call(&self, args: &Value) -> Result<ToolResult, ToolError> {
            if let Some(input) = args.get("input").and_then(|v| v.as_str()) {
                Ok(ToolResult::text(format!("Processed: {}", input)))
            } else {
                Err(ToolError::InvalidArguments("Missing input".to_string()))
            }
        }
    }

    #[test]
    fn test_initialize() {
        let server = McpServer::new(TestTool);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2025-03-26");
        assert_eq!(result["serverInfo"]["name"], "ftl-test_tool");
    }

    #[test]
    fn test_tools_list() {
        let server = McpServer::new(TestTool);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "test_tool");
        assert_eq!(tools[0]["description"], "A test tool");
    }

    #[test]
    fn test_tools_call_success() {
        let server = McpServer::new(TestTool);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool",
                "arguments": {
                    "input": "hello"
                }
            })),
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        assert_eq!(result["content"][0]["text"], "Processed: hello");
    }

    #[test]
    fn test_tools_call_missing_argument() {
        let server = McpServer::new(TestTool);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool",
                "arguments": {}
            })),
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32603);
    }

    #[test]
    fn test_unknown_method() {
        let server = McpServer::new(TestTool);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "unknown/method".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[test]
    fn test_missing_params() {
        let server = McpServer::new(TestTool);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32600);
    }

    #[test]
    fn test_wrong_tool_name() {
        let server = McpServer::new(TestTool);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "wrong_tool",
                "arguments": {"input": "test"}
            })),
            id: Some(json!(1)),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_some());
    }
}