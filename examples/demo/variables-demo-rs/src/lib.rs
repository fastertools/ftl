use ftl_sdk::{text, tools, ToolResponse};
use schemars::JsonSchema;
use serde::Deserialize;
use spin_sdk::variables;

#[derive(Deserialize, JsonSchema)]
struct ConfigInfoInput {}

#[derive(Deserialize, JsonSchema)]
struct MakeApiCallInput {}

#[derive(Deserialize, JsonSchema)]
struct EnvironmentCheckInput {}

tools! {
    /// Shows configuration information using Spin variables
    fn config_info(_input: ConfigInfoInput) -> ToolResponse {
        // Get variables from Spin - these are configured in ftl.toml
        let api_url = variables::get("api_url").unwrap_or_else(|_| "not set".to_string());
        let api_version = variables::get("api_version").unwrap_or_else(|_| "not set".to_string());
        let environment = variables::get("environment").unwrap_or_else(|_| "not set".to_string());
        
        // Try to get a secret token - this should be a required variable
        let token_status = match variables::get("api_token") {
            Ok(_) => "Token is configured (hidden for security)",
            Err(_) => "Token is not configured",
        };
        
        text!(
            "Configuration Info:\n\
            - API URL: {}\n\
            - API Version: {}\n\
            - Environment: {}\n\
            - Token Status: {}",
            api_url, api_version, environment, token_status
        )
    }

    /// Makes a mock API call using configured variables
    fn make_api_call(_input: MakeApiCallInput) -> ToolResponse {
        // Get the API configuration from variables
        let api_url = match variables::get("api_url") {
            Ok(url) => url,
            Err(e) => return text!("Error: API URL not configured - {}", e),
        };
        
        let api_version = variables::get("api_version").unwrap_or_else(|_| "v1".to_string());
        
        // Try to get the auth token
        let token = match variables::get("api_token") {
            Ok(t) => t,
            Err(_) => return text!("Error: API token not configured. Set the api_token variable."),
        };
        
        // Construct the full API endpoint
        let endpoint = format!("{}/{}/data", api_url, api_version);
        
        // In a real app, you'd make an actual HTTP request here
        // For demo purposes, we just show what would be sent
        text!(
            "Would make API call with these settings:\n\
            - Endpoint: {}\n\
            - Authorization: Bearer {}...\n\
            - Content-Type: application/json\n\n\
            Note: This is a demo - no actual API call was made",
            endpoint,
            &token[0..6.min(token.len())]
        )
    }

    /// Shows different behavior based on the environment variable
    fn environment_check(_input: EnvironmentCheckInput) -> ToolResponse {
        let env = variables::get("environment").unwrap_or_else(|_| "development".to_string());
        
        let (debug_mode, log_level, caching, rate_limiting, detailed_errors) = match env.as_str() {
            "production" => (false, "error", true, true, false),
            "staging" => (true, "info", true, true, true),
            _ => (true, "debug", false, false, true),
        };
        
        text!(
            "Environment: {}\n\
            - Debug Mode: {}\n\
            - Log Level: {}\n\
            - Features:\n\
              * Caching: {}\n\
              * Rate Limiting: {}\n\
              * Detailed Errors: {}",
            env, debug_mode, log_level, caching, rate_limiting, detailed_errors
        )
    }
}