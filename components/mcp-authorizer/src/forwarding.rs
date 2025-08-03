//! Request forwarding to the MCP gateway

use spin_sdk::http::{Request, Response};

use crate::auth::Context as AuthContext;
use crate::config::Config;

/// Forward request to the MCP gateway
pub async fn forward_to_gateway(
    req: Request,
    config: &Config,
    auth_context: AuthContext,
    trace_id: Option<String>,
) -> anyhow::Result<Response> {
    // Parse gateway URL to set the scheme and authority
    let gateway_url = url::Url::parse(&config.gateway_url)?;
    
    // Create headers first
    let headers = spin_sdk::http::Headers::new();
    
    // Copy request headers
    for (name, value) in req.headers() {
        headers.append(&name.to_string(), &value.as_bytes().to_vec())?;
    }
    
    // Add authentication context headers
    headers.append(&"x-auth-client-id".to_string(), &auth_context.client_id.as_bytes().to_vec())?;
    headers.append(&"x-auth-user-id".to_string(), &auth_context.user_id.as_bytes().to_vec())?;
    headers.append(&"x-auth-issuer".to_string(), &auth_context.issuer.as_bytes().to_vec())?;
    
    if !auth_context.scopes.is_empty() {
        headers.append(&"x-auth-scopes".to_string(), &auth_context.scopes.join(" ").as_bytes().to_vec())?;
    }
    
    // Forward the original authorization header
    headers.append(&"authorization".to_string(), &format!("Bearer {}", auth_context.raw_token).as_bytes().to_vec())?;
    
    // Add trace ID if present
    if let Some(trace_id) = &trace_id {
        headers.append(&config.trace_header, &trace_id.as_bytes().to_vec())?;
    }
    
    // Build outgoing request with the headers
    let outgoing = spin_sdk::http::OutgoingRequest::new(headers);
    
    // Set method and path
    outgoing.set_method(req.method()).map_err(|_| anyhow::anyhow!("Failed to set method"))?;
    
    // Set scheme based on the gateway URL
    let scheme = if gateway_url.scheme() == "https" {
        spin_sdk::http::Scheme::Https
    } else {
        spin_sdk::http::Scheme::Http
    };
    outgoing.set_scheme(Some(&scheme)).map_err(|_| anyhow::anyhow!("Failed to set scheme"))?;
    
    if let Some(host) = gateway_url.host_str() {
        let authority = if let Some(port) = gateway_url.port() {
            format!("{}:{}", host, port)
        } else {
            host.to_string()
        };
        outgoing.set_authority(Some(&authority)).map_err(|_| anyhow::anyhow!("Failed to set authority"))?;
    }
    
    // Use the gateway URL's path, not the incoming request's path
    let gateway_path = gateway_url.path();
    // For simplicity, we'll just use the gateway path without preserving query strings
    // since MCP requests typically don't use query strings
    outgoing.set_path_with_query(Some(gateway_path)).map_err(|_| anyhow::anyhow!("Failed to set path"))?;
    
    // Transfer request body
    let body_bytes = req.into_body();
    if !body_bytes.is_empty() {
        use futures::SinkExt;
        let mut outgoing_body = outgoing.take_body();
        outgoing_body.send(body_bytes).await
            .map_err(|e| anyhow::anyhow!("Failed to send body: {:?}", e))?;
    }
    
    // Send request
    let incoming_response: spin_sdk::http::Response = spin_sdk::http::send(outgoing).await?;
    
    // Extract status
    let status = *incoming_response.status();
    
    // Collect headers from the gateway response
    let mut headers_vec: Vec<(String, String)> = Vec::new();
    for (name, value) in incoming_response.headers() {
        if let Ok(value_str) = std::str::from_utf8(value.as_bytes()) {
            // Skip certain headers that we'll override
            if !name.eq_ignore_ascii_case("access-control-allow-origin") &&
               !name.eq_ignore_ascii_case("access-control-allow-methods") &&
               !name.eq_ignore_ascii_case("access-control-allow-headers") {
                headers_vec.push((name.to_string(), value_str.to_string()));
            }
        }
    }
    
    // Extract body
    let body = incoming_response.into_body();
    
    // Build the final response
    let mut response_builder = Response::builder();
    response_builder.status(status);
    
    // Add gateway response headers first
    for (name, value) in headers_vec {
        response_builder.header(name, value);
    }
    
    // Add/override CORS headers
    response_builder
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization");
    
    // Add trace ID if present
    if let Some(trace_id) = trace_id {
        response_builder.header(&config.trace_header, trace_id);
    }
    
    // Build the response with body
    Ok(response_builder.body(body).build())
}