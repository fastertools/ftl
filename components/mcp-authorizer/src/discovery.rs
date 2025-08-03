//! OAuth 2.0 discovery endpoints implementation

use spin_sdk::http::{Request, Response};
use serde_json::json;

use crate::config::Config;

/// Handle OAuth protected resource metadata endpoint
pub fn oauth_protected_resource(req: &Request, config: &Config, trace_id: &Option<String>) -> Response {
    let provider = &config.provider;
    
    // Build metadata
    let metadata = json!({
        "resource": extract_resource_url(req),
        "authorization_servers": [
            {
                "issuer": provider.issuer,
                "jwks_uri": provider.jwks_uri,
            }
        ],
        "authentication_methods": {
            "bearer": {
                "required": true,
                "algs_supported": ["RS256"],
            }
        },
    });
    
    build_success_response(metadata, trace_id, &config.trace_header)
}

/// Handle OAuth authorization server metadata endpoint
pub fn oauth_authorization_server(_req: &Request, config: &Config, trace_id: &Option<String>) -> Response {
    let provider = &config.provider;
    
    let oauth_endpoints = provider.oauth_endpoints.as_ref();
    
    // Build metadata
    let metadata = json!({
        "issuer": provider.issuer,
        "authorization_endpoint": oauth_endpoints.map(|e| &e.authorize),
        "token_endpoint": oauth_endpoints.map(|e| &e.token),
        "userinfo_endpoint": oauth_endpoints.and_then(|e| e.userinfo.as_ref()),
        "jwks_uri": provider.jwks_uri,
        "response_types_supported": ["code", "token", "id_token"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "profile", "email"],
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
        "claims_supported": ["sub", "iss", "aud", "exp", "iat", "scope", "client_id"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
    });
    
    build_success_response(metadata, trace_id, &config.trace_header)
}

/// Handle OpenID configuration endpoint
pub fn openid_configuration(_req: &Request, config: &Config, trace_id: &Option<String>) -> Response {
    // OpenID configuration is similar to OAuth authorization server metadata
    // but with some additional fields
    let provider = &config.provider;
    
    let oauth_endpoints = provider.oauth_endpoints.as_ref();
    
    // Build metadata
    let metadata = json!({
        "issuer": provider.issuer,
        "authorization_endpoint": oauth_endpoints.map(|e| &e.authorize),
        "token_endpoint": oauth_endpoints.map(|e| &e.token),
        "userinfo_endpoint": oauth_endpoints.and_then(|e| e.userinfo.as_ref()),
        "jwks_uri": provider.jwks_uri,
        "response_types_supported": ["code", "token", "id_token", "code id_token"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "profile", "email", "offline_access"],
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
        "claims_supported": [
            "sub", "iss", "aud", "exp", "iat", "auth_time",
            "nonce", "acr", "amr", "azp", "name", "given_name",
            "family_name", "middle_name", "nickname", "preferred_username",
            "profile", "picture", "website", "email", "email_verified",
            "gender", "birthdate", "zoneinfo", "locale", "phone_number",
            "phone_number_verified", "address", "updated_at"
        ],
        "grant_types_supported": ["authorization_code", "implicit", "refresh_token"],
        "acr_values_supported": [],
        "code_challenge_methods_supported": ["S256"],
    });
    
    build_success_response(metadata, trace_id, &config.trace_header)
}

/// Build a successful response with optional trace header
fn build_success_response(metadata: serde_json::Value, _trace_id: &Option<String>, _trace_header: &str) -> Response {
    let body = metadata.to_string();
    
    // Try a different approach - build response with body first, then add headers
    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .header("access-control-allow-methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("access-control-allow-headers", "Content-Type, Authorization")
        .body(body)
        .build();
    
    // If we have a trace header, we might need to rebuild with it
    // For now, return the response as-is to see if all headers are included
    response
}


/// Extract resource URL from request
fn extract_resource_url(req: &Request) -> String {
    let scheme = "https";
    let host = req.headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("host"))
        .and_then(|(_, value)| value.as_str())
        .unwrap_or("localhost");
    
    format!("{}://{}", scheme, host)
}