//! Request forwarding to the MCP gateway

use spin_sdk::http::{Headers, Request, Response};

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

    // Build headers with authentication context
    let headers = build_forwarding_headers(&req, &auth_context, trace_id.as_ref(), config)?;

    // Build outgoing request with the headers
    let outgoing = spin_sdk::http::OutgoingRequest::new(headers);

    // Set method and path
    outgoing
        .set_method(req.method())
        .map_err(|()| anyhow::anyhow!("Failed to set method"))?;

    // Set scheme based on the gateway URL
    let scheme = if gateway_url.scheme() == "https" {
        spin_sdk::http::Scheme::Https
    } else {
        spin_sdk::http::Scheme::Http
    };
    outgoing
        .set_scheme(Some(&scheme))
        .map_err(|()| anyhow::anyhow!("Failed to set scheme"))?;

    if let Some(host) = gateway_url.host_str() {
        let authority = gateway_url
            .port()
            .map_or_else(|| host.to_string(), |port| format!("{host}:{port}"));
        outgoing
            .set_authority(Some(&authority))
            .map_err(|()| anyhow::anyhow!("Failed to set authority"))?;
    }

    // Preserve the incoming request's path and append it to the gateway URL's base path
    let incoming_path = req.path();
    let gateway_base_path = gateway_url.path();

    // Combine the gateway base path with the incoming request path
    // If gateway base path ends with '/' and incoming path starts with '/', avoid double slash
    let combined_path = if gateway_base_path.ends_with('/') && incoming_path.starts_with('/') {
        format!("{}{}", gateway_base_path, &incoming_path[1..])
    } else if !gateway_base_path.ends_with('/') && !incoming_path.starts_with('/') {
        format!("{gateway_base_path}/{incoming_path}")
    } else {
        format!("{gateway_base_path}{incoming_path}")
    };

    // Preserve query string if present
    let query = req.query();
    let path_with_query = if query.is_empty() {
        combined_path
    } else {
        format!("{combined_path}?{query}")
    };

    outgoing
        .set_path_with_query(Some(&path_with_query))
        .map_err(|()| anyhow::anyhow!("Failed to set path"))?;

    // Transfer request body
    let body_bytes = req.into_body();
    if !body_bytes.is_empty() {
        use futures::SinkExt;
        let mut outgoing_body = outgoing.take_body();
        outgoing_body
            .send(body_bytes)
            .await
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
            if !name.eq_ignore_ascii_case("access-control-allow-origin")
                && !name.eq_ignore_ascii_case("access-control-allow-methods")
                && !name.eq_ignore_ascii_case("access-control-allow-headers")
            {
                headers_vec.push((name.to_string(), value_str.to_string()));
            }
        }
    }

    // Extract body
    let body = incoming_response.into_body();

    // Build response with proper headers
    Ok(build_gateway_response(
        status,
        headers_vec,
        body,
        trace_id,
        &config.trace_header,
    ))
}

/// Build headers for forwarding request
fn build_forwarding_headers(
    req: &Request,
    auth_context: &AuthContext,
    trace_id: Option<&String>,
    config: &Config,
) -> anyhow::Result<Headers> {
    let headers = Headers::new();

    // Copy request headers
    for (name, value) in req.headers() {
        headers.append(&name.to_string(), &value.as_bytes().to_vec())?;
    }

    // Add standard authentication context headers
    headers.append(
        &"x-auth-client-id".to_string(),
        &auth_context.client_id.as_bytes().to_vec(),
    )?;
    headers.append(
        &"x-auth-user-id".to_string(),
        &auth_context.user_id.as_bytes().to_vec(),
    )?;
    headers.append(
        &"x-auth-issuer".to_string(),
        &auth_context.issuer.as_bytes().to_vec(),
    )?;

    if !auth_context.scopes.is_empty() {
        headers.append(
            &"x-auth-scopes".to_string(),
            &auth_context.scopes.join(" ").as_bytes().to_vec(),
        )?;
    }

    // Note: Claim forwarding has been removed in favor of policy-based authorization
    // If specific claims need to be forwarded, they should be added as explicit headers
    // in the policy evaluation result or as part of the auth context

    // Forward the original authorization header
    headers.append(
        &"authorization".to_string(),
        &format!("Bearer {}", auth_context.raw_token)
            .as_bytes()
            .to_vec(),
    )?;

    // Add trace ID if present
    if let Some(trace_id) = trace_id {
        headers.append(&config.trace_header, &trace_id.as_bytes().to_vec())?;
    }

    Ok(headers)
}

/// Build gateway response with CORS headers
fn build_gateway_response(
    status: u16,
    headers_vec: Vec<(String, String)>,
    body: Vec<u8>,
    trace_id: Option<String>,
    trace_header: &str,
) -> Response {
    let mut binding = Response::builder();
    let mut response_builder = binding.status(status);

    // Add gateway response headers first
    for (name, value) in headers_vec {
        response_builder = response_builder.header(&name, &value);
    }

    // Add/override CORS headers
    response_builder = response_builder
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        );

    // Add trace ID if present
    if let Some(trace_id) = trace_id {
        response_builder = response_builder.header(trace_header, trace_id);
    }

    // Build the response with body
    response_builder.body(body).build()
}
