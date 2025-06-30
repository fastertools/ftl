#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::{Request, Response, ServerInfo, McpProtocolVersion};
    use crate::gateway::{GatewayConfig, McpGateway, ToolEndpoint};
    use serde_json::json;

    #[test]
    fn test_gateway_initialize() {
        let config = GatewayConfig {
            tools: vec![],
            server_info: ServerInfo {
                name: "test-gateway".to_string(),
                version: "1.0.0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        };

        let mut gateway = McpGateway::new(config);
        
        let request = Request {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "initialize".to_string(),
            params: json!({
                "protocolVersion": "2025-03-26",
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        };

        let response = gateway.handle_request(request);
        
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        
        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2025-03-26");
        assert_eq!(result["serverInfo"]["name"], "test-gateway");
    }

    #[test]
    fn test_gateway_list_tools_empty() {
        let config = GatewayConfig {
            tools: vec![],
            server_info: ServerInfo {
                name: "test-gateway".to_string(),
                version: "1.0.0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        };

        let mut gateway = McpGateway::new(config);
        
        let request = Request {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "tools/list".to_string(),
            params: json!({}),
        };

        let response = gateway.handle_request(request);
        
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["tools"], json!([]));
    }

    #[test]
    fn test_gateway_invalid_method() {
        let config = GatewayConfig {
            tools: vec![],
            server_info: ServerInfo {
                name: "test-gateway".to_string(),
                version: "1.0.0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        };

        let mut gateway = McpGateway::new(config);
        
        let request = Request {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "invalid/method".to_string(),
            params: json!({}),
        };

        let response = gateway.handle_request(request);
        
        assert!(response.error.is_some());
        assert!(response.result.is_none());
        
        let error = response.error.unwrap();
        assert_eq!(error.code.0, -32601); // MethodNotFound
    }

    #[test]
    fn test_gateway_tool_not_found() {
        let config = GatewayConfig {
            tools: vec![
                ToolEndpoint {
                    name: "tool1".to_string(),
                    route: "/tool1".to_string(),
                    description: None,
                }
            ],
            server_info: ServerInfo {
                name: "test-gateway".to_string(),
                version: "1.0.0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        };

        let mut gateway = McpGateway::new(config);
        
        let request = Request {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "tools/call".to_string(),
            params: json!({
                "name": "nonexistent",
                "arguments": {}
            }),
        };

        let response = gateway.handle_request(request);
        
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert!(error.message.contains("not found"));
    }
}