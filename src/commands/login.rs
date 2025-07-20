//! Refactored login command with dependency injection for better testability

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::deps::{
    AsyncRuntime, MessageStyle, UserInterface
};

pub const CLIENT_ID: &str = "client_01K06E1DRP26N8A3T9CGMB1YSP";
pub const AUTHKIT_DOMAIN: &str = "divine-lion-50-staging.authkit.app";
pub const LOGIN_TIMEOUT: Duration = Duration::from_secs(60 * 30); // 30 minutes

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u64,
    pub interval: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenError {
    pub error: String,
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub authkit_domain: String,
}

/// HTTP client trait for making requests
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn post(&self, url: &str, body: &str) -> Result<HttpResponse>;
}

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
}

/// Keyring storage trait
pub trait KeyringStorage: Send + Sync {
    fn store(&self, service: &str, username: &str, password: &str) -> Result<()>;
    fn retrieve(&self, service: &str, username: &str) -> Result<String>;
    fn delete(&self, service: &str, username: &str) -> Result<()>;
}

/// Browser launcher trait
pub trait BrowserLauncher: Send + Sync {
    fn open(&self, url: &str) -> Result<()>;
}

/// Clock trait for time operations
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
    fn instant_now(&self) -> std::time::Instant;
}

/// Login configuration
pub struct LoginConfig {
    pub no_browser: bool,
    pub authkit_domain: Option<String>,
    pub client_id: Option<String>,
}

/// Dependencies for the login command
pub struct LoginDependencies {
    pub ui: Arc<dyn UserInterface>,
    pub http_client: Arc<dyn HttpClient>,
    pub keyring: Arc<dyn KeyringStorage>,
    pub browser_launcher: Arc<dyn BrowserLauncher>,
    pub async_runtime: Arc<dyn AsyncRuntime>,
    pub clock: Arc<dyn Clock>,
}

/// Execute the login command with injected dependencies
pub async fn execute_with_deps(
    config: LoginConfig,
    deps: Arc<LoginDependencies>,
) -> Result<()> {
    let authkit_domain = config.authkit_domain.as_deref().unwrap_or(AUTHKIT_DOMAIN);
    let client_id = config.client_id.as_deref().unwrap_or(CLIENT_ID);

    deps.ui.print(&format!("→ Logging in to FTL ({})", authkit_domain));
    deps.ui.print("");

    // Request device authorization
    let auth_response = request_device_authorization(
        &deps.http_client,
        authkit_domain,
        client_id,
    ).await?;

    // Display login instructions
    deps.ui.print("");
    deps.ui.print("🌐 To complete login, visit:");
    deps.ui.print(&format!("   {}", auth_response.verification_uri));
    deps.ui.print("");
    deps.ui.print("And enter this code:");
    deps.ui.print_styled(&format!("   {}", auth_response.user_code), MessageStyle::Success);
    deps.ui.print("");

    // Optionally open browser
    if !config.no_browser && deps.ui.is_interactive() {
        if deps.ui.prompt_select("Open browser automatically?", &["Yes", "No"], 0)? == 0 {
            deps.browser_launcher.open(&auth_response.verification_uri_complete)?;
        }
    }

    // Poll for token
    let token_response = poll_for_token(
        &deps.http_client,
        &deps.ui,
        &deps.async_runtime,
        &deps.clock,
        authkit_domain,
        client_id,
        &auth_response.device_code,
        auth_response.interval.unwrap_or(5),
    ).await?;

    // Store credentials
    store_credentials(
        &deps.keyring,
        &deps.clock,
        authkit_domain,
        &token_response,
    )?;

    deps.ui.print("");
    deps.ui.print_styled("✅ Success! Successfully logged in!", MessageStyle::Success);

    Ok(())
}

async fn request_device_authorization(
    http_client: &Arc<dyn HttpClient>,
    authkit_domain: &str,
    client_id: &str,
) -> Result<DeviceAuthResponse> {
    let url = format!("https://{authkit_domain}/oauth2/device_authorization");
    let body = format!(
        "client_id={client_id}&scope=openid%20email%20profile%20offline_access"
    );

    let response = http_client.post(&url, &body).await
        .context("Failed to request device authorization")?;

    if !response.is_success() {
        return Err(anyhow!("Device authorization failed: {}", response.body));
    }

    serde_json::from_str(&response.body)
        .context("Failed to parse device authorization response")
}

async fn poll_for_token(
    http_client: &Arc<dyn HttpClient>,
    ui: &Arc<dyn UserInterface>,
    async_runtime: &Arc<dyn AsyncRuntime>,
    clock: &Arc<dyn Clock>,
    authkit_domain: &str,
    client_id: &str,
    device_code: &str,
    poll_interval_secs: u64,
) -> Result<TokenResponse> {
    let url = format!("https://{authkit_domain}/oauth2/token");
    
    let pb = ui.create_spinner();
    pb.set_message("Waiting for authorization...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let start = clock.instant_now();
    let mut interval_secs = poll_interval_secs;

    loop {
        if start.elapsed() > LOGIN_TIMEOUT {
            pb.finish_and_clear();
            return Err(anyhow!("Login timeout - please try again"));
        }

        async_runtime.sleep(Duration::from_secs(interval_secs)).await;

        let body = format!(
            "grant_type=urn:ietf:params:oauth:grant-type:device_code&device_code={}&client_id={}",
            device_code, client_id
        );

        let response = http_client.post(&url, &body).await
            .context("Failed to poll for token")?;

        if response.is_success() {
            pb.finish_and_clear();
            return serde_json::from_str(&response.body)
                .context("Failed to parse token response");
        }

        // Handle error responses
        if let Ok(error) = serde_json::from_str::<TokenError>(&response.body) {
            match error.error.as_str() {
                "authorization_pending" => continue,
                "slow_down" => {
                    // Increase interval
                    interval_secs = poll_interval_secs + 5;
                }
                "access_denied" => {
                    pb.finish_and_clear();
                    return Err(anyhow!("Access denied by user"));
                }
                "expired_token" => {
                    pb.finish_and_clear();
                    return Err(anyhow!("Device code expired - please try again"));
                }
                _ => {
                    pb.finish_and_clear();
                    return Err(anyhow!(
                        "Token error: {} - {}",
                        error.error,
                        error.error_description.unwrap_or_default()
                    ));
                }
            }
        }
    }
}

fn store_credentials(
    keyring: &Arc<dyn KeyringStorage>,
    clock: &Arc<dyn Clock>,
    authkit_domain: &str,
    token_response: &TokenResponse,
) -> Result<()> {
    let expires_at = token_response
        .expires_in
        .map(|expires_in| clock.now() + chrono::Duration::seconds(expires_in as i64));

    let credentials = StoredCredentials {
        access_token: token_response.access_token.clone(),
        refresh_token: token_response.refresh_token.clone(),
        id_token: token_response.id_token.clone(),
        expires_at,
        authkit_domain: authkit_domain.to_string(),
    };

    let json = serde_json::to_string(&credentials)?;
    keyring.store("ftl-cli", "default", &json)?;

    Ok(())
}

pub fn get_stored_credentials_with_deps(
    keyring: &Arc<dyn KeyringStorage>,
) -> Result<StoredCredentials> {
    let json = keyring.retrieve("ftl-cli", "default")?;
    let credentials: StoredCredentials = serde_json::from_str(&json)?;
    Ok(credentials)
}

/// Helper function to get stored credentials using default keyring
pub fn get_stored_credentials() -> Result<Option<StoredCredentials>> {
    let entry = keyring::Entry::new("ftl-cli", "default")?;
    match entry.get_password() {
        Ok(json) => {
            let credentials: StoredCredentials = serde_json::from_str(&json)?;
            Ok(Some(credentials))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Failed to retrieve credentials: {}", e)),
    }
}

/// Helper function to get or refresh credentials using default implementations
pub async fn get_or_refresh_credentials() -> Result<StoredCredentials> {
    // Create real implementations
    struct RealKeyringStorage;
    impl KeyringStorage for RealKeyringStorage {
        fn store(&self, service: &str, username: &str, password: &str) -> Result<()> {
            let entry = keyring::Entry::new(service, username)?;
            entry.set_password(password)?;
            Ok(())
        }
        
        fn retrieve(&self, service: &str, username: &str) -> Result<String> {
            let entry = keyring::Entry::new(service, username)?;
            entry.get_password()
                .map_err(|e| anyhow::anyhow!("Failed to retrieve credentials: {}", e))
        }
        
        fn delete(&self, service: &str, username: &str) -> Result<()> {
            let entry = keyring::Entry::new(service, username)?;
            entry.delete_credential()?;
            Ok(())
        }
    }
    
    struct RealHttpClient;
    #[async_trait::async_trait]
    impl HttpClient for RealHttpClient {
        async fn post(&self, url: &str, body: &str) -> Result<HttpResponse> {
            let response = reqwest::Client::new()
                .post(url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body.to_string())
                .send()
                .await?;
            
            let status = response.status().as_u16();
            let body = response.text().await?;
            
            Ok(HttpResponse { status, body })
        }
    }
    
    struct RealClock;
    impl Clock for RealClock {
        fn now(&self) -> DateTime<Utc> {
            Utc::now()
        }
        
        fn instant_now(&self) -> std::time::Instant {
            std::time::Instant::now()
        }
    }
    
    let keyring: Arc<dyn KeyringStorage> = Arc::new(RealKeyringStorage);
    let http_client: Arc<dyn HttpClient> = Arc::new(RealHttpClient);
    let clock: Arc<dyn Clock> = Arc::new(RealClock);
    
    get_or_refresh_credentials_with_deps(&keyring, &http_client, &clock).await
}

pub async fn get_or_refresh_credentials_with_deps(
    keyring: &Arc<dyn KeyringStorage>,
    http_client: &Arc<dyn HttpClient>,
    clock: &Arc<dyn Clock>,
) -> Result<StoredCredentials> {
    let json = keyring.retrieve("ftl-cli", "default")?;
    let mut credentials: StoredCredentials = serde_json::from_str(&json)?;

    // Check if token is expired or about to expire (within 30 seconds)
    if let Some(expires_at) = credentials.expires_at {
        let now = clock.now();
        let buffer = chrono::Duration::seconds(30);

        if expires_at < now + buffer {
            // Token is expired or about to expire, try to refresh
            if let Some(refresh_token) = credentials.refresh_token.clone() {
                match refresh_access_token(
                    http_client,
                    &credentials.authkit_domain,
                    &refresh_token,
                ).await {
                    Ok(new_tokens) => {
                        // Update credentials with new tokens
                        credentials.access_token = new_tokens.access_token;
                        credentials.expires_at = new_tokens
                            .expires_in
                            .map(|expires_in| now + chrono::Duration::seconds(expires_in as i64));

                        // Update refresh token if a new one was provided
                        if let Some(new_refresh) = new_tokens.refresh_token {
                            credentials.refresh_token = Some(new_refresh);
                        }

                        // Save updated credentials
                        let updated_json = serde_json::to_string(&credentials)?;
                        keyring.store("ftl-cli", "default", &updated_json)?;

                        return Ok(credentials);
                    }
                    Err(e) => {
                        return Err(anyhow!(
                            "Token refresh failed: {}. Please run 'ftl login' again.",
                            e
                        ));
                    }
                }
            } else {
                return Err(anyhow!(
                    "Authentication token has expired and no refresh token available. Please run 'ftl login' again."
                ));
            }
        }
    }

    Ok(credentials)
}

async fn refresh_access_token(
    http_client: &Arc<dyn HttpClient>,
    authkit_domain: &str,
    refresh_token: &str,
) -> Result<TokenResponse> {
    let url = format!("https://{authkit_domain}/oauth2/token");
    let client_id = CLIENT_ID; // Use default for refresh
    
    let body = format!(
        "grant_type=refresh_token&refresh_token={refresh_token}&client_id={client_id}"
    );

    let response = http_client.post(&url, &body).await
        .context("Failed to send refresh token request")?;

    if !response.is_success() {
        return Err(anyhow!(
            "Token refresh failed with status {}: {}",
            response.status,
            response.body
        ));
    }

    serde_json::from_str(&response.body)
        .context("Failed to parse refresh token response")
}

pub fn clear_stored_credentials_with_deps(
    keyring: &Arc<dyn KeyringStorage>,
) -> Result<()> {
    keyring.delete("ftl-cli", "default")?;
    Ok(())
}

/// Helper function to clear stored credentials using default keyring
pub fn clear_stored_credentials() -> Result<()> {
    let entry = keyring::Entry::new("ftl-cli", "default")?;
    entry.delete_credential()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_response_is_success() {
        let response = HttpResponse { status: 200, body: "ok".to_string() };
        assert!(response.is_success());
        
        let response = HttpResponse { status: 201, body: "created".to_string() };
        assert!(response.is_success());
        
        let response = HttpResponse { status: 400, body: "bad request".to_string() };
        assert!(!response.is_success());
        
        let response = HttpResponse { status: 500, body: "server error".to_string() };
        assert!(!response.is_success());
    }

    #[test]
    fn test_stored_credentials_serialization() {
        let creds = StoredCredentials {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            id_token: None,
            expires_at: Some(Utc::now()),
            authkit_domain: "test.domain".to_string(),
        };
        
        let json = serde_json::to_string(&creds).unwrap();
        let deserialized: StoredCredentials = serde_json::from_str(&json).unwrap();
        
        assert_eq!(creds.access_token, deserialized.access_token);
        assert_eq!(creds.refresh_token, deserialized.refresh_token);
        assert_eq!(creds.authkit_domain, deserialized.authkit_domain);
    }
}