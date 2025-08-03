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

/// Provider type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Provider {
    /// JWT provider with JWKS or static key
    #[serde(rename = "jwt")]
    JWT(JWTProvider),
    
    /// Static token provider for development
    #[serde(rename = "static")]
    Static(StaticProvider),
}

/// JWT provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTProvider {
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

/// Static token provider configuration for development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticProvider {
    /// Map of token strings to their metadata
    pub tokens: std::collections::HashMap<String, StaticTokenInfo>,
    
    /// Required scopes for all requests
    pub required_scopes: Option<Vec<String>>,
}

/// Static token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticTokenInfo {
    /// Client ID for this token
    pub client_id: String,
    
    /// User ID (subject)
    pub sub: String,
    
    /// Scopes granted to this token
    pub scopes: Vec<String>,
    
    /// Optional expiration timestamp
    pub expires_at: Option<i64>,
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
        // Check provider type first
        let provider_type = variables::get("mcp_provider_type")
            .unwrap_or_else(|_| "jwt".to_string());
        
        match provider_type.as_str() {
            "static" => Self::load_static_provider(),
            "jwt" | _ => Self::load_jwt_provider(),
        }
    }
    
    /// Load JWT provider configuration
    fn load_jwt_provider() -> Result<Self> {
        // Load issuer (optional - empty means no issuer validation)
        let issuer = variables::get("mcp_jwt_issuer").ok()
            .filter(|s| !s.is_empty())
            .map(normalize_issuer)
            .transpose()?
            .unwrap_or_default();
        
        // Load public key first to check if we should skip JWKS auto-derivation
        let public_key = variables::get("mcp_jwt_public_key").ok()
            .filter(|s| !s.is_empty());
        
        // Load JWKS URI or auto-derive it (but only if no public key is set)
        let jwks_uri = variables::get("mcp_jwt_jwks_uri").ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                // Auto-derive JWKS URI for known providers only if:
                // 1. Issuer is set
                // 2. No public key is configured
                if !issuer.is_empty() && public_key.is_none() {
                    if issuer.contains(".authkit.app") || issuer.contains(".workos.com") {
                        // WorkOS AuthKit uses /oauth2/jwks endpoint
                        Some(format!("{}/oauth2/jwks", issuer))
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
        
        // Load required scopes (optional)
        let required_scopes = variables::get("mcp_jwt_required_scopes").ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.split(',').map(|scope| scope.trim().to_string()).collect());
        
        // Load OAuth endpoints (all optional)
        let oauth_endpoints = load_oauth_endpoints()?;
        
        Ok(Provider::JWT(JWTProvider {
            issuer,
            jwks_uri,
            public_key,
            audience,
            algorithm,
            required_scopes,
            oauth_endpoints,
        }))
    }
    
    /// Load static provider configuration
    fn load_static_provider() -> Result<Self> {
        use std::collections::HashMap;
        
        // Load static tokens from configuration
        // Format: mcp_static_tokens = "token1:client1:user1:read,write;token2:client2:user2:admin"
        let tokens_config = variables::get("mcp_static_tokens")
            .map_err(|_| anyhow::anyhow!("Missing mcp_static_tokens for static provider"))?;
        
        let mut tokens = HashMap::new();
        
        for token_def in tokens_config.split(';') {
            let token_def = token_def.trim();
            if token_def.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = token_def.split(':').collect();
            if parts.len() < 4 {
                return Err(anyhow::anyhow!(
                    "Invalid static token format. Expected: token:client_id:sub:scope1,scope2"
                ));
            }
            
            let token = parts[0].to_string();
            let client_id = parts[1].to_string();
            let sub = parts[2].to_string();
            let scopes = parts[3].split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            // Optional expiration timestamp as 5th part
            let expires_at = parts.get(4)
                .and_then(|s| s.parse::<i64>().ok());
            
            tokens.insert(token, StaticTokenInfo {
                client_id,
                sub,
                scopes,
                expires_at,
            });
        }
        
        if tokens.is_empty() {
            return Err(anyhow::anyhow!("No static tokens configured"));
        }
        
        // Load required scopes (optional)
        let required_scopes = variables::get("mcp_jwt_required_scopes").ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.split(',').map(|scope| scope.trim().to_string()).collect());
        
        Ok(Provider::Static(StaticProvider {
            tokens,
            required_scopes,
        }))
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