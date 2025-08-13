//! OAuth 2.0 discovery endpoints implementation

use serde_json::json;
use spin_sdk::http::{Request, Response};

use crate::config::Config;

/// Get resource URLs based on the request
fn get_resource_urls(req: &Request) -> Vec<String> {
    // Try multiple header names for the host
    let host = req
        .headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("host"))
        .or_else(|| {
            req.headers()
                .find(|(name, _)| name.eq_ignore_ascii_case("x-forwarded-host"))
        })
        .or_else(|| {
            req.headers()
                .find(|(name, _)| name.eq_ignore_ascii_case("x-original-host"))
        })
        .and_then(|(_, value)| value.as_str());

    host.map_or_else(
        || {
            // Fallback to localhost for development
            vec![
                "http://localhost:3000/mcp".to_string(),
                "http://127.0.0.1:3000/mcp".to_string(),
            ]
        },
        |host_header| {
            // Determine scheme based on X-Forwarded-Proto or host
            let scheme = req
                .headers()
                .find(|(name, _)| name.eq_ignore_ascii_case("x-forwarded-proto"))
                .and_then(|(_, value)| value.as_str())
                .unwrap_or_else(|| {
                    if host_header.starts_with("localhost") || host_header.starts_with("127.0.0.1")
                    {
                        "http"
                    } else {
                        "https"
                    }
                });

            vec![format!("{scheme}://{host_header}/mcp")]
        },
    )
}

/// Handle OAuth protected resource metadata endpoint
pub fn oauth_protected_resource(
    req: &Request,
    config: &Config,
    trace_id: Option<&String>,
) -> Response {
    // Build metadata based on provider type
    let metadata = match &config.provider {
        Some(crate::config::Provider::Jwt(jwt_provider)) => {
            // For AuthKit domains, return simplified metadata pointing to AuthKit
            let authorization_servers = if !jwt_provider.issuer.is_empty()
                && (jwt_provider.issuer.contains(".authkit.app")
                    || jwt_provider.issuer.contains(".workos.com"))
            {
                // AuthKit: Just return the issuer as authorization server
                vec![json!(jwt_provider.issuer)]
            } else {
                // Non-AuthKit: Return full metadata
                vec![json!({
                    "issuer": jwt_provider.issuer,
                    "jwks_uri": jwt_provider.jwks_uri,
                })]
            };

            json!({
                "resource": get_resource_urls(req),
                "authorization_servers": authorization_servers,
                "bearer_methods_supported": ["header"],
                "authentication_methods": {
                    "bearer": {
                        "required": true,
                        "algs_supported": ["RS256"],
                    }
                },
            })
        }
        None => {
            // Public mode - no authentication required
            json!({
                "resource": get_resource_urls(req),
                "authorization_servers": [],
                "bearer_methods_supported": [],
                "authentication_methods": {},
            })
        }
    };

    build_success_response(&metadata, trace_id, &config.trace_header)
}

/// Handle OAuth authorization server metadata endpoint
pub fn oauth_authorization_server(
    _req: &Request,
    config: &Config,
    trace_id: Option<&String>,
) -> Response {
    // Build metadata based on provider type
    let metadata = match &config.provider {
        Some(crate::config::Provider::Jwt(jwt_provider)) => {
            // For AuthKit domains, return comprehensive metadata
            if !jwt_provider.issuer.is_empty()
                && (jwt_provider.issuer.contains(".authkit.app")
                    || jwt_provider.issuer.contains(".workos.com"))
            {
                // AuthKit metadata with all required endpoints
                let jwks_uri = jwt_provider.jwks_uri.as_ref().map_or_else(
                    || format!("{}/oauth2/jwks", jwt_provider.issuer),
                    std::clone::Clone::clone,
                );
                json!({
                    "issuer": jwt_provider.issuer,
                    "authorization_endpoint": format!("{}/oauth2/authorize", jwt_provider.issuer),
                    "token_endpoint": format!("{}/oauth2/token", jwt_provider.issuer),
                    "userinfo_endpoint": format!("{}/oauth2/userinfo", jwt_provider.issuer),
                    "jwks_uri": jwks_uri,
                    "registration_endpoint": format!("{}/oauth2/register", jwt_provider.issuer),
                    "introspection_endpoint": format!("{}/oauth2/introspection", jwt_provider.issuer),
                    "revocation_endpoint": format!("{}/oauth2/revoke", jwt_provider.issuer),
                    "response_types_supported": ["code"],
                    "response_modes_supported": ["query"],
                    "grant_types_supported": ["authorization_code", "refresh_token"],
                    "subject_types_supported": ["public"],
                    "id_token_signing_alg_values_supported": ["RS256"],
                    "scopes_supported": ["email", "offline_access", "openid", "profile"],
                    "token_endpoint_auth_methods_supported": [
                        "none",
                        "client_secret_post",
                        "client_secret_basic"
                    ],
                    "claims_supported": ["sub", "iss", "aud", "exp", "iat", "scope", "client_id"],
                    "code_challenge_methods_supported": ["S256"],
                })
            } else {
                // Non-AuthKit: Use configured endpoints or basic metadata
                let oauth_endpoints = jwt_provider.oauth_endpoints.as_ref();

                json!({
                    "issuer": jwt_provider.issuer,
                    "authorization_endpoint": oauth_endpoints.map(|e| &e.authorize),
                    "token_endpoint": oauth_endpoints.map(|e| &e.token),
                    "userinfo_endpoint": oauth_endpoints.and_then(|e| e.userinfo.as_ref()),
                    "jwks_uri": jwt_provider.jwks_uri,
                    "response_types_supported": ["code", "token", "id_token"],
                    "subject_types_supported": ["public"],
                    "id_token_signing_alg_values_supported": ["RS256"],
                    "scopes_supported": ["openid", "profile", "email"],
                    "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
                    "claims_supported": ["sub", "iss", "aud", "exp", "iat", "scope", "client_id"],
                    "grant_types_supported": ["authorization_code", "refresh_token"],
                })
            }
        }
        None => {
            // Public mode - no authorization server
            json!({
                "error": "not_supported",
                "error_description": "Public mode does not require OAuth authorization"
            })
        }
    };

    build_success_response(&metadata, trace_id, &config.trace_header)
}

/// Handle `OpenID` configuration endpoint
pub fn openid_configuration(
    _req: &Request,
    config: &Config,
    trace_id: Option<&String>,
) -> Response {
    // OpenID configuration is similar to OAuth authorization server metadata
    // but with some additional fields
    // Build metadata based on provider type
    let metadata = match &config.provider {
        Some(crate::config::Provider::Jwt(jwt_provider)) => {
            // For AuthKit, return AuthKit-specific OpenID metadata
            if !jwt_provider.issuer.is_empty()
                && (jwt_provider.issuer.contains(".authkit.app")
                    || jwt_provider.issuer.contains(".workos.com"))
            {
                // Match what AuthKit actually returns
                let jwks_uri = jwt_provider.jwks_uri.as_ref().map_or_else(
                    || format!("{}/oauth2/jwks", jwt_provider.issuer),
                    std::clone::Clone::clone,
                );
                json!({
                    "issuer": jwt_provider.issuer,
                    "authorization_endpoint": format!("{}/oauth2/authorize", jwt_provider.issuer),
                    "token_endpoint": format!("{}/oauth2/token", jwt_provider.issuer),
                    "userinfo_endpoint": format!("{}/oauth2/userinfo", jwt_provider.issuer),
                    "jwks_uri": jwks_uri,
                    "registration_endpoint": format!("{}/oauth2/register", jwt_provider.issuer),
                    "introspection_endpoint": format!("{}/oauth2/introspection", jwt_provider.issuer),
                    "revocation_endpoint": format!("{}/oauth2/revoke", jwt_provider.issuer),
                    "response_types_supported": ["code", "code id_token"],
                    "response_modes_supported": ["query"],
                    "grant_types_supported": ["authorization_code", "refresh_token"],
                    "subject_types_supported": ["public"],
                    "id_token_signing_alg_values_supported": ["RS256"],
                    "scopes_supported": ["email", "offline_access", "openid", "profile"],
                    "token_endpoint_auth_methods_supported": [
                        "none",
                        "client_secret_post",
                        "client_secret_basic"
                    ],
                    "claims_supported": [
                        "sub", "iss", "aud", "exp", "iat", "jti", "nonce",
                        "auth_time", "email", "email_verified", "name", "given_name",
                        "family_name", "picture", "locale", "updated_at"
                    ],
                    "code_challenge_methods_supported": ["S256"],
                    "ui_locales_supported": ["en"],
                })
            } else {
                // Non-AuthKit providers
                let oauth_endpoints = jwt_provider.oauth_endpoints.as_ref();

                json!({
                    "issuer": jwt_provider.issuer,
                    "authorization_endpoint": oauth_endpoints.map(|e| &e.authorize),
                    "token_endpoint": oauth_endpoints.map(|e| &e.token),
                    "userinfo_endpoint": oauth_endpoints.and_then(|e| e.userinfo.as_ref()),
                    "jwks_uri": jwt_provider.jwks_uri,
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
                })
            }
        }
        None => {
            // Public mode - no OpenID support
            json!({
                "error": "not_supported",
                "error_description": "Public mode does not require OpenID Connect"
            })
        }
    };

    build_success_response(&metadata, trace_id, &config.trace_header)
}

/// Build a successful response with optional trace header
fn build_success_response(
    metadata: &serde_json::Value,
    _trace_id: Option<&String>,
    _trace_header: &str,
) -> Response {
    let body = metadata.to_string();

    // Try a different approach - build response with body first, then add headers
    Response::builder()
        .status(200)
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
        .body(body)
        .build()
}
