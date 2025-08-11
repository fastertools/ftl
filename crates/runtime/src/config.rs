//! Centralized configuration for the FTL CLI
//!
//! This module provides a single source of truth for all configuration values
//! used throughout the application.
//!
//! # Environment Variables
//!
//! The following environment variables can be used to override defaults:
//! - `FTL_API_URL`: Override the default backend API URL
//! - `FTL_AUTH_TOKEN`: Provide an authentication token

/// Default backend API base URL
pub const DEFAULT_API_BASE_URL: &str = "https://vnwyancgjj.execute-api.us-west-2.amazonaws.com";

/// Environment variable name for overriding the API URL
pub const API_URL_ENV_VAR: &str = "FTL_API_URL";

/// Environment variable name for the auth token
pub const AUTH_TOKEN_ENV_VAR: &str = "FTL_AUTH_TOKEN";

/// Default API timeout in seconds
pub const DEFAULT_API_TIMEOUT_SECS: u64 = 30;
