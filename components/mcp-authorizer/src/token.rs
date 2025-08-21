//! JWT token verification with JWKS support

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use spin_sdk::key_value::Store;

use crate::config::JwtProvider;
use crate::error::{AuthError, Result};
use crate::jwks;

/// Token information extracted from a verified JWT
#[derive(Debug, Clone)]
pub struct TokenInfo {
    /// Client ID (from `client_id` claim or sub)
    pub client_id: String,

    /// Subject (user ID)
    pub sub: String,

    /// Issuer
    pub iss: String,

    /// Scopes
    pub scopes: Vec<String>,

    /// All claims from the token (for authorization and forwarding)
    pub claims: std::collections::HashMap<String, serde_json::Value>,
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Subject
    sub: String,

    /// Issuer
    iss: String,

    /// Audience (can be string or array)
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<AudienceValue>,

    /// Expiration time
    exp: i64,

    /// Issued at
    iat: i64,

    /// `OAuth2` scope claim
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,

    /// Microsoft-style scope claim (can be string or array)
    #[serde(skip_serializing_if = "Option::is_none")]
    scp: Option<ScopeValue>,

    /// Additional claims (captures all other claims)
    #[serde(flatten)]
    additional: serde_json::Map<String, serde_json::Value>,
}

/// Audience value (can be string or array)
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum AudienceValue {
    Single(String),
    Multiple(Vec<String>),
}

/// Scope value (can be string or array)
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ScopeValue {
    String(String),
    List(Vec<String>),
}

/// Verify a JWT token using the provided configuration
#[allow(clippy::too_many_lines)]
pub async fn verify(token: &str, provider: &JwtProvider, store: &Store) -> Result<TokenInfo> {
    // Decode header to get KID if present
    let header = decode_header(token)?;
    let kid = header.kid.as_deref();

    // Get decoding key
    let decoding_key = if let Some(public_key) = &provider.public_key {
        // Use static public key
        DecodingKey::from_rsa_pem(public_key.as_bytes())
            .map_err(|e| AuthError::Configuration(format!("Invalid public key: {e}")))?
    } else if let Some(jwks_uri) = &provider.jwks_uri {
        // Fetch JWKS and find matching key
        let jwks = jwks::fetch_jwks(jwks_uri, store).await?;
        jwks::find_key(&jwks, kid)?
    } else {
        return Err(AuthError::Configuration(
            "No key source configured".to_string(),
        ));
    };

    // Set up validation using configured algorithm (defaults to RS256)
    let algorithm = match provider.algorithm.as_deref().unwrap_or("RS256") {
        "HS256" => Algorithm::HS256,
        "HS384" => Algorithm::HS384,
        "HS512" => Algorithm::HS512,
        "RS256" => Algorithm::RS256,
        "RS384" => Algorithm::RS384,
        "RS512" => Algorithm::RS512,
        "ES256" => Algorithm::ES256,
        "ES384" => Algorithm::ES384,
        "PS256" => Algorithm::PS256,
        "PS384" => Algorithm::PS384,
        "PS512" => Algorithm::PS512,
        alg => {
            return Err(AuthError::Configuration(format!(
                "Unsupported algorithm: {alg}"
            )));
        }
    };
    let mut validation = Validation::new(algorithm);

    // Set issuer validation (only if configured)
    if !provider.issuer.is_empty() {
        validation.set_issuer(&[&provider.issuer]);
    }

    // Set audience validation (always required for security)
    if let Some(audiences) = &provider.audience {
        validation.set_audience(audiences);
    } else {
        // This should never happen as audience is required in config
        return Err(AuthError::Configuration(
            "Audience validation is required but no audience configured".to_string(),
        ));
    }

    // Enable nbf (not before) validation if present in token
    validation.validate_nbf = true;

    // Add leeway for clock skew tolerance (60 seconds is reasonable for distributed systems)
    // This helps with slight time differences between the token issuer and validator
    validation.leeway = 60;

    // Set required claims - we always require exp (default), sub, and iss
    // The jsonwebtoken library will ensure these claims are present before validation
    validation.set_required_spec_claims(&["exp", "sub", "iss"]);

    // Decode and validate token
    let token_data = match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(data) => data,
        Err(e) => {
            // Provide more specific error messages for common issues
            return match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => Err(AuthError::ExpiredToken),
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => Err(AuthError::InvalidIssuer),
                jsonwebtoken::errors::ErrorKind::InvalidAudience => Err(AuthError::InvalidAudience),
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    Err(AuthError::InvalidSignature)
                }
                _ => Err(AuthError::InvalidToken(e.to_string())),
            };
        }
    };
    let claims = token_data.claims;

    // Extract scopes
    let scopes = extract_scopes(&claims);

    // Check required scopes
    if let Some(required_scopes) = &provider.required_scopes {
        use std::collections::HashSet;

        let token_scopes: HashSet<String> = scopes.iter().cloned().collect();
        let required_set: HashSet<String> = required_scopes.iter().cloned().collect();

        if !required_set.is_subset(&token_scopes) {
            let missing_scopes: Vec<String> =
                required_set.difference(&token_scopes).cloned().collect();
            return Err(AuthError::Unauthorized(format!(
                "Token missing required scopes: {missing_scopes:?}"
            )));
        }
    }

    // Extract client ID (prefer explicit claim over sub)
    let client_id = claims
        .additional
        .get("client_id")
        .and_then(|v| v.as_str())
        .unwrap_or(&claims.sub)
        .to_string();

    // Build complete claims map
    let mut all_claims = std::collections::HashMap::new();
    all_claims.insert(
        "sub".to_string(),
        serde_json::Value::String(claims.sub.clone()),
    );
    all_claims.insert(
        "iss".to_string(),
        serde_json::Value::String(claims.iss.clone()),
    );
    if let Some(aud) = claims.aud {
        all_claims.insert(
            "aud".to_string(),
            match aud {
                AudienceValue::Single(s) => serde_json::Value::String(s),
                AudienceValue::Multiple(v) => serde_json::json!(v),
            },
        );
    }
    all_claims.insert("exp".to_string(), serde_json::json!(claims.exp));
    all_claims.insert("iat".to_string(), serde_json::json!(claims.iat));
    if let Some(scope) = claims.scope {
        all_claims.insert("scope".to_string(), serde_json::Value::String(scope));
    }
    if let Some(scp) = claims.scp {
        all_claims.insert(
            "scp".to_string(),
            match scp {
                ScopeValue::String(s) => serde_json::Value::String(s),
                ScopeValue::List(v) => serde_json::json!(v),
            },
        );
    }
    // Add all additional claims
    for (key, value) in claims.additional {
        all_claims.insert(key, value);
    }

    Ok(TokenInfo {
        client_id,
        sub: claims.sub,
        iss: claims.iss,
        scopes,
        claims: all_claims,
    })
}

/// Extract scopes from claims
fn extract_scopes(claims: &Claims) -> Vec<String> {
    // OAuth2 'scope' claim takes precedence
    if let Some(scope) = &claims.scope {
        return scope.split_whitespace().map(String::from).collect();
    }

    // Fall back to Microsoft 'scp' claim
    if let Some(scp) = &claims.scp {
        return match scp {
            ScopeValue::String(s) => s.split_whitespace().map(String::from).collect(),
            ScopeValue::List(list) => list.clone(),
        };
    }

    // No scopes
    Vec::new()
}
