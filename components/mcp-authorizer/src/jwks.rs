//! JWKS (JSON Web Key Set) fetching and caching

use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use spin_sdk::http::Response;
use spin_sdk::key_value::Store;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AuthError, Result};

/// JWKS cache TTL in seconds (1 hour)
const JWKS_CACHE_TTL: u64 = 3600;

/// JWKS response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    /// Key type (RSA, EC, etc.)
    pub kty: String,

    /// Key use (sig, enc)
    #[serde(rename = "use")]
    pub use_: Option<String>,

    /// Algorithm
    pub alg: Option<String>,

    /// Key ID
    pub kid: Option<String>,

    /// RSA modulus (base64url)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,

    /// RSA exponent (base64url)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
}

/// Cached JWKS with expiration
#[derive(Debug, Serialize, Deserialize)]
struct CachedJwks {
    jwks: Jwks,
    expires_at: u64,
}

/// Fetch JWKS from URI with caching
pub async fn fetch_jwks(jwks_uri: &str, store: &Store) -> Result<Jwks> {
    let cache_key = format!("jwks:{jwks_uri}");

    // Check cache first
    if let Ok(Some(cached_data)) = store.get(&cache_key) {
        if let Ok(cached_str) = String::from_utf8(cached_data) {
            if let Ok(cached) = serde_json::from_str::<CachedJwks>(&cached_str) {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if now < cached.expires_at {
                    return Ok(cached.jwks);
                }
            }
        }
    }

    // Fetch JWKS from URI
    eprintln!("Fetching JWKS from: {jwks_uri}");
    let request = spin_sdk::http::Request::builder()
        .method(spin_sdk::http::Method::Get)
        .uri(jwks_uri)
        .header("Accept", "application/json")
        .build();

    let response: Response = spin_sdk::http::send(request).await.map_err(|e| {
        eprintln!("Failed to fetch JWKS from {jwks_uri}: {e}");
        AuthError::Internal(format!("Failed to fetch JWKS: {e}"))
    })?;

    if *response.status() != 200 {
        return Err(AuthError::Internal(format!(
            "JWKS fetch failed with status: {}",
            response.status()
        )));
    }

    let body = response.body();
    let jwks: Jwks = serde_json::from_slice(body)?;

    // Cache the JWKS
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + JWKS_CACHE_TTL;

    let cached = CachedJwks {
        jwks: jwks.clone(),
        expires_at,
    };

    let _ = store.set(&cache_key, serde_json::to_string(&cached)?.as_bytes());

    Ok(jwks)
}

/// Find a key in JWKS that matches the given KID
pub fn find_key(jwks: &Jwks, kid: Option<&str>) -> Result<DecodingKey> {
    // Filter keys by type and use
    let matching_keys: Vec<&Jwk> = jwks
        .keys
        .iter()
        .filter(|key| {
            // Check key type
            if key.kty != "RSA" {
                return false;
            }

            // Check use if specified
            if let Some(use_) = &key.use_ {
                if use_ != "sig" {
                    return false;
                }
            }

            true
        })
        .collect();

    if matching_keys.is_empty() {
        return Err(AuthError::InvalidToken(
            "No matching keys found in JWKS".to_string(),
        ));
    }

    // Find key by KID if specified
    let key = if let Some(kid) = kid {
        // Token has KID - find exact match
        matching_keys
            .iter()
            .find(|k| k.kid.as_deref() == Some(kid))
            .ok_or_else(|| AuthError::InvalidToken(format!("Key with kid '{kid}' not found")))?
    } else {
        // No KID in token - only allow if there's exactly one key
        if matching_keys.len() == 1 {
            matching_keys
                .first()
                .copied()
                .ok_or_else(|| AuthError::InvalidToken("No keys found".to_string()))?
        } else if matching_keys.is_empty() {
            return Err(AuthError::InvalidToken("No keys found in JWKS".to_string()));
        } else {
            return Err(AuthError::InvalidToken(
                "Multiple keys in JWKS but no key ID (kid) in token".to_string(),
            ));
        }
    };

    // Extract RSA components
    let n = key
        .n
        .as_ref()
        .ok_or_else(|| AuthError::InvalidToken("Missing RSA modulus".to_string()))?;
    let e = key
        .e
        .as_ref()
        .ok_or_else(|| AuthError::InvalidToken("Missing RSA exponent".to_string()))?;

    // Build RSA public key
    build_rsa_key(n, e)
}

/// Build RSA decoding key from modulus and exponent
fn build_rsa_key(n: &str, e: &str) -> Result<DecodingKey> {
    // jsonwebtoken provides a convenient method for this
    DecodingKey::from_rsa_components(n, e)
        .map_err(|e| AuthError::InvalidToken(format!("Invalid RSA key components: {e}")))
}
