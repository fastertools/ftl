use anyhow::Result;
use serde_json::Value;
use spin_sdk::http::{Request, Response};

use crate::providers::UserContext;

/// Forward requests to the MCP gateway
pub async fn forward_to_mcp_gateway(
    req: Request,
    mcp_gateway_url: &str,
    user_context: Option<UserContext>,
    trace_id: &str,
) -> Result<Response> {
    // Parse the request body
    let body = req.body();
    let mut request_data: Value = if body.is_empty() {
        serde_json::json!(null)
    } else {
        serde_json::from_slice(body)?
    };

    // If this is an initialize request and we have auth context, inject user info
    if let Some(user) = &user_context {
        if let Some(obj) = request_data.as_object_mut() {
            if let Some(method) = obj.get("method").and_then(|m| m.as_str()) {
                if method == "initialize" {
                    // Add user context to the request
                    if let Some(params) = obj.get_mut("params").and_then(|p| p.as_object_mut()) {
                        params.insert(
                            "_authContext".to_string(),
                            serde_json::json!({
                                "authenticated_user": user.id,
                                "email": user.email,
                                "provider": user.provider,
                            }),
                        );
                    }
                }
            }
        }
    }

    // Build the request to forward
    let forward_body = if body.is_empty() {
        body.to_vec()
    } else if request_data == serde_json::json!(null) {
        body.to_vec()
    } else {
        serde_json::to_vec(&request_data)?
    };

    let forward_req = Request::builder()
        .method(req.method().clone())
        .uri(mcp_gateway_url)
        .header("Content-Type", "application/json")
        .header("X-Trace-Id", trace_id)
        .body(forward_body)
        .build();

    // Forward the request
    let resp: Response = spin_sdk::http::send(forward_req).await?;

    // Parse the response to potentially inject auth info
    let resp_body = resp.body();
    let mut response_data: Value = if resp_body.is_empty() {
        serde_json::json!({})
    } else {
        match serde_json::from_slice(resp_body) {
            Ok(data) => data,
            Err(_) => {
                // If we can't parse, just return as-is
                return Ok(Response::builder()
                    .status(*resp.status())
                    .body(resp_body.to_vec())
                    .build());
            }
        }
    };

    // If this is an initialize response and we have auth context, inject auth info
    if let Some(user) = &user_context {
        if let Some(result) = response_data
            .as_object_mut()
            .and_then(|obj| obj.get_mut("result"))
            .and_then(|r| r.as_object_mut())
        {
            if let Some(server_info) = result
                .get_mut("serverInfo")
                .and_then(|si| si.as_object_mut())
            {
                server_info.insert(
                    "authInfo".to_string(),
                    serde_json::json!({
                        "authenticated_user": user.id,
                        "email": user.email,
                        "provider": user.provider,
                    }),
                );
            }
        }
    }

    // Build the response
    if response_data == serde_json::json!(null) || resp_body.is_empty() {
        Ok(Response::builder()
            .status(*resp.status())
            .body(resp_body.to_vec())
            .build())
    } else {
        Ok(Response::builder()
            .status(*resp.status())
            .header("Content-Type", "application/json")
            .header("X-Trace-Id", trace_id)
            .body(serde_json::to_string(&response_data)?)
            .build())
    }
}