//! OAuth 2.0 discovery endpoints implementation

use spin_sdk::http::{Request, Response};
use serde_json::json;

use crate::config::Config;

/// Handle OAuth protected resource metadata endpoint
pub fn oauth_protected_resource(req: &Request, config: &Config, trace_id: &Option<String>) -> Response {
    // Build metadata based on provider type
    let metadata = match &config.provider {
        crate::config::Provider::JWT(jwt_provider) => {
            // For AuthKit domains, return simplified metadata pointing to AuthKit
            let authorization_servers = if !jwt_provider.issuer.is_empty() && 
                (jwt_provider.issuer.contains(".authkit.app") || jwt_provider.issuer.contains(".workos.com")) {
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
                "resource": [
                    "http://localhost:3000/mcp",
                    "http://127.0.0.1:3000/mcp"
                ],
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
        crate::config::Provider::Static(_) => {
            json!({
                "resource": [
                    "http://localhost:3000/mcp",
                    "http://127.0.0.1:3000/mcp"
                ],
                "authorization_servers": [],
                "bearer_methods_supported": ["header"],
                "authentication_methods": {
                    "bearer": {
                        "required": true,
                        "description": "Static token authentication for development",
                    }
                },
            })
        }
    };
    
    build_success_response(metadata, trace_id, &config.trace_header)
}

/// Handle OAuth authorization server metadata endpoint
pub fn oauth_authorization_server(_req: &Request, config: &Config, trace_id: &Option<String>) -> Response {
    // Build metadata based on provider type
    let metadata = match &config.provider {
        crate::config::Provider::JWT(jwt_provider) => {
            // For AuthKit domains, return comprehensive metadata
            if !jwt_provider.issuer.is_empty() && 
                (jwt_provider.issuer.contains(".authkit.app") || jwt_provider.issuer.contains(".workos.com")) {
                // AuthKit metadata with all required endpoints
                json!({
                    "issuer": jwt_provider.issuer,
                    "authorization_endpoint": format!("{}/oauth2/authorize", jwt_provider.issuer),
                    "token_endpoint": format!("{}/oauth2/token", jwt_provider.issuer),
                    "userinfo_endpoint": format!("{}/oauth2/userinfo", jwt_provider.issuer),
                    "jwks_uri": jwt_provider.jwks_uri.as_ref().unwrap_or(&format!("{}/oauth2/jwks", jwt_provider.issuer)),
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
        crate::config::Provider::Static(_) => {
            // Static provider has no authorization server
            json!({
                "error": "not_supported",
                "error_description": "Static token provider does not support OAuth authorization server metadata"
            })
        }
    };
    
    build_success_response(metadata, trace_id, &config.trace_header)
}

/// Handle OpenID configuration endpoint
pub fn openid_configuration(_req: &Request, config: &Config, trace_id: &Option<String>) -> Response {
    // OpenID configuration is similar to OAuth authorization server metadata
    // but with some additional fields
    // Build metadata based on provider type
    let metadata = match &config.provider {
        crate::config::Provider::JWT(jwt_provider) => {
            // For AuthKit, return AuthKit-specific OpenID metadata
            if !jwt_provider.issuer.is_empty() && 
                (jwt_provider.issuer.contains(".authkit.app") || jwt_provider.issuer.contains(".workos.com")) {
                // Match what AuthKit actually returns
                json!({
                    "issuer": jwt_provider.issuer,
                    "authorization_endpoint": format!("{}/oauth2/authorize", jwt_provider.issuer),
                    "token_endpoint": format!("{}/oauth2/token", jwt_provider.issuer),
                    "userinfo_endpoint": format!("{}/oauth2/userinfo", jwt_provider.issuer),
                    "jwks_uri": jwt_provider.jwks_uri.as_ref().unwrap_or(&format!("{}/oauth2/jwks", jwt_provider.issuer)),
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
        crate::config::Provider::Static(_) => {
            // Static provider has no OIDC support
            json!({
                "error": "not_supported",
                "error_description": "Static token provider does not support OpenID Connect"
            })
        }
    };
    
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


