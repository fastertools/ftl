//! Authentication context and token extraction

use crate::error::{AuthError, Result};
use spin_sdk::http::Request;

/// Authentication context for an authenticated request
#[derive(Debug, Clone)]
pub struct Context {
    /// Client ID from the token (from `client_id` claim or sub)
    pub client_id: String,

    /// User ID (subject) from the token
    pub user_id: String,

    /// Scopes granted to the token
    pub scopes: Vec<String>,

    /// Token issuer
    pub issuer: String,

    /// Raw bearer token (for forwarding if needed)
    pub raw_token: String,

    /// Additional claims from the token (for generic authorization and forwarding)
    #[allow(dead_code)] // Will be used for claim forwarding in future
    pub additional_claims: std::collections::HashMap<String, serde_json::Value>,
}

/// Extract bearer token from request
pub fn extract_bearer_token(req: &Request) -> Result<&str> {
    // Get authorization header
    let auth_header = req
        .headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        .ok_or_else(|| AuthError::Unauthorized("Missing authorization header".to_string()))?
        .1;

    // Convert to string
    let auth_str = auth_header.as_str().ok_or_else(|| {
        AuthError::InvalidToken("Invalid authorization header encoding".to_string())
    })?;

    // Extract bearer token
    auth_str.strip_prefix("Bearer ").ok_or_else(|| {
        AuthError::InvalidToken("Authorization header must use Bearer scheme".to_string())
    })
}
