#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        gateway::{GatewayConfig, McpGateway, ToolEndpoint},
        mcp::ServerInfo,
        types::JsonRpcRequest,
    };

    #[tokio::test]
    async fn test_gateway_initialize() {
        let config = GatewayConfig {
            tools: vec![],
            server_info: ServerInfo {
                name: "test-gateway".to_string(),
                version: "1.0.0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        };

        let mut gateway = McpGateway::new(config);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2025-03-26",
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        };

        let response = gateway.handle_request(request).await;

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2025-03-26");
        assert_eq!(result["serverInfo"]["name"], "test-gateway");
    }

    #[tokio::test]
    async fn test_gateway_list_tools_empty() {
        let config = GatewayConfig {
            tools: vec![],
            server_info: ServerInfo {
                name: "test-gateway".to_string(),
                version: "1.0.0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        };

        let mut gateway = McpGateway::new(config);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: Some(json!({})),
        };

        let response = gateway.handle_request(request).await;

        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["tools"], json!([]));
    }

    #[tokio::test]
    async fn test_gateway_invalid_method() {
        let config = GatewayConfig {
            tools: vec![],
            server_info: ServerInfo {
                name: "test-gateway".to_string(),
                version: "1.0.0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        };

        let mut gateway = McpGateway::new(config);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "invalid/method".to_string(),
            params: Some(json!({})),
        };

        let response = gateway.handle_request(request).await;

        assert!(response.error.is_some());
        assert!(response.result.is_none());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601); // MethodNotFound
    }

    #[tokio::test]
    async fn test_gateway_tool_not_found() {
        let config = GatewayConfig {
            tools: vec![ToolEndpoint {
                name: "tool1".to_string(),
                route: "/tool1".to_string(),
                description: None,
            }],
            server_info: ServerInfo {
                name: "test-gateway".to_string(),
                version: "1.0.0".to_string(),
            },
            base_url: "http://localhost".to_string(),
        };

        let mut gateway = McpGateway::new(config);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "nonexistent",
                "arguments": {}
            })),
        };

        let response = gateway.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert!(error.message.contains("not found"));
    }
}
