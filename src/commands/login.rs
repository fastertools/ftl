use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use console::{Emoji, style};
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::time::interval;

const DEFAULT_CLIENT_ID: &str = "client_01JZM53FW3WYV08AFC4QWQ3BNB";
const DEFAULT_AUTHKIT_DOMAIN: &str = "auth.ftl.tools";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes
static CHECK: Emoji<'_, '_> = Emoji("✅", "");
static GLOBE: Emoji<'_, '_> = Emoji("🌐", "");

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: u64,
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenError {
    error: String,
    error_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub authkit_domain: String,
}

fn get_authkit_domain() -> String {
    // First check runtime env var
    if let Ok(domain) = env::var("FTL_AUTHKIT_DOMAIN") {
        return domain;
    }

    // Then check compile-time env var
    option_env!("FTL_AUTHKIT_DOMAIN")
        .unwrap_or(DEFAULT_AUTHKIT_DOMAIN)
        .to_string()
}

fn get_client_id() -> String {
    // First check runtime env var
    if let Ok(client_id) = env::var("FTL_CLIENT_ID") {
        return client_id;
    }

    // Then check compile-time env var
    option_env!("FTL_CLIENT_ID")
        .unwrap_or(DEFAULT_CLIENT_ID)
        .to_string()
}

pub async fn execute(no_browser: bool) -> Result<()> {
    let authkit_domain = get_authkit_domain();

    println!(
        "{} Logging in to FTL ({})",
        style("→").cyan(),
        style(&authkit_domain).bold()
    );
    println!();

    // Request device authorization
    let auth_response = request_device_authorization(&authkit_domain).await?;

    // Display login instructions
    println!();
    println!("{GLOBE} To complete login, visit:");
    println!(
        "   {}",
        style(&auth_response.verification_uri).cyan().bold()
    );
    println!();
    println!("And enter this code:");
    println!("   {}", style(&auth_response.user_code).green().bold());
    println!();

    // Optionally open browser
    if !no_browser
        && Confirm::new()
            .with_prompt("Open browser automatically?")
            .default(true)
            .interact()?
    {
        webbrowser::open(&auth_response.verification_uri_complete)?;
    }

    // Poll for token
    let token_response = poll_for_token(
        &authkit_domain,
        &auth_response.device_code,
        auth_response.interval.unwrap_or(5),
    )
    .await?;

    // Store credentials
    store_credentials(&authkit_domain, &token_response)?;

    println!();
    println!(
        "{} {} Successfully logged in!",
        CHECK,
        style("Success!").green().bold()
    );

    Ok(())
}

async fn request_device_authorization(authkit_domain: &str) -> Result<DeviceAuthResponse> {
    let client = reqwest::Client::new();
    // Use WorkOS Connect endpoint
    let url = format!("https://{authkit_domain}/oauth2/device_authorization");

    let client_id = get_client_id();
    let response = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "client_id={client_id}&scope=openid%20email%20profile"
        ))
        .send()
        .await
        .context("Failed to request device authorization")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Device authorization failed: {}", error_text));
    }

    response
        .json::<DeviceAuthResponse>()
        .await
        .context("Failed to parse device authorization response")
}

async fn poll_for_token(
    authkit_domain: &str,
    device_code: &str,
    poll_interval_secs: u64,
) -> Result<TokenResponse> {
    let client = reqwest::Client::new();
    // Use WorkOS Connect endpoint
    let url = format!("https://{authkit_domain}/oauth2/token");

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg} [{elapsed}]")?
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.set_message("Waiting for authorization...");

    let mut interval = interval(Duration::from_secs(poll_interval_secs));
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > LOGIN_TIMEOUT {
            pb.finish_and_clear();
            return Err(anyhow!("Login timeout - please try again"));
        }

        interval.tick().await;
        pb.tick();

        let response = client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type=urn:ietf:params:oauth:grant-type:device_code&device_code={}&client_id={}",
                device_code, get_client_id()
            ))
            .send()
            .await
            .context("Failed to poll for token")?;

        if response.status().is_success() {
            pb.finish_and_clear();
            return response
                .json::<TokenResponse>()
                .await
                .context("Failed to parse token response");
        }

        // Handle error responses
        if let Ok(error) = response.json::<TokenError>().await {
            match error.error.as_str() {
                "authorization_pending" => continue,
                "slow_down" => {
                    // Increase interval - recreate with longer duration
                    interval = tokio::time::interval(Duration::from_secs(poll_interval_secs + 5));
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

fn store_credentials(authkit_domain: &str, token_response: &TokenResponse) -> Result<()> {
    let expires_at = token_response
        .expires_in
        .map(|expires_in| Utc::now() + chrono::Duration::seconds(expires_in as i64));

    let credentials = StoredCredentials {
        access_token: token_response.access_token.clone(),
        refresh_token: token_response.refresh_token.clone(),
        id_token: token_response.id_token.clone(),
        expires_at,
        authkit_domain: authkit_domain.to_string(),
    };

    // Store in keyring
    let entry = keyring::Entry::new("ftl-cli", "default")?;
    let json = serde_json::to_string(&credentials)?;
    entry.set_password(&json)?;

    Ok(())
}

pub fn get_stored_credentials() -> Result<StoredCredentials> {
    let entry = keyring::Entry::new("ftl-cli", "default")?;
    let json = entry.get_password()?;
    let credentials: StoredCredentials = serde_json::from_str(&json)?;
    Ok(credentials)
}

pub fn clear_stored_credentials() -> Result<()> {
    let entry = keyring::Entry::new("ftl-cli", "default")?;
    entry.delete_credential()?;
    Ok(())
}
