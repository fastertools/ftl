use std::time::Duration;

use anyhow::Result;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use crate::config::{DEFAULT_API_BASE_URL, API_URL_ENV_VAR, AUTH_TOKEN_ENV_VAR, DEFAULT_API_TIMEOUT_SECS};

// Include the generated client code
#[allow(clippy::use_self)]
#[allow(clippy::pedantic)]
#[allow(clippy::nursery)]
#[allow(unused_imports)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/ftl_backend_client.rs"));
}

// Re-export from the generated module
pub use generated::*;

// Re-export the generated types module and its submodules for easier access
pub use generated::types;
pub use generated::types::error;

/// Configuration for the FTL API client
pub struct ApiConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
    pub timeout: Duration,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_API_BASE_URL.to_string(),
            auth_token: None,
            timeout: Duration::from_secs(DEFAULT_API_TIMEOUT_SECS),
        }
    }
}

/// Create a configured FTL API client
#[allow(dead_code)]
pub fn create_client(config: ApiConfig) -> Result<Client> {
    let mut headers = HeaderMap::new();

    // Add authorization header if token is provided
    if let Some(token) = config.auth_token {
        let auth_value = HeaderValue::from_str(&format!("Bearer {token}"))?;
        headers.insert(AUTHORIZATION, auth_value);
    }

    let http_client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .timeout(config.timeout)
        .build()?;

    Ok(Client::new_with_client(&config.base_url, http_client))
}

/// Helper function to create client from environment variables
#[allow(dead_code)]
pub fn create_client_from_env() -> Result<Client> {
    let base_url = std::env::var(API_URL_ENV_VAR)
        .unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_string());

    let auth_token = std::env::var(AUTH_TOKEN_ENV_VAR).ok();

    create_client(ApiConfig {
        base_url,
        auth_token,
        ..Default::default()
    })
}

/// Get the API base URL from environment or default
pub fn get_api_base_url() -> String {
    std::env::var(API_URL_ENV_VAR)
        .unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_string())
}
