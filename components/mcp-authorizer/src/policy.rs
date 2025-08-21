//! Policy-based authorization using Regorous (Rego interpreter)

use anyhow::{Result, anyhow};
use regorus::{Engine, Value};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::Request;
use std::collections::HashMap;

use crate::error::AuthError;
use crate::token::TokenInfo;

/// Policy engine for authorization decisions
pub struct PolicyEngine {
    engine: Engine,
}

/// MCP context extracted from request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    /// JSON-RPC method (e.g., "tools/call", "tools/list")
    pub method: String,
    /// Tool name for tools/call requests
    pub tool: Option<String>,
    /// Tool arguments if present
    pub arguments: Option<serde_json::Value>,
}

/// JSON-RPC request structure for MCP
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

impl PolicyEngine {
    /// Create a new policy engine with policy and data
    pub fn new_with_policy_and_data(policy: &str, data: Option<&str>) -> Result<Self> {
        let mut engine = Engine::new();

        // Add the policy
        log::debug!("Loading policy ({} bytes)", policy.len());
        engine
            .add_policy("authorization.rego".to_string(), policy.to_string())
            .map_err(|e| {
                log::error!("Failed to parse policy - {e}");
                log::error!("This is likely a syntax error in your Rego policy");
                anyhow!("Failed to add policy: {e}")
            })?;
        log::debug!("Policy loaded successfully");

        // Add data if provided
        if let Some(data_json) = data {
            log::debug!("Loading external data ({} bytes)", data_json.len());
            let data_value = Value::from_json_str(data_json).map_err(|e| {
                log::error!("Failed to parse policy data - {e}");
                log::error!("Check that your policy data is valid JSON");
                anyhow!("Failed to parse policy data: {e}")
            })?;
            engine.add_data(data_value).map_err(|e| {
                log::error!("Failed to add policy data - {e}");
                anyhow!("Failed to add policy data: {e}")
            })?;
            log::debug!("External data loaded successfully");
        }

        Ok(Self { engine })
    }

    /// Evaluate authorization policy
    pub fn evaluate(
        &mut self,
        token_info: &TokenInfo,
        req: &Request,
        body: Option<&[u8]>,
    ) -> Result<bool, AuthError> {
        // Build the input for policy evaluation
        let input = Self::build_policy_input(token_info, req, body)?;

        // Set the input
        self.engine.set_input(input);

        // Evaluate the allow rule
        log::trace!("Evaluating authorization rule: data.mcp.authorization.allow");
        match self
            .engine
            .eval_rule("data.mcp.authorization.allow".to_string())
        {
            Ok(value) => {
                // Check if the result is a boolean true
                match value {
                    Value::Bool(b) => {
                        log::debug!("Authorization result: {}", if b { "ALLOW" } else { "DENY" });
                        Ok(b)
                    }
                    Value::Undefined => {
                        log::debug!("Authorization result: UNDEFINED (treating as DENY)");
                        log::info!("Policy may not have an 'allow' rule defined");
                        Ok(false) // Undefined means not allowed
                    }
                    _ => {
                        log::error!("Policy returned non-boolean value: {value:?}");
                        Err(AuthError::Internal(
                            "Policy returned non-boolean value".to_string(),
                        ))
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to evaluate policy rule - {e}");
                log::error!("This may indicate a runtime error in your Rego policy");
                Err(AuthError::Internal(format!(
                    "Policy evaluation failed: {e}"
                )))
            }
        }
    }

    /// Build policy input from request context
    fn build_policy_input(
        token_info: &TokenInfo,
        req: &Request,
        body: Option<&[u8]>,
    ) -> Result<Value, AuthError> {
        // Extract component from path
        let component = extract_component_from_path(req.path());

        // Build base input
        let mut input = json!({
            "token": {
                "sub": token_info.sub,
                "iss": token_info.iss,
                "claims": token_info.claims,
                "scopes": token_info.scopes
            },
            "request": {
                "method": req.method().to_string(),
                "path": req.path(),
                "component": component,
                "headers": headers_to_json(req.headers())
            }
        });

        // Always try to add MCP context if we have a body
        // The policy decides whether to use this information
        if let Some(body_bytes) = body
            && let Ok(mcp_context) = parse_mcp_request(body_bytes)
            && let Some(obj) = input.as_object_mut()
        {
            obj.insert("mcp".to_string(), mcp_context);
        }
        // If parsing fails, we just don't add the mcp field
        // This allows policies to handle both MCP and non-MCP requests

        Value::from_json_str(&input.to_string())
            .map_err(|e| AuthError::Internal(format!("Failed to build policy input: {e}")))
    }
}

/// Extract component name from MCP path
/// Handles:
/// - `/mcp` -> None (all components)
/// - `/mcp/x/{component}` -> Some(component)
/// - `/mcp/x/{component}/readonly` -> Some(component)
fn extract_component_from_path(path: &str) -> Option<String> {
    let path = path.trim_start_matches('/').trim_end_matches('/');

    // Check for component scoping pattern
    if let Some(remaining) = path.strip_prefix("mcp/x/") {
        let parts: Vec<&str> = remaining.split('/').collect();
        if let Some(component) = parts.first()
            && !component.is_empty()
        {
            return Some((*component).to_string());
        }
    }

    None
}

/// Convert headers to JSON for policy input
fn headers_to_json<'a>(
    headers: impl Iterator<Item = (&'a str, &'a spin_sdk::http::HeaderValue)>,
) -> serde_json::Value {
    let mut header_map = HashMap::new();

    for (name, value) in headers {
        if let Some(value_str) = value.as_str() {
            header_map.insert(name.to_string(), value_str.to_string());
        }
    }

    serde_json::to_value(header_map).unwrap_or_else(|_| json!({}))
}

/// Parse MCP request from body
fn parse_mcp_request(body: &[u8]) -> Result<serde_json::Value> {
    let json_rpc: JsonRpcRequest = serde_json::from_slice(body)?;

    let mcp_context = match json_rpc.method.as_str() {
        "tools/call" => {
            // Extract tool name and arguments from params
            json_rpc.params.map_or_else(
                || {
                    json!({
                        "method": "tools/call",
                        "tool": null,
                        "arguments": null
                    })
                },
                |params| {
                    json!({
                        "method": "tools/call",
                        "tool": params.get("name"),
                        "arguments": params.get("arguments")
                    })
                },
            )
        }
        "tools/list" => {
            json!({ "method": "tools/list" })
        }
        "prompts/list" => {
            json!({ "method": "prompts/list" })
        }
        "resources/list" => {
            json!({ "method": "resources/list" })
        }
        method => {
            // Pass through other methods
            json!({ "method": method })
        }
    };

    Ok(mcp_context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_component_from_path() {
        assert_eq!(extract_component_from_path("/mcp"), None);
        assert_eq!(extract_component_from_path("/mcp/"), None);
        assert_eq!(
            extract_component_from_path("/mcp/x/data-processor"),
            Some("data-processor".to_string())
        );
        assert_eq!(
            extract_component_from_path("/mcp/x/data-processor/"),
            Some("data-processor".to_string())
        );
        assert_eq!(
            extract_component_from_path("/mcp/x/data-processor/readonly"),
            Some("data-processor".to_string())
        );
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_parse_mcp_request_tools_call() {
        let body = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "delete_database",
                "arguments": {"database": "test"}
            }
        }"#;

        let result = parse_mcp_request(body.as_bytes()).expect("should parse valid JSON-RPC");
        assert_eq!(result.get("method"), Some(&json!("tools/call")));
        assert_eq!(result.get("tool"), Some(&json!("delete_database")));
        assert_eq!(
            result.get("arguments").and_then(|a| a.get("database")),
            Some(&json!("test"))
        );
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_parse_mcp_request_tools_list() {
        let body = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }"#;

        let result = parse_mcp_request(body.as_bytes()).expect("should parse valid JSON-RPC");
        assert_eq!(result.get("method"), Some(&json!("tools/list")));
    }
}
