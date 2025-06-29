use spin_sdk::http::{Method, Request, Response};

use crate::{server::McpServer, tool::Tool, types::JsonRpcRequest};

/// Create a handler function for serving a single FTL SDK tool as an MCP
/// server over HTTP
///
/// This function handles all HTTP/CORS details and delegates MCP protocol
/// handling to the McpServer.
pub fn create_handler<T: Tool + 'static>(
    tool: T,
) -> impl Fn(Request) -> Result<Response, String> + Clone {
    move |req: Request| -> Result<Response, String> {
        let server = McpServer::new(tool.clone());
        handle_mcp_request(server, req)
    }
}

fn handle_mcp_request<T: Tool>(server: McpServer<T>, req: Request) -> Result<Response, String> {
    // Handle CORS preflight requests
    if *req.method() == Method::Options {
        return Ok(Response::builder()
            .status(200)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .header("Content-Type", "application/json")
            .body("")
            .build());
    }

    // Only handle POST requests for JSON-RPC
    if *req.method() != Method::Post {
        return Ok(Response::builder()
            .status(405)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .header("Content-Type", "application/json")
            .body("Method not allowed")
            .build());
    }

    // Parse the JSON-RPC request
    match read_request_body(&req) {
        Ok(body_str) => match serde_json::from_str::<JsonRpcRequest>(&body_str) {
            Ok(json_req) => {
                let response_data = server.handle_request(json_req);
                let response_json = serde_json::to_string(&response_data)
                    .map_err(|e| format!("Failed to serialize response: {e}"))?;

                Ok(Response::builder()
                    .status(200)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                    .header("Access-Control-Allow-Headers", "Content-Type")
                    .header("Content-Type", "application/json")
                    .body(response_json)
                    .build())
            }
            Err(e) => {
                eprintln!("Failed to parse JSON-RPC request: {e}");
                let error_response =
                    crate::types::JsonRpcResponse::error(None, -32700, "Parse error");

                let response_json = serde_json::to_string(&error_response)
                    .map_err(|e| format!("Failed to serialize error response: {e}"))?;
                Ok(Response::builder()
                    .status(400)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                    .header("Access-Control-Allow-Headers", "Content-Type")
                    .header("Content-Type", "application/json")
                    .body(response_json)
                    .build())
            }
        },
        Err(e) => {
            eprintln!("Failed to read request body: {e}");
            let error_response =
                crate::types::JsonRpcResponse::error(None, -32700, "Failed to read request body");

            let response_json = serde_json::to_string(&error_response)
                .map_err(|e| format!("Failed to serialize error response: {e}"))?;
            Ok(Response::builder()
                .status(400)
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .header("Content-Type", "application/json")
                .body(response_json)
                .build())
        }
    }
}

fn read_request_body(req: &Request) -> Result<String, String> {
    String::from_utf8(req.body().to_vec())
        .map_err(|e| format!("Failed to parse request body as UTF-8: {e}"))
}

/// Macro to create the main entry point for a tool server
///
/// Example:
/// ```rust
/// use ftl_sdk::prelude::*;
///
/// #[derive(Clone)]
/// struct MyTool;
///
/// impl Tool for MyTool {
///     fn name(&self) -> &'static str {
///         "my-tool"
///     }
///     fn description(&self) -> &'static str {
///         "Example tool"
///     }
///     fn input_schema(&self) -> serde_json::Value {
///         serde_json::json!({})
///     }
///     fn call(&self, _args: &serde_json::Value) -> Result<ToolResult, ToolError> {
///         Ok(ToolResult::text("Hello".to_string()))
///     }
/// }
///
/// // Then use the macro:
/// // ftl_mcp_server!(MyTool);
/// ```
#[macro_export]
macro_rules! ftl_mcp_server {
    ($tool:expr) => {
        #[spin_sdk::http_component]
        fn handle_mcp_request(req: spin_sdk::http::Request) -> spin_sdk::http::Response {
            let server = $crate::McpServer::new($tool);

            // Helper function to create error responses
            let create_error_response = |status: u16, message: &str| {
                spin_sdk::http::Response::builder()
                    .status(status)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                    .header("Access-Control-Allow-Headers", "Content-Type")
                    .header("Content-Type", "application/json")
                    .body(message)
                    .build()
            };

            match req.method() {
                &spin_sdk::http::Method::Options => spin_sdk::http::Response::builder()
                    .status(200)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                    .header("Access-Control-Allow-Headers", "Content-Type")
                    .header("Content-Type", "application/json")
                    .body("")
                    .build(),
                &spin_sdk::http::Method::Post => {
                    let body_str = match String::from_utf8(req.body().to_vec()) {
                        Ok(s) => s,
                        Err(_) => {
                            return create_error_response(400, "Invalid UTF-8 in request body");
                        }
                    };

                    match serde_json::from_str::<$crate::JsonRpcRequest>(&body_str) {
                        Ok(json_req) => {
                            let response_data = server.handle_request(json_req);
                            match serde_json::to_string(&response_data) {
                                Ok(response_json) => spin_sdk::http::Response::builder()
                                    .status(200)
                                    .header("Access-Control-Allow-Origin", "*")
                                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                                    .header("Access-Control-Allow-Headers", "Content-Type")
                                    .header("Content-Type", "application/json")
                                    .body(response_json)
                                    .build(),
                                Err(_) => {
                                    create_error_response(500, "Failed to serialize response")
                                }
                            }
                        }
                        Err(_) => {
                            let error_response =
                                $crate::JsonRpcResponse::error(None, -32700, "Parse error");
                            match serde_json::to_string(&error_response) {
                                Ok(response_json) => spin_sdk::http::Response::builder()
                                    .status(400)
                                    .header("Access-Control-Allow-Origin", "*")
                                    .header("Access-Control-Allow-Methods", "POST, OPTIONS")
                                    .header("Access-Control-Allow-Headers", "Content-Type")
                                    .header("Content-Type", "application/json")
                                    .body(response_json)
                                    .build(),
                                Err(_) => {
                                    create_error_response(500, "Failed to serialize error response")
                                }
                            }
                        }
                    }
                }
                _ => create_error_response(405, "Method not allowed"),
            }
        }
    };
}
