use serde::{Deserialize, Serialize};
use serde_json::Value;
use spin_sdk::http::Method;

use crate::{
    mcp::{
        CallToolRequest, ErrorCode, InitializeRequest, InitializeResponse, ListToolsRequest,
        ListToolsResponse, McpProtocolVersion, ServerInfo,
    },
    types::{JsonRpcRequest, JsonRpcResponse},
};

/// Configuration for a tool endpoint in the gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEndpoint {
    /// Name of the tool
    pub name: String,
    /// HTTP route to the tool's MCP endpoint
    pub route: String,
    /// Optional description override
    pub description: Option<String>,
}

/// Configuration for the MCP gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// List of tool endpoints to proxy
    pub tools: Vec<ToolEndpoint>,
    /// Gateway server info
    pub server_info: ServerInfo,
    /// Base URL for internal tool requests (e.g., "http://localhost:3000")
    pub base_url: String,
}

/// MCP Gateway that proxies requests to multiple tool servers
pub struct McpGateway {
    config: GatewayConfig,
}

impl McpGateway {
    /// Create a new MCP gateway
    pub fn new(config: GatewayConfig) -> Self {
        Self { config }
    }

    /// Handle MCP protocol requests
    pub async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            _ => JsonRpcResponse::error(
                request.id,
                ErrorCode::METHOD_NOT_FOUND.0,
                &format!("Method {} not found", request.method),
            ),
        }
    }

    /// Handle initialize request
    fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: InitializeRequest =
            match serde_json::from_value(request.params.unwrap_or(serde_json::json!({}))) {
                Ok(p) => p,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &format!("Invalid initialize parameters: {e}"),
                    );
                }
            };

        // Check protocol version compatibility
        if params.protocol_version != McpProtocolVersion::V1 {
            return JsonRpcResponse::error(
                request.id,
                ErrorCode::INVALID_REQUEST.0,
                &format!(
                    "Unsupported protocol version: {:?}",
                    params.protocol_version
                ),
            );
        }

        let response = InitializeResponse {
            protocol_version: McpProtocolVersion::V1,
            capabilities: serde_json::json!({
                "tools": {}
            }),
            server_info: self.config.server_info.clone(),
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(response).unwrap()),
            error: None,
            id: request.id.unwrap_or(Value::Null),
        }
    }

    /// Handle tools/list request
    async fn handle_list_tools(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let _params: ListToolsRequest =
            match serde_json::from_value(request.params.unwrap_or(serde_json::json!({}))) {
                Ok(p) => p,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &format!("Invalid list tools parameters: {e}"),
                    );
                }
            };

        // Always fetch fresh tool list from all endpoints
        let mut all_tools = Vec::new();

        for endpoint in &self.config.tools.clone() {
            // Use Spin's service chaining for internal requests
            let component_id = endpoint.route.trim_start_matches('/');
            let tool_url = format!("http://{component_id}.spin.internal/mcp");

            // Create a tools/list request
            let list_request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(serde_json::Value::Number(1.into())),
                method: "tools/list".to_string(),
                params: Some(serde_json::to_value(ListToolsRequest {}).unwrap()),
            };

            // Send request to tool endpoint
            match self.forward_request(&tool_url, list_request).await {
                Ok(response) => {
                    if let Some(result) = response.result
                        && let Ok(list_response) =
                            serde_json::from_value::<ListToolsResponse>(result)
                    {
                        for mut tool in list_response.tools {
                            // Override description if configured
                            if let Some(desc) = &endpoint.description {
                                tool.description = Some(desc.clone());
                            }
                            all_tools.push(tool);
                        }
                    }
                }
                Err(e) => {
                    // Log error but continue with other tools
                    eprintln!("Failed to fetch tools from {}: {}", endpoint.name, e);
                }
            }
        }

        let response = ListToolsResponse { tools: all_tools };
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(response).unwrap()),
            error: None,
            id: request.id.unwrap_or(Value::Null),
        }
    }

    /// Handle tools/call request
    async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: CallToolRequest =
            match serde_json::from_value(request.params.unwrap_or(serde_json::json!({}))) {
                Ok(p) => p,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &format!("Invalid call tool parameters: {e}"),
                    );
                }
            };

        // Find the endpoint for this tool
        let endpoint = match self.config.tools.iter().find(|t| t.name == params.name) {
            Some(e) => e,
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    ErrorCode::INVALID_REQUEST.0,
                    &format!("Tool '{}' not found", params.name),
                );
            }
        };

        // Forward the request to the tool's MCP endpoint using Spin's service chaining
        // Extract component name from route (e.g., "/echo-tool" -> "echo-tool")
        let component_id = endpoint.route.trim_start_matches('/');
        let tool_url = format!("http://{component_id}.spin.internal/mcp");
        let forwarded_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            method: "tools/call".to_string(),
            params: Some(serde_json::to_value(params).unwrap()),
        };

        match self.forward_request(&tool_url, forwarded_request).await {
            Ok(response) => response,
            Err(e) => JsonRpcResponse::error(
                request.id,
                ErrorCode::INTERNAL_ERROR.0,
                &format!("Failed to call tool: {e}"),
            ),
        }
    }

    /// Forward a request to a tool endpoint
    async fn forward_request(
        &self,
        url: &str,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Serialize request body
        let body = serde_json::to_vec(&request)?;

        // Create and send outgoing request using Spin SDK
        let req = spin_sdk::http::Request::builder()
            .method(Method::Post)
            .uri(url)
            .header("Content-Type", "application/json")
            .body(body)
            .build();

        // Send the request and await the response
        let resp: spin_sdk::http::Response = spin_sdk::http::send(req)
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        // Parse response body
        let response_body: JsonRpcResponse = serde_json::from_slice(resp.body())
            .map_err(|e| format!("Failed to parse response: {e}"))?;
        Ok(response_body)
    }
}

/// Handler for a single gateway instance
pub struct GatewayHandler {
    gateway: std::sync::Arc<std::sync::Mutex<McpGateway>>,
}

impl GatewayHandler {
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            gateway: std::sync::Arc::new(std::sync::Mutex::new(McpGateway::new(config))),
        }
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn handle_request(&self, req: spin_sdk::http::Request) -> spin_sdk::http::Response {
        use spin_sdk::http::Response as SpinResponse;

        // Handle CORS preflight
        if req.method() == &spin_sdk::http::Method::Options {
            return SpinResponse::builder()
                .status(200)
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .build();
        }

        // Only accept POST requests
        if req.method() != &spin_sdk::http::Method::Post {
            return SpinResponse::builder()
                .status(405)
                .header("Allow", "POST, OPTIONS")
                .body("Method not allowed")
                .build();
        }

        // Parse request body
        let body = req.body();
        let request: JsonRpcRequest = match serde_json::from_slice(body) {
            Ok(r) => r,
            Err(e) => {
                let error_response = JsonRpcResponse::error(
                    None,
                    ErrorCode::PARSE_ERROR.0,
                    &format!("Invalid JSON-RPC request: {e}"),
                );
                return SpinResponse::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(serde_json::to_vec(&error_response).unwrap())
                    .build();
            }
        };

        // Handle request asynchronously
        // In WASM/Spin environment, this is safe as we're single-threaded
        let response = {
            let mut gateway = self.gateway.lock().unwrap();
            gateway.handle_request(request).await
        };

        SpinResponse::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .body(serde_json::to_vec(&response).unwrap())
            .build()
    }
}

/// Macro to create a Spin component for the MCP gateway
#[macro_export]
macro_rules! ftl_mcp_gateway {
    ($config:expr) => {
        static GATEWAY_HANDLER: std::sync::OnceLock<$crate::gateway::GatewayHandler> =
            std::sync::OnceLock::new();

        #[spin_sdk::http_component]
        async fn handle_request(req: spin_sdk::http::Request) -> spin_sdk::http::Response {
            let handler =
                GATEWAY_HANDLER.get_or_init(|| $crate::gateway::GatewayHandler::new($config));
            handler.handle_request(req).await
        }
    };
}
