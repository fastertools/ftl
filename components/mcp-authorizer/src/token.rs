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

    /// Client ID
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,

    /// Additional claims
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

    // Set up validation using configured algorithm (defaults to RS256 like FastMCP)
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

    // Set issuer validation
    if !provider.issuer.is_empty() {
        validation.set_issuer(&[&provider.issuer]);
    }

    // Set audience validation
    if let Some(audiences) = &provider.audience {
        validation.set_audience(audiences);
    } else {
        // Explicitly disable audience validation when no audience is configured
        // This is needed for WorkOS AuthKit compatibility
        validation.validate_aud = false;
    }

    // Decode and validate token
    let token_data = match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(data) => data,
        Err(e) => {
            // Provide more specific error messages for common issues
            return match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    eprintln!("TOKEN_ERROR type=expired");
                    Err(AuthError::ExpiredToken)
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                    eprintln!("TOKEN_ERROR type=invalid_issuer");
                    Err(AuthError::InvalidIssuer)
                }
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    eprintln!("TOKEN_ERROR type=invalid_audience");
                    Err(AuthError::InvalidAudience)
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    eprintln!("TOKEN_ERROR type=invalid_signature");
                    Err(AuthError::InvalidSignature)
                }
                _ => {
                    eprintln!("TOKEN_ERROR type=other detail={e:?}");
                    Err(AuthError::InvalidToken(e.to_string()))
                }
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
    let client_id = claims.client_id.as_ref().unwrap_or(&claims.sub).clone();

    Ok(TokenInfo {
        client_id,
        sub: claims.sub,
        iss: claims.iss,
        scopes,
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
