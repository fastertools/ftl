use std::time::Duration;

use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

// Include the generated client code
include!(concat!(env!("OUT_DIR"), "/ftl_backend_client.rs"));

// Re-export the generated types module for easier access
pub use types::*;

/// Configuration for the FTL API client
pub struct ApiConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
    pub timeout: Duration,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://fqwe5s59ob.execute-api.us-east-1.amazonaws.com".to_string(),
            auth_token: None,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Create a configured FTL API client
pub fn create_client(config: ApiConfig) -> Result<Client> {
    let mut headers = HeaderMap::new();
    
    // Add authorization header if token is provided
    if let Some(token) = config.auth_token {
        let auth_value = HeaderValue::from_str(&format!("Bearer {}", token))?;
        headers.insert(AUTHORIZATION, auth_value);
    }
    
    let http_client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .timeout(config.timeout)
        .build()?;
    
    Ok(Client::new_with_client(&config.base_url, http_client))
}

/// Helper function to create client from environment variables
pub fn create_client_from_env() -> Result<Client> {
    let base_url = std::env::var("FTL_API_URL")
        .unwrap_or_else(|_| "https://fqwe5s59ob.execute-api.us-east-1.amazonaws.com".to_string());
    
    let auth_token = std::env::var("FTL_AUTH_TOKEN").ok();
    
    create_client(ApiConfig {
        base_url,
        auth_token,
        ..Default::default()
    })
}