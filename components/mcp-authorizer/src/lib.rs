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
mod token;

use config::Config;
use error::{AuthError, Result};

/// Main HTTP component handler
#[spin_sdk::http_component]
async fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle CORS preflight requests immediately
    if *req.method() == spin_sdk::http::Method::Options {
        return Ok(create_cors_response());
    }

    // Load configuration
    let config = Config::load()?;

    // Extract trace ID for request tracking
    let trace_id = extract_trace_id(&req, &config.trace_header);

    // Handle OAuth discovery endpoints (no auth required)
    if let Some(response) = handle_discovery(&req, &config, trace_id.as_ref()) {
        return Ok(response);
    }

    // Authentication is always required for an auth gateway
    // The presence of a provider configuration determines the auth method
    match authenticate(&req, &config).await {
        Ok(auth_context) => {
            // Only forward if gateway URL is configured and valid
            // This allows tests to run without forwarding
            if !config.gateway_url.is_empty() && config.gateway_url != "none" {
                forward_request(req, &config, auth_context, trace_id).await
            } else {
                // No gateway configured - return success directly (for testing)
                Ok(Response::new(200, "OK"))
            }
        }
        Err(auth_error) => {
            // Return authentication error
            Ok(create_error_response(&auth_error, &req, &config, trace_id))
        }
    }
}

/// Authenticate the incoming request
async fn authenticate(req: &Request, config: &Config) -> Result<auth::Context> {
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
                .map_err(|e| AuthError::Internal(format!("Failed to open KV store: {e}")))?;

            // Verify JWT token (signature, expiry, issuer, audience)
            token::verify(token, jwt_provider, &store).await?
        }
    };

    // Check authorization rules if configured
    if let Some(authz_rules) = &config.authorization {
        apply_authorization_rules(&token_info, authz_rules)?;
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

/// Apply authorization rules if configured
fn apply_authorization_rules(
    token_info: &token::TokenInfo,
    rules: &config::AuthorizationRules,
) -> Result<()> {
    // Check allowed subjects
    if let Some(allowed_subjects) = &rules.allowed_subjects
        && !allowed_subjects.contains(&token_info.sub)
    {
        return Err(AuthError::Unauthorized(
            "Access denied: subject not authorized".to_string(),
        ));
    }

    // Check required claims
    if let Some(required_claims) = &rules.required_claims {
        for (claim_name, required_value) in required_claims {
            let token_value = token_info.claims.get(claim_name);

            match token_value {
                Some(value) if value == required_value => {}
                Some(_) => {
                    return Err(AuthError::Unauthorized(format!(
                        "Access denied: claim '{claim_name}' mismatch"
                    )));
                }
                None => {
                    return Err(AuthError::Unauthorized(format!(
                        "Access denied: missing required claim '{claim_name}'"
                    )));
                }
            }
        }
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
