use serde::{Deserialize, Serialize};
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::variables;

use crate::mcp_types::{
    CallToolRequest, ErrorCode, InitializeRequest, InitializeResponse, JsonRpcRequest,
    JsonRpcResponse, ListToolsResponse, McpProtocolVersion, ServerCapabilities, ServerInfo,
    ToolContent, ToolMetadata, ToolResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub server_info: ServerInfo,
    #[serde(default = "default_validate_arguments")]
    pub validate_arguments: bool,
}

fn default_validate_arguments() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct ToolScope {
    pub component: Option<String>,
    pub readonly: bool,
}

impl ToolScope {
    /// Parse a URL path to extract component scope following GitHub's pattern
    ///
    /// Supported patterns:
    /// - `/mcp` -> All tools (unscoped) - returns Ok(None)
    /// - `/mcp/readonly` -> All tools (readonly) - returns Ok(Some(...))
    /// - `/mcp/x/{component}` -> Component tools - returns Ok(Some(...))
    /// - `/mcp/x/{component}/readonly` -> Component tools (readonly) - returns Ok(Some(...))
    ///
    /// Invalid paths return Err
    fn from_path(path: &str) -> Result<Option<Self>, String> {
        let path = path.trim_start_matches('/').trim_end_matches('/');

        // Handle empty, root, or just "mcp" - valid unscoped paths
        if path.is_empty() || path == "mcp" {
            return Ok(None);
        }

        // Handle /mcp/readonly - all tools in readonly mode
        if path == "mcp/readonly" {
            return Ok(Some(Self {
                component: None,
                readonly: true,
            }));
        }

        // Must start with mcp/x/ for component scoping
        path.strip_prefix("mcp/x/").map_or_else(
            || {
                if path.starts_with("mcp/") {
                    // Path starts with mcp/ but doesn't match any valid pattern
                    Err(format!("Invalid MCP path: /{path}"))
                } else {
                    // Path doesn't start with mcp
                    Err(format!(
                        "Invalid path: /{path}. MCP endpoints must start with /mcp"
                    ))
                }
            },
            |remaining| {
                let parts: Vec<&str> = remaining.split('/').collect();

                match parts.as_slice() {
                    [] => Err("Invalid path: /mcp/x/ requires a component name".to_string()),
                    [component] if !component.is_empty() => Ok(Some(Self {
                        component: Some((*component).to_string()),
                        readonly: false,
                    })),
                    [component, "readonly"] if !component.is_empty() => Ok(Some(Self {
                        component: Some((*component).to_string()),
                        readonly: true,
                    })),
                    _ => Err(format!("Invalid path: {path}")),
                }
            },
        )
    }
}

pub struct McpGateway {
    config: GatewayConfig,
    scope: Option<ToolScope>,
    allowed_toolsets: Option<Vec<String>>,
}

impl McpGateway {
    pub fn new(
        config: GatewayConfig,
        scope: Option<ToolScope>,
        allowed_toolsets: Option<Vec<String>>,
    ) -> Self {
        Self {
            config,
            scope,
            allowed_toolsets,
        }
    }

    /// Convert `snake_case` to kebab-case for component names
    fn snake_to_kebab(name: &str) -> String {
        name.replace('_', "-")
    }

    /// Fetch metadata for all tools in a component
    async fn fetch_component_tools(&self, component_name: &str) -> Vec<ToolMetadata> {
        let component_name_kebab = Self::snake_to_kebab(component_name);
        let component_url = format!("http://{component_name_kebab}.spin.internal/");

        let req = Request::builder()
            .method(Method::Get)
            .uri(&component_url)
            .build();

        match spin_sdk::http::send::<_, spin_sdk::http::Response>(req).await {
            Ok(resp) => {
                if *resp.status() == 200 {
                    match serde_json::from_slice::<Vec<ToolMetadata>>(resp.body()) {
                        Ok(tools) => tools,
                        Err(e) => {
                            eprintln!(
                                "Failed to parse metadata from component '{component_name}': {e}"
                            );
                            vec![]
                        }
                    }
                } else {
                    eprintln!(
                        "Component '{}' returned status {} for metadata request",
                        component_name,
                        resp.status()
                    );
                    vec![]
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch metadata from component '{component_name}': {e}");
                vec![]
            }
        }
    }

    /// Validate tool arguments against the tool's input schema
    fn validate_arguments(
        tool_name: &str,
        schema: &serde_json::Value,
        arguments: &serde_json::Value,
    ) -> Result<(), String> {
        match jsonschema::validator_for(schema) {
            Ok(validator) => {
                // Use iter_errors which returns an iterator
                let errors: Vec<jsonschema::ValidationError<'_>> =
                    validator.iter_errors(arguments).collect();
                if errors.is_empty() {
                    Ok(())
                } else {
                    let error_messages: Vec<String> =
                        errors.iter().map(|error| format!("{error}")).collect();
                    Err(format!(
                        "Invalid arguments for tool '{}': {}",
                        tool_name,
                        error_messages.join("; ")
                    ))
                }
            }
            Err(e) => Err(format!(
                "Failed to compile schema for tool '{tool_name}': {e}"
            )),
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        match request.method.as_str() {
            "initialize" => Some(self.handle_initialize(request)),
            "initialized" => {
                // This is a notification, no response needed
                None
            }
            "tools/list" => Some(self.handle_list_tools(request).await),
            "tools/call" => Some(self.handle_call_tool(request).await),
            "prompts/list" => Some(Self::handle_list_prompts(request)),
            "resources/list" => Some(Self::handle_list_resources(request)),
            "ping" => Some(Self::handle_ping(self, request)),
            _ => Some(JsonRpcResponse::error(
                request.id,
                ErrorCode::METHOD_NOT_FOUND.0,
                &format!("Method '{}' not found", request.method),
            )),
        }
    }

    fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params: InitializeRequest = match request.params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &format!("Invalid initialize parameters: {e}"),
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    ErrorCode::INVALID_PARAMS.0,
                    "Missing initialize parameters",
                );
            }
        };

        if params.protocol_version != McpProtocolVersion::V1 {
            return JsonRpcResponse::error(
                request.id,
                ErrorCode::INVALID_REQUEST.0,
                "Unsupported protocol version",
            );
        }

        let response = InitializeResponse {
            protocol_version: McpProtocolVersion::V1,
            capabilities: ServerCapabilities {
                tools: Some(serde_json::json!({
                    "listChanged": true
                })),
                resources: Some(serde_json::json!({
                    "subscribe": false,
                    "listChanged": false
                })),
                prompts: Some(serde_json::json!({
                    "listChanged": false
                })),
                experimental_capabilities: Some(serde_json::json!({
                    "logging": {}
                })),
            },
            server_info: self.config.server_info.clone(),
            instructions: Some(
                "This MCP server provides access to tools via WebAssembly components. \
                 Each tool is implemented as an independent component with its own \
                 capabilities and annotations."
                    .to_string(),
            ),
        };

        match serde_json::to_value(response) {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(e) => JsonRpcResponse::error(
                request.id,
                ErrorCode::INTERNAL_ERROR.0,
                &format!("Failed to serialize response: {e}"),
            ),
        }
    }

    async fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Get the list of components from the spin variable
        let component_names_str = match variables::get("component_names") {
            Ok(components) => components,
            Err(e) => {
                return JsonRpcResponse::error(
                    request.id,
                    ErrorCode::INTERNAL_ERROR.0,
                    &format!("Failed to get components configuration: {e}"),
                );
            }
        };

        // Parse the comma-separated list of component names
        let all_component_names: Vec<&str> =
            component_names_str.split(',').map(str::trim).collect();

        // Filter components based on scope and allowed_toolsets header
        let mut component_names = if let Some(ref scope) = self.scope {
            if let Some(ref component_filter) = scope.component {
                // Only include the specified component if it exists
                all_component_names
                    .into_iter()
                    .filter(|name| *name == component_filter)
                    .collect()
            } else {
                all_component_names
            }
        } else {
            all_component_names
        };

        // Further filter by X-MCP-Toolsets header if present
        if let Some(ref allowed) = self.allowed_toolsets {
            component_names.retain(|name| allowed.iter().any(|a| a == name));
        }

        // Create futures for fetching metadata from selected components in parallel
        let metadata_futures: Vec<_> = component_names
            .iter()
            .map(|component_name| async move {
                let tools = self.fetch_component_tools(component_name).await;
                ((*component_name).to_string(), tools)
            })
            .collect();

        // Execute all futures concurrently and collect results
        let results = futures::future::join_all(metadata_futures).await;

        // Process results - only prefix tool names when unscoped
        let mut tools: Vec<ToolMetadata> = Vec::new();
        let is_scoped = self.scope.as_ref().is_some_and(|s| s.component.is_some());

        for (component_name, component_tools) in results {
            for mut tool in component_tools {
                // Only prefix tool names when unscoped (at /mcp root)
                if !is_scoped {
                    tool.name = format!("{}__{}", component_name, tool.name);
                }
                tools.push(tool);
            }
        }

        let response = ListToolsResponse { tools };
        match serde_json::to_value(response) {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(e) => JsonRpcResponse::error(
                request.id,
                ErrorCode::INTERNAL_ERROR.0,
                &format!("Failed to serialize response: {e}"),
            ),
        }
    }

    async fn execute_tool_call(
        &self,
        component_name: &str,
        tool_name: &str,
        tool_arguments: serde_json::Value,
    ) -> Result<ToolResponse, String> {
        let component_name_kebab = Self::snake_to_kebab(component_name);
        let tool_url = format!("http://{component_name_kebab}.spin.internal/{tool_name}");

        let req = Request::builder()
            .method(Method::Post)
            .uri(&tool_url)
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_vec(&tool_arguments)
                    .unwrap_or_else(|_| br#"{"error":"Failed to serialize request"}"#.to_vec()),
            )
            .build();

        match spin_sdk::http::send::<_, spin_sdk::http::Response>(req).await {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.body();

                if *status == 200 {
                    serde_json::from_slice::<ToolResponse>(body)
                        .map_err(|e| format!("Tool returned invalid response format: {e}"))
                } else {
                    let error_text = String::from_utf8_lossy(body);
                    Ok(ToolResponse {
                        content: vec![ToolContent::Text {
                            text: format!("Tool execution failed (status {status}): {error_text}"),
                            annotations: None,
                        }],
                        structured_content: None,
                        is_error: Some(true),
                    })
                }
            }
            Err(e) => Err(format!("Failed to call tool '{tool_name}': {e}")),
        }
    }

    /// Parse and validate the tool call parameters
    fn parse_tool_params(
        request_id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> Result<CallToolRequest, JsonRpcResponse> {
        match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                JsonRpcResponse::error(
                    request_id,
                    ErrorCode::INVALID_PARAMS.0,
                    &format!("Invalid params: {e}"),
                )
            }),
            None => Err(JsonRpcResponse::error(
                request_id,
                ErrorCode::INVALID_PARAMS.0,
                "Invalid params: missing required parameters",
            )),
        }
    }

    /// Parse component and tool names from the prefixed format
    fn parse_tool_name(
        request_id: Option<serde_json::Value>,
        tool_name: &str,
    ) -> Result<(String, String), JsonRpcResponse> {
        tool_name.split_once("__").map_or_else(
            || {
                Err(JsonRpcResponse::error(
                    request_id,
                    ErrorCode::INVALID_PARAMS.0,
                    &format!("Invalid tool name format '{tool_name}'. Expected format: 'component__toolname'"),
                ))
            },
            |(component, tool)| Ok((component.to_string(), tool.to_string())),
        )
    }

    async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Check if in readonly mode
        if let Some(ref scope) = self.scope
            && scope.readonly
        {
            return JsonRpcResponse::error(
                request.id,
                ErrorCode::INVALID_REQUEST.0,
                "Tool execution is disabled in readonly mode",
            );
        }

        // Parse and validate parameters
        let params = match Self::parse_tool_params(request.id.clone(), request.params) {
            Ok(p) => p,
            Err(e) => return e,
        };

        // Determine component and tool names based on scope
        let (component_name, actual_tool_name) = if let Some(ref scope) = self.scope {
            if let Some(ref scoped_component) = scope.component {
                // When scoped, tool names aren't prefixed, use scope's component
                (scoped_component.clone(), params.name.clone())
            } else {
                // Unscoped but might have readonly - parse prefixed name
                match Self::parse_tool_name(request.id.clone(), &params.name) {
                    Ok(names) => names,
                    Err(e) => return e,
                }
            }
        } else {
            // Unscoped - parse prefixed name
            match Self::parse_tool_name(request.id.clone(), &params.name) {
                Ok(names) => names,
                Err(e) => return e,
            }
        };

        // Validate scope access if scoped
        if let Some(ref scope) = self.scope
            && let Some(ref scope_component) = scope.component
            && &component_name != scope_component
        {
            return JsonRpcResponse::error(
                request.id,
                ErrorCode::INVALID_PARAMS.0,
                &format!(
                    "Tool '{}' is not accessible in the current scope",
                    params.name
                ),
            );
        }

        // Validate against X-MCP-Toolsets header if present
        if let Some(ref allowed) = self.allowed_toolsets
            && !allowed.contains(&component_name)
        {
            return JsonRpcResponse::error(
                request.id,
                ErrorCode::INVALID_PARAMS.0,
                &format!("Component '{component_name}' is not in the allowed toolsets"),
            );
        }

        // Validate arguments if validation is enabled
        let tool_arguments = params.arguments.unwrap_or_else(|| serde_json::json!({}));

        if self.config.validate_arguments {
            // Fetch the tool metadata from the specific component for validation
            let tools = self.fetch_component_tools(&component_name).await;
            let tool_metadata = tools.into_iter().find(|t| t.name == actual_tool_name);

            match tool_metadata {
                Some(metadata) => {
                    // Validate arguments against the tool's input schema
                    if let Err(validation_error) = Self::validate_arguments(
                        &params.name,
                        &metadata.input_schema,
                        &tool_arguments,
                    ) {
                        return JsonRpcResponse::error(
                            request.id,
                            ErrorCode::INVALID_PARAMS.0,
                            &format!("Invalid params: {validation_error}"),
                        );
                    }
                }
                None => {
                    return JsonRpcResponse::error(
                        request.id,
                        ErrorCode::INVALID_PARAMS.0,
                        &format!(
                            "Unknown tool '{actual_tool_name}' in component '{component_name}'"
                        ),
                    );
                }
            }
        }

        // Execute the tool call
        match self
            .execute_tool_call(&component_name, &actual_tool_name, tool_arguments)
            .await
        {
            Ok(tool_response) => match serde_json::to_value(tool_response) {
                Ok(value) => JsonRpcResponse::success(request.id, value),
                Err(e) => JsonRpcResponse::error(
                    request.id,
                    ErrorCode::INTERNAL_ERROR.0,
                    &format!("Internal error: {e}"),
                ),
            },
            Err(e) => JsonRpcResponse::error(
                request.id,
                ErrorCode::INTERNAL_ERROR.0,
                &format!("Internal error: {e}"),
            ),
        }
    }

    fn handle_ping(_gateway: &Self, request: JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse::success(request.id, serde_json::json!({}))
    }

    fn handle_list_prompts(request: JsonRpcRequest) -> JsonRpcResponse {
        // Return empty prompts list - this gateway doesn't support prompts
        JsonRpcResponse::success(
            request.id,
            serde_json::json!({
                "prompts": []
            }),
        )
    }

    fn handle_list_resources(request: JsonRpcRequest) -> JsonRpcResponse {
        // Return empty resources list - this gateway doesn't support resources
        JsonRpcResponse::success(
            request.id,
            serde_json::json!({
                "resources": []
            }),
        )
    }
}

#[allow(clippy::too_many_lines)] // This function handles the entire MCP request flow
pub async fn handle_mcp_request(req: Request) -> Response {
    // Handle CORS preflight first
    if *req.method() == Method::Options {
        return Response::builder()
            .status(200)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header(
                "Access-Control-Allow-Headers",
                "Content-Type, X-MCP-Toolsets, X-MCP-Readonly",
            )
            .build();
    }

    // Only accept POST requests for MCP operations
    if *req.method() != Method::Post {
        return Response::builder()
            .status(405)
            .header("Allow", "POST, OPTIONS")
            .header("Access-Control-Allow-Origin", "*")
            .body(b"Method not allowed. MCP requires POST requests".to_vec())
            .build();
    }

    // Extract scope from path
    let path = req.path();
    let mut scope = match ToolScope::from_path(path) {
        Ok(s) => s,
        Err(err) => {
            // Invalid path - return 404
            return Response::builder()
                .status(404)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(
                    serde_json::to_vec(&serde_json::json!({
                        "error": err
                    }))
                    .unwrap_or_else(|_| b"{\"error\":\"Not found\"}".to_vec()),
                )
                .build();
        }
    };

    // Parse headers to augment scope
    let mut allowed_toolsets: Option<Vec<String>> = None;

    for (name, value) in req.headers() {
        if name.eq_ignore_ascii_case("x-mcp-toolsets") {
            if let Ok(toolsets_str) = std::str::from_utf8(value.as_bytes()) {
                // Parse comma-separated list of allowed toolsets/components
                allowed_toolsets = Some(
                    toolsets_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                );
            }
        } else if name.eq_ignore_ascii_case("x-mcp-readonly")
            && let Ok(readonly_str) = std::str::from_utf8(value.as_bytes())
            && readonly_str.eq_ignore_ascii_case("true")
        {
            // Force readonly mode
            if let Some(ref mut s) = scope {
                s.readonly = true;
            } else {
                scope = Some(ToolScope {
                    component: None,
                    readonly: true,
                });
            }
        }
    }

    // Parse JSON-RPC request
    let request: JsonRpcRequest = match serde_json::from_slice::<JsonRpcRequest>(req.body()) {
        Ok(r) => {
            // Validate JSON-RPC version
            if r.jsonrpc != "2.0" {
                let error_response = JsonRpcResponse::error(
                    r.id,
                    ErrorCode::INVALID_REQUEST.0,
                    "Unsupported JSON-RPC version",
                );
                return Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(serde_json::to_vec(&error_response).unwrap_or_else(|_| {
                        br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal serialization error"}}"#.to_vec()
                    }))
                    .build();
            }
            r
        }
        Err(e) => {
            let error_response = JsonRpcResponse::error(
                None,
                ErrorCode::PARSE_ERROR.0,
                &format!("Invalid JSON-RPC request: {e}"),
            );
            return Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_vec(&error_response).unwrap_or_else(|_| {
                    br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal serialization error"}}"#.to_vec()
                }))
                .build();
        }
    };

    // Create gateway with config
    let validate_arguments = variables::get("validate_arguments")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    let config = GatewayConfig {
        server_info: ServerInfo {
            name: "mcp-gateway".to_string(),
            version: "0.0.1".to_string(),
        },
        validate_arguments,
    };

    let gateway = McpGateway::new(config, scope, allowed_toolsets);

    // Handle the request
    gateway.handle_request(request).await.map_or_else(
        || {
            // Notification - return empty response
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(Vec::new())
                .build()
        },
        |response| {
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_vec(&response).unwrap_or_else(|_| {
                    br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal serialization error"}}"#.to_vec()
                }))
                .build()
        },
    )
}
