use spin_sdk::http::Response;

use crate::providers::AuthProvider;

/// Handle OAuth discovery requests
pub fn handle_discovery_request(
    path: &str,
    provider: &dyn AuthProvider,
    host: Option<&str>,
) -> Response {
    let resource_url = determine_resource_url(host);

    match path {
        "/.well-known/oauth-protected-resource" => {
            // RFC 9728 Resource Metadata
            let metadata = serde_json::json!({
                "resource": resource_url,
                "authorization_servers": [provider.issuer()],
                "bearer_methods_supported": ["header"],
                "scopes_supported": ["mcp"]
            });

            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(metadata.to_string())
                .build()
        }
        "/.well-known/oauth-authorization-server" => {
            // OAuth 2.0 Authorization Server Metadata
            let discovery = provider.discovery_metadata(&resource_url);
            let metadata = serde_json::json!({
                "issuer": discovery.issuer,
                "authorization_endpoint": discovery.authorization_endpoint,
                "token_endpoint": discovery.token_endpoint,
                "jwks_uri": discovery.jwks_uri,
                "userinfo_endpoint": discovery.userinfo_endpoint,
                "response_types_supported": ["code"],
                "response_modes_supported": ["query"],
                "grant_types_supported": ["authorization_code", "refresh_token"],
                "code_challenge_methods_supported": ["S256"],
                "token_endpoint_auth_methods_supported": ["none"],
                "scopes_supported": ["mcp", "openid", "profile", "email"]
            });

            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(metadata.to_string())
                .build()
        }
        _ => Response::builder()
            .status(404)
            .body("Not found")
            .build(),
    }
}

/// Determine the resource URL based on host
fn determine_resource_url(host: Option<&str>) -> String {
    host.map_or_else(
        || "http://localhost:3000".to_string(),
        |h| {
            // Determine protocol based on host
            let protocol = if h.contains(".fermyon.tech") || 
                             h.contains(".fermyon.cloud") ||
                             h.contains(":443") {
                "https"
            } else {
                "http"
            };
            
            format!("{protocol}://{h}")
        }
    )
}