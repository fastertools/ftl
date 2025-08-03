//! Static token verification for development and testing

use chrono::Utc;

use crate::config::StaticProvider;
use crate::error::{AuthError, Result};
use crate::token::TokenInfo;

/// Verify a static token using the provided configuration
pub async fn verify(token: &str, provider: &StaticProvider) -> Result<TokenInfo> {
    // Look up token in static token map
    let token_info = provider.tokens.get(token)
        .ok_or_else(|| AuthError::InvalidToken("Token not found".to_string()))?;
    
    // Check expiration if present
    if let Some(expires_at) = token_info.expires_at {
        let now = Utc::now().timestamp();
        if expires_at < now {
            return Err(AuthError::ExpiredToken);
        }
    }
    
    // Check required scopes
    if let Some(required_scopes) = &provider.required_scopes {
        use std::collections::HashSet;
        
        let token_scopes: HashSet<_> = token_info.scopes.iter().collect();
        let required_set: HashSet<_> = required_scopes.iter().collect();
        
        if !required_set.is_subset(&token_scopes) {
            let missing_scopes: Vec<_> = required_set.difference(&token_scopes)
                .map(|s| (*s).clone())
                .collect();
            return Err(AuthError::Unauthorized(
                format!("Token missing required scopes: {:?}", missing_scopes)
            ));
        }
    }
    
    Ok(TokenInfo {
        client_id: token_info.client_id.clone(),
        sub: token_info.sub.clone(),
        iss: "static".to_string(), // Static provider has no issuer
        scopes: token_info.scopes.clone(),
    })
}