//! MCP Authorizer - A high-performance JWT authentication gateway for MCP servers
//!
//! This component implements OAuth 2.0 Bearer Token authentication with JWKS support,
//! providing a secure gateway to MCP (Model Context Protocol) servers.

use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::key_value::Store;

mod auth;
mod config;
mod discovery;
mod error;
mod forwarding;
mod jwks;
mod policy;
mod token;

use config::{Config, PolicyAuthorization};
use error::{AuthError, Result};
use policy::PolicyEngine;

/// Main HTTP component handler
#[spin_sdk::http_component]
async fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle CORS preflight requests immediately
    if *req.method() == spin_sdk::http::Method::Options {
        return Ok(create_cors_response());
    }

    // Load configuration and handle errors properly
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Configuration failed: {e}");
            // Return configuration error as a proper HTTP response
            let error = AuthError::Configuration(format!("Configuration error: {e}"));
            return Ok(create_config_error_response(&error));
        }
    };

    // Extract trace ID for request tracking
    let trace_id = extract_trace_id(&req, &config.trace_header);

    // Log request details for debugging
    if let Some(ref id) = trace_id {
        eprintln!("[{}] {} {}", id, req.method(), req.path());
    } else {
        eprintln!("{} {}", req.method(), req.path());
    }

    // Handle OAuth discovery endpoints (no auth required)
    if let Some(response) = handle_discovery(&req, &config, trace_id.as_ref()) {
        return Ok(response);
    }

    // For POST requests with JSON content, capture the body for potential policy evaluation
    // The policy decides whether to use this information
    let body_bytes = if *req.method() == spin_sdk::http::Method::Post {
        // Check if it's likely JSON content
        let is_json = req.headers()
            .any(|(name, value)| {
                name.eq_ignore_ascii_case("content-type") &&
                value.as_str().map_or(false, |v| v.contains("json"))
            });
        
        if is_json {
            Some(req.body().to_vec())
        } else {
            None
        }
    } else {
        None
    };

    // Authentication is always required for an auth gateway
    // The presence of a provider configuration determines the auth method
    match authenticate_with_policy(&req, &config, body_bytes.as_deref()).await {
        Ok(auth_context) => {
            // Only forward if gateway URL is configured and valid
            // This allows tests to run without forwarding
            if !config.gateway_url.is_empty() && config.gateway_url != "none" {
                // Reconstruct request with body if we consumed it
                let req_to_forward = if let Some(body) = body_bytes {
                    // Collect headers first
                    let headers: Vec<(String, String)> = req.headers()
                        .map(|(name, value)| (name.to_string(), value.as_str().unwrap_or("").to_string()))
                        .collect();
                    
                    // Create a new request with headers and body
                    Request::builder()
                        .method(req.method().clone())
                        .uri(req.uri())
                        .headers(headers)
                        .body(body)
                        .build()
                } else {
                    req
                };
                forward_request(req_to_forward, &config, auth_context, trace_id).await
            } else {
                // No gateway configured - return success directly (for testing)
                Ok(Response::new(200, "OK"))
            }
        }
        Err(auth_error) => {
            // Log auth failures for debugging
            if let Some(ref id) = trace_id {
                eprintln!("[{id}] Auth failed: {auth_error}");
            } else {
                eprintln!("Auth failed: {auth_error}");
            }
            // Return authentication error
            Ok(create_error_response(&auth_error, &req, &config, trace_id))
        }
    }
}

/// Authenticate the incoming request with policy-based authorization
async fn authenticate_with_policy(
    req: &Request,
    config: &Config,
    body: Option<&[u8]>,
) -> Result<auth::Context> {
    // Extract bearer token
    let token = auth::extract_bearer_token(req)?;

    // Provider must exist for authentication
    let provider = config.provider.as_ref().ok_or_else(|| {
        AuthError::Unauthorized("No authentication provider configured".to_string())
    })?;

    // Verify token using JWT provider
    let token_info = match provider {
        config::Provider::Jwt(jwt_provider) => {
            // Open KV store for JWKS caching
            let store = Store::open_default()
                .map_err(|e| {
                    eprintln!("ERROR: Failed to open KV store: {e}");
                    eprintln!("HINT: Ensure the mcp-authorizer component has 'key_value_stores = [\"default\"]' in spin.toml");
                    AuthError::Internal("KV store access denied. Ensure component has key_value_stores permission in spin.toml".to_string())
                })?;

            // Verify JWT token (signature, expiry, issuer, audience)
            token::verify(token, jwt_provider, &store).await?
        }
    };

    // Apply policy-based authorization if configured
    if let Some(policy_config) = &config.authorization {
        apply_policy_authorization(&token_info, req, body, policy_config)?;
    }

    // Build auth context with all available claims
    Ok(auth::Context {
        client_id: token_info.client_id,
        user_id: token_info.sub.clone(),
        scopes: token_info.scopes,
        issuer: token_info.iss,
        raw_token: token.to_string(),
        additional_claims: token_info.claims,
    })
}

/// Apply policy-based authorization using Regorous
fn apply_policy_authorization(
    token_info: &token::TokenInfo,
    req: &Request,
    body: Option<&[u8]>,
    policy_config: &PolicyAuthorization,
) -> Result<()> {
    // Create policy engine with the configured policy and data
    let mut engine = PolicyEngine::new_with_policy_and_data(
        &policy_config.policy,
        policy_config.data.as_deref(),
    )
    .map_err(|e| AuthError::Configuration(format!("Failed to initialize policy engine: {e}")))?;
    
    // Evaluate policy
    let allowed = engine.evaluate(token_info, req, body)?;
    
    if !allowed {
        return Err(AuthError::Unauthorized(
            "Access denied by authorization policy".to_string(),
        ));
    }
    
    Ok(())
}

/// Handle OAuth discovery endpoints
fn handle_discovery(req: &Request, config: &Config, trace_id: Option<&String>) -> Option<Response> {
    let path = req.path();

    // Handle discovery endpoints with or without path suffixes
    if path.starts_with("/.well-known/oauth-protected-resource") {
        Some(discovery::oauth_protected_resource(req, config, trace_id))
    } else if path.starts_with("/.well-known/oauth-authorization-server") {
        Some(discovery::oauth_authorization_server(req, config, trace_id))
    } else if path.starts_with("/.well-known/openid-configuration") {
        Some(discovery::openid_configuration(req, config, trace_id))
    } else {
        None
    }
}

/// Forward request to the MCP gateway
async fn forward_request(
    req: Request,
    config: &Config,
    auth_context: auth::Context,
    trace_id: Option<String>,
) -> anyhow::Result<Response> {
    forwarding::forward_to_gateway(req, config, auth_context, trace_id).await
}

/// Create authentication error response
fn create_error_response(
    error: &AuthError,
    req: &Request,
    config: &Config,
    trace_id: Option<String>,
) -> Response {
    let (status, error_code, description) = match error {
        AuthError::Unauthorized(msg) => (401, "unauthorized", msg.as_str()),
        AuthError::InvalidToken(msg) => (401, "invalid_token", msg.as_str()),
        AuthError::ExpiredToken => (401, "invalid_token", "Token has expired"),
        AuthError::InvalidIssuer => (401, "invalid_token", "Invalid issuer"),
        AuthError::InvalidAudience => (401, "invalid_token", "Invalid audience"),
        AuthError::InvalidSignature => (401, "invalid_token", "Invalid signature"),
        AuthError::Configuration(msg) | AuthError::Internal(msg) => {
            (500, "server_error", msg.as_str())
        }
    };

    // Build JSON error body
    let body = serde_json::json!({
        "error": error_code,
        "error_description": description
    });

    // Build response with appropriate headers
    let mut binding = Response::builder();
    let status_u16 = u16::try_from(status).unwrap_or(500);
    let mut builder = binding.status(status_u16);

    // Add common headers
    let cors_headers = [
        ("content-type", "application/json"),
        ("access-control-allow-origin", "*"),
        (
            "access-control-allow-methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        ),
        (
            "access-control-allow-headers",
            "Content-Type, Authorization",
        ),
    ];

    for (key, value) in cors_headers {
        builder = builder.header(key, value);
    }

    // Add WWW-Authenticate header for 401 responses
    if status == 401 {
        let www_auth = format!(r#"Bearer error="{error_code}", error_description="{description}""#);

        // Add resource metadata if we have a host
        let www_auth_value = if let Some(host) = extract_host(req) {
            // Use http for local development (localhost/127.0.0.1)
            let scheme = if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
                "http"
            } else {
                "https"
            };
            let resource_url = format!("{scheme}://{host}/.well-known/oauth-protected-resource");
            format!("{www_auth}, resource_metadata=\"{resource_url}\"")
        } else {
            www_auth
        };

        builder = builder.header("www-authenticate", www_auth_value);
    }

    // Add trace header if present
    if let Some(trace_id) = trace_id {
        builder = builder.header(&config.trace_header, trace_id);
    }

    builder.body(body.to_string()).build()
}

/// Create CORS preflight response
fn create_cors_response() -> Response {
    let headers = [
        ("access-control-allow-origin", "*"),
        (
            "access-control-allow-methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        ),
        (
            "access-control-allow-headers",
            "Content-Type, Authorization",
        ),
        ("access-control-max-age", "86400"),
    ];

    let mut binding = Response::builder();
    let mut builder = binding.status(204);

    for (key, value) in headers {
        builder = builder.header(key, value);
    }

    builder.build()
}

/// Extract trace ID from request headers
fn extract_trace_id(req: &Request, trace_header: &str) -> Option<String> {
    req.headers()
        .find(|(name, _)| name.eq_ignore_ascii_case(trace_header))
        .and_then(|(_, value)| value.as_str())
        .map(String::from)
}

/// Extract host from request headers
fn extract_host(req: &Request) -> Option<String> {
    req.headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("host"))
        .and_then(|(_, value)| value.as_str())
        .map(String::from)
        .or_else(|| {
            req.headers()
                .find(|(name, _)| name.eq_ignore_ascii_case("x-forwarded-host"))
                .and_then(|(_, value)| value.as_str())
                .map(String::from)
        })
}

/// Create configuration error response (simpler version without request context)
fn create_config_error_response(error: &AuthError) -> Response {
    let body = serde_json::json!({
        "error": "server_error",
        "error_description": error.to_string()
    });

    Response::builder()
        .status(500)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .header(
            "access-control-allow-methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .header(
            "access-control-allow-headers",
            "Content-Type, Authorization",
        )
        .body(body.to_string())
        .build()
}
