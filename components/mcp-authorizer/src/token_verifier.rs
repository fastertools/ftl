use anyhow::{Context, Result};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use spin_sdk::http::Request;

use crate::kv::{self, KvStore};
use crate::providers::AuthProvider;

/// Token information extracted from JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub client_id: String,
    pub email: Option<String>,
    pub provider: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<u64>,
}

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingToken,
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Token has expired")]
    ExpiredToken,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// JWT Claims structure (OIDC compliant)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // Standard OIDC claims
    pub sub: String,                          // Subject (user ID)
    pub iss: Option<String>,                  // Issuer
    pub aud: Option<serde_json::Value>,       // Audience (string or array)
    pub exp: Option<u64>,                     // Expiration time
    pub iat: Option<u64>,                     // Issued at
    pub jti: Option<String>,                  // JWT ID
    
    // Profile claims
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    
    // OAuth 2.0 scopes
    pub scope: Option<String>,                // Space-delimited scopes
    pub scp: Option<Vec<String>>,             // Array of scopes (some providers)
    
    // Additional claims
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Extract bearer token from request
pub fn extract_bearer_token(req: &Request) -> Result<&str, AuthError> {
    req.headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        .and_then(|(_, value)| value.as_str())
        .and_then(|auth| auth.strip_prefix("Bearer ").or(auth.strip_prefix("bearer ")))
        .ok_or(AuthError::MissingToken)
}

/// Verify a JWT token using the specified provider
pub async fn verify_token(
    token: &str,
    provider: &dyn AuthProvider,
    kv: &KvStore,
) -> Result<TokenInfo, AuthError> {
    // Get JWKS URI from provider
    let jwks_uri = provider.jwks_uri();
    
    // Extract key ID from token header
    let header = decode_header(token)
        .map_err(|e| AuthError::InvalidToken(format!("Invalid JWT header: {}", e)))?;
    
    // Get decoding key (with caching)
    let key = get_decoding_key(jwks_uri, header.kid.as_deref(), kv).await
        .map_err(|e| AuthError::Internal(format!("Failed to get decoding key: {}", e)))?;
    
    // Set up validation
    let mut validation = Validation::new(header.alg);
    
    // Always validate issuer
    validation.set_issuer(&[provider.issuer()]);
    
    // Validate audience if provided
    if let Some(audience) = provider.audience() {
        validation.set_audience(&[audience]);
    }
    
    // Decode and validate token
    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
            _ => AuthError::InvalidToken(format!("JWT validation failed: {}", e)),
        })?;
    
    let claims = token_data.claims;
    
    // Extract scopes
    let scopes = extract_scopes(&claims);
    
    Ok(TokenInfo {
        client_id: claims.sub.clone(),
        email: claims.email.clone(),
        provider: provider.name().to_string(),
        scopes,
        expires_at: claims.exp,
    })
}

/// Get decoding key with caching
async fn get_decoding_key(
    jwks_uri: &str,
    kid: Option<&str>,
    kv: &KvStore,
) -> Result<DecodingKey> {
    // Try cache first
    let cache_key = kv::cache_key(kv::keys::JWKS_CACHE, jwks_uri);
    
    // Check if we have cached JWKS
    if let Ok(Some(jwks)) = kv.get::<crate::jwks::JwksResponse>(&cache_key) {
        return find_key_in_jwks(&jwks, kid);
    }
    
    // Fetch JWKS
    let jwks = crate::jwks::fetch_jwks(jwks_uri).await?;
    
    // Cache for 5 minutes
    let _ = kv.set(&cache_key, &jwks, kv::ttl::JWKS_CACHE);
    
    find_key_in_jwks(&jwks, kid)
}

/// Find a specific key in JWKS
fn find_key_in_jwks(
    jwks: &crate::jwks::JwksResponse,
    kid: Option<&str>,
) -> Result<DecodingKey> {
    let jwk = if let Some(kid) = kid {
        // Find specific key by ID
        jwks.keys
            .iter()
            .find(|k| k.kid.as_deref() == Some(kid))
            .ok_or_else(|| anyhow::anyhow!("Key with id '{}' not found in JWKS", kid))?
    } else if jwks.keys.len() == 1 {
        // Only one key, use it
        &jwks.keys[0]
    } else {
        return Err(anyhow::anyhow!("Multiple keys in JWKS but no key ID in token"));
    };
    
    // Convert JWK to decoding key
    match jwk.kty.as_str() {
        "RSA" => {
            let n = jwk.n.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing 'n' in RSA key"))?;
            let e = jwk.e.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing 'e' in RSA key"))?;
            
            DecodingKey::from_rsa_components(n, e)
                .context("Failed to create RSA key")
        }
        _ => Err(anyhow::anyhow!("Unsupported key type: {}", jwk.kty)),
    }
}

/// Extract scopes from JWT claims
fn extract_scopes(claims: &Claims) -> Vec<String> {
    // Try 'scope' claim first (standard OAuth2)
    if let Some(scope) = &claims.scope {
        return scope.split_whitespace().map(String::from).collect();
    }
    
    // Try 'scp' claim (some providers use this)
    if let Some(scp) = &claims.scp {
        return scp.clone();
    }
    
    Vec::new()
}