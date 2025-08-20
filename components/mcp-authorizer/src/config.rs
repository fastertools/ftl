//! Configuration management for the MCP Authorizer

use anyhow::Result;
use serde::{Deserialize, Serialize};
use spin_sdk::variables;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// URL of the MCP gateway to forward requests to
    pub gateway_url: String,

    /// Header name for request tracing
    pub trace_header: String,

    /// JWT provider configuration (optional - if not set, all requests pass through)
    pub provider: Option<Provider>,

    /// Policy-based authorization configuration
    pub authorization: Option<PolicyAuthorization>,
}

/// Provider type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Provider {
    /// JWT provider with JWKS or static key
    #[serde(rename = "jwt")]
    Jwt(JwtProvider),
}

/// JWT provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtProvider {
    /// JWT issuer URL (must be HTTPS)
    pub issuer: String,

    /// JWKS URI for key discovery
    pub jwks_uri: Option<String>,

    /// Static public key (PEM format)
    pub public_key: Option<String>,

    /// Expected audience(s)
    pub audience: Option<Vec<String>>,

    /// JWT signing algorithm (defaults to RS256)
    pub algorithm: Option<String>,

    /// Required scopes for all requests
    pub required_scopes: Option<Vec<String>>,

    /// OAuth 2.0 endpoints (optional)
    pub oauth_endpoints: Option<OAuthEndpoints>,
}

/// Policy-based authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAuthorization {
    /// Rego policy as a string
    pub policy: String,
    
    /// Policy data as JSON string (optional)
    pub data: Option<String>,
}

/// OAuth 2.0 endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthEndpoints {
    pub authorize: Option<String>,
    pub token: Option<String>,
    pub userinfo: Option<String>,
}

impl Config {
    /// Load configuration from Spin variables
    pub fn load() -> Result<Self> {
        let gateway_url = variables::get("mcp_gateway_url")
            .unwrap_or_else(|_| "http://mcp-gateway.spin.internal".to_string());

        let trace_header = variables::get("mcp_trace_header")
            .unwrap_or_else(|_| "x-trace-id".to_string())
            .to_lowercase();

        // Load provider configuration - propagate errors for invalid configs
        // but allow missing provider (returns None)
        let provider = match Provider::load() {
            Ok(p) => Some(p),
            Err(e) => {
                // Check if this is a "no provider configured" situation vs actual error
                // If all provider variables are missing/empty, that's OK (no provider)
                // Otherwise it's a configuration error that should be propagated
                let has_provider_config = variables::get("mcp_jwt_issuer")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .is_some()
                    || variables::get("mcp_jwt_jwks_uri")
                        .ok()
                        .filter(|s| !s.is_empty())
                        .is_some()
                    || variables::get("mcp_jwt_public_key")
                        .ok()
                        .filter(|s| !s.is_empty())
                        .is_some();

                if has_provider_config {
                    // Provider was configured but has errors - propagate the error
                    return Err(e);
                }
                // No provider configured at all - that's OK
                None
            }
        };

        // Load policy authorization if configured
        let authorization = PolicyAuthorization::load().ok();

        Ok(Self {
            gateway_url,
            trace_header,
            provider,
            authorization,
        })
    }
}

impl Provider {
    /// Load provider configuration
    fn load() -> Result<Self> {
        // Check if any provider configuration exists
        let has_jwt_config = variables::get("mcp_jwt_issuer")
            .ok()
            .filter(|s| !s.is_empty())
            .is_some()
            || variables::get("mcp_jwt_jwks_uri")
                .ok()
                .filter(|s| !s.is_empty())
                .is_some()
            || variables::get("mcp_jwt_public_key")
                .ok()
                .filter(|s| !s.is_empty())
                .is_some();

        // If no provider configuration at all, return error (no provider configured)
        if !has_jwt_config {
            return Err(anyhow::anyhow!("No authentication provider configured"));
        }

        Self::load_jwt_provider()
    }

    /// Load JWT provider configuration
    fn load_jwt_provider() -> Result<Self> {
        // Load issuer (optional - empty means no issuer validation)
        let issuer = variables::get("mcp_jwt_issuer")
            .ok()
            .filter(|s| !s.is_empty())
            .map(normalize_issuer)
            .transpose()?
            .unwrap_or_default();

        // Load public key first to check if we should skip JWKS auto-derivation
        let public_key = variables::get("mcp_jwt_public_key")
            .ok()
            .filter(|s| !s.is_empty());

        // Load JWKS URI or auto-derive it (but only if no public key is set)
        let jwks_uri = variables::get("mcp_jwt_jwks_uri")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                // Auto-derive JWKS URI for known providers only if:
                // 1. Issuer is set
                // 2. No public key is configured
                if !issuer.is_empty() && public_key.is_none() {
                    if issuer.contains(".authkit.app") || issuer.contains(".workos.com") {
                        // WorkOS AuthKit uses /oauth2/jwks endpoint
                        Some(format!("{issuer}/oauth2/jwks"))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .map(|uri| normalize_url(&uri))
            .transpose()?;

        // Validate we have at least one key source
        if jwks_uri.is_none() && public_key.is_none() {
            return Err(anyhow::anyhow!(
                "Either mcp_jwt_jwks_uri or mcp_jwt_public_key must be provided"
            ));
        }

        // Validate we don't have both
        if jwks_uri.is_some() && public_key.is_some() {
            return Err(anyhow::anyhow!(
                "Cannot specify both mcp_jwt_jwks_uri and mcp_jwt_public_key"
            ));
        }

        // Load audience (required for security)
        let audience_str = variables::get("mcp_jwt_audience")
            .ok()
            .filter(|s| !s.is_empty());

        // Audience is required when using JWT provider
        if audience_str.is_none() {
            return Err(anyhow::anyhow!(
                "mcp_jwt_audience is required for JWT authentication (security best practice)"
            ));
        }

        // Parse audience - can be comma-separated for multiple audiences
        let audience = audience_str.map(|s| {
            s.split(',')
                .map(|aud| aud.trim().to_string())
                .filter(|aud| !aud.is_empty())
                .collect()
        });

        // Load algorithm (optional, defaults to RS256)
        let algorithm = variables::get("mcp_jwt_algorithm")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|alg| {
                // Validate algorithm
                let valid_algorithms = [
                    "HS256", "HS384", "HS512", "RS256", "RS384", "RS512", "ES256", "ES384",
                    "PS256", "PS384", "PS512",
                ];

                if !valid_algorithms.contains(&alg.as_str()) {
                    return Err(anyhow::anyhow!("Unsupported algorithm: {}", alg));
                }
                Ok(alg)
            })
            .transpose()?;

        // Load required scopes (optional)
        let required_scopes = variables::get("mcp_jwt_required_scopes")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.split(',').map(|scope| scope.trim().to_string()).collect());

        // Load OAuth endpoints (all optional)
        let oauth_endpoints = load_oauth_endpoints()?;

        Ok(Self::Jwt(JwtProvider {
            issuer,
            jwks_uri,
            public_key,
            audience,
            algorithm,
            required_scopes,
            oauth_endpoints,
        }))
    }
}

/// Load OAuth endpoints if any are configured
fn load_oauth_endpoints() -> Result<Option<OAuthEndpoints>> {
    let authorize = variables::get("mcp_oauth_authorize_endpoint")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|url| normalize_url(&url))
        .transpose()?;

    let token = variables::get("mcp_oauth_token_endpoint")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|url| normalize_url(&url))
        .transpose()?;

    let userinfo = variables::get("mcp_oauth_userinfo_endpoint")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|url| normalize_url(&url))
        .transpose()?;

    if authorize.is_some() || token.is_some() || userinfo.is_some() {
        Ok(Some(OAuthEndpoints {
            authorize,
            token,
            userinfo,
        }))
    } else {
        Ok(None)
    }
}

/// Normalize issuer (handle both URLs and plain strings)
fn normalize_issuer(mut issuer: String) -> Result<String> {
    // Check if it looks like a URL
    if issuer.starts_with("http://") || issuer.starts_with("https://") {
        // For URLs, validate HTTPS
        if !issuer.starts_with("https://") {
            return Err(anyhow::anyhow!("URL issuers must use HTTPS"));
        }

        // Remove trailing slash from URLs
        if issuer.ends_with('/') {
            issuer.pop();
        }
    }
    // Otherwise, keep as-is (string issuer per RFC 7519)

    Ok(issuer)
}

/// Normalize URL (ensure HTTPS, validate format)
fn normalize_url(url: &str) -> Result<String> {
    // Add https:// if no protocol
    let normalized = if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{url}")
    } else {
        url.to_string()
    };

    // Validate HTTPS for security
    if !normalized.starts_with("https://") {
        return Err(anyhow::anyhow!("URL must use HTTPS: {url}"));
    }

    Ok(normalized)
}

impl PolicyAuthorization {
    /// Load policy authorization from Spin variables
    pub fn load() -> Result<Self> {
        // Load policy (required)
        let policy = variables::get("mcp_policy")
            .map_err(|_| anyhow::anyhow!("mcp_policy variable is required for authorization"))?;
        
        if policy.is_empty() {
            return Err(anyhow::anyhow!("mcp_policy cannot be empty"));
        }
        
        // Load policy data (optional)
        let data = variables::get("mcp_policy_data")
            .ok()
            .filter(|s| !s.is_empty());
        
        Ok(Self {
            policy,
            data,
        })
    }
}
