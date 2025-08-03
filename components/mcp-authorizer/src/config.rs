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
    
    /// JWT provider configuration (always required)
    pub provider: Provider,
}

/// JWT provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
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
    
    /// OAuth 2.0 endpoints (optional)
    pub oauth_endpoints: Option<OAuthEndpoints>,
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
            .unwrap_or_else(|_| "https://mcp-gateway.spin.internal".to_string());
        
        let trace_header = variables::get("mcp_trace_header")
            .unwrap_or_else(|_| "x-trace-id".to_string())
            .to_lowercase();
        
        // Provider configuration is always required
        let provider = Provider::load()?;
        
        Ok(Config {
            gateway_url,
            trace_header,
            provider,
        })
    }
}

impl Provider {
    /// Load provider configuration
    fn load() -> Result<Self> {
        // Load issuer (required)
        let issuer = normalize_issuer(
            variables::get("mcp_jwt_issuer")
                .map_err(|_| anyhow::anyhow!("Missing mcp_jwt_issuer"))?
        )?;
        
        // Load JWKS URI or public key (one required)
        let jwks_uri = variables::get("mcp_jwt_jwks_uri").ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                // Auto-derive JWKS URI for known providers
                if issuer.contains(".authkit.app") || issuer.contains(".workos.com") {
                    Some(format!("{}/.well-known/jwks.json", issuer))
                } else {
                    None
                }
            })
            .map(|uri| normalize_url(&uri))
            .transpose()?;
        
        let public_key = variables::get("mcp_jwt_public_key").ok()
            .filter(|s| !s.is_empty());
        
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
        
        // Load audience (optional)
        let audience = variables::get("mcp_jwt_audience").ok()
            .filter(|s| !s.is_empty())
            .map(|s| vec![s]);
        
        // Load algorithm (optional, defaults to RS256 like FastMCP)
        let algorithm = variables::get("mcp_jwt_algorithm").ok()
            .filter(|s| !s.is_empty())
            .map(|alg| {
                // Validate algorithm
                let valid_algorithms = [
                    "HS256", "HS384", "HS512",
                    "RS256", "RS384", "RS512", 
                    "ES256", "ES384",
                    "PS256", "PS384", "PS512",
                ];
                
                if !valid_algorithms.contains(&alg.as_str()) {
                    return Err(anyhow::anyhow!("Unsupported algorithm: {}", alg));
                }
                Ok(alg)
            })
            .transpose()?;
        
        // Load OAuth endpoints (all optional)
        let oauth_endpoints = load_oauth_endpoints()?;
        
        Ok(Provider {
            issuer,
            jwks_uri,
            public_key,
            audience,
            algorithm,
            oauth_endpoints,
        })
    }
}

/// Load OAuth endpoints if any are configured
fn load_oauth_endpoints() -> Result<Option<OAuthEndpoints>> {
    let authorize = variables::get("mcp_oauth_authorize_endpoint").ok()
        .filter(|s| !s.is_empty())
        .map(|url| normalize_url(&url))
        .transpose()?;
    
    let token = variables::get("mcp_oauth_token_endpoint").ok()
        .filter(|s| !s.is_empty())
        .map(|url| normalize_url(&url))
        .transpose()?;
    
    let userinfo = variables::get("mcp_oauth_userinfo_endpoint").ok()
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
        format!("https://{}", url)
    } else {
        url.to_string()
    };
    
    // Validate HTTPS for security
    if !normalized.starts_with("https://") {
        return Err(anyhow::anyhow!("URL must use HTTPS: {}", url));
    }
    
    Ok(normalized)
}