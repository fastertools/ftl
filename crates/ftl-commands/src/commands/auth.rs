//! Refactored auth command with dependency injection for better testability

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};

use ftl_core::deps::{MessageStyle, StoredCredentials, UserInterface};

/// Credentials provider trait
pub trait CredentialsProvider: Send + Sync {
    /// Get stored credentials from the credential store
    fn get_stored_credentials(&self) -> Result<StoredCredentials>;
}

/// Clock trait for time operations
pub trait Clock: Send + Sync {
    /// Get the current UTC time
    fn now(&self) -> DateTime<Utc>;
}

/// Dependencies for the auth command
pub struct AuthDependencies {
    /// User interface for output and interaction
    pub ui: Arc<dyn UserInterface>,
    /// Provider for accessing stored credentials
    pub credentials_provider: Arc<dyn CredentialsProvider>,
    /// Clock for time operations
    pub clock: Arc<dyn Clock>,
}

/// Execute the auth status command with injected dependencies
pub fn status_with_deps(deps: &Arc<AuthDependencies>) {
    deps.ui
        .print_styled("→ Authentication Status", MessageStyle::Cyan);
    deps.ui.print("");

    match deps.credentials_provider.get_stored_credentials() {
        Ok(credentials) => {
            deps.ui.print(&format!(
                "✅ {}",
                styled_text("Logged in", MessageStyle::Success)
            ));
            deps.ui.print("");
            deps.ui.print(&format!(
                "AuthKit Domain: {}",
                styled_text(&credentials.authkit_domain, MessageStyle::Cyan)
            ));

            if let Some(expires_at) = credentials.expires_at {
                let now = deps.clock.now();
                if expires_at < now {
                    deps.ui.print(&format!(
                        "Access Token: ⚠️  {}",
                        styled_text("Expired", MessageStyle::Yellow)
                    ));
                } else {
                    let duration = expires_at - now;
                    let hours = duration.num_hours();
                    let minutes = duration.num_minutes() % 60;
                    deps.ui.print(&format!(
                        "Access Token: Valid for {}h {}m",
                        styled_text(&hours.to_string(), MessageStyle::Success),
                        styled_text(&minutes.to_string(), MessageStyle::Success)
                    ));
                }
            } else {
                deps.ui.print(&format!(
                    "Access Token: {}",
                    styled_text("Valid", MessageStyle::Success)
                ));
            }

            if credentials.refresh_token.is_some() {
                deps.ui.print(&format!(
                    "Refresh Token: {}",
                    styled_text("Available", MessageStyle::Success)
                ));
            }
        }
        Err(e) => {
            if e.to_string().contains("No matching entry found") {
                deps.ui.print("🔐 Not logged in");
                deps.ui.print("");
                deps.ui.print(&format!(
                    "Run {} to authenticate",
                    styled_text("ftl login", MessageStyle::Cyan)
                ));
            } else {
                deps.ui.print("⚠️  Error checking authentication status");
                deps.ui.print("");
                deps.ui.print(&format!(
                    "Run {} to re-authenticate",
                    styled_text("ftl login", MessageStyle::Cyan)
                ));
            }
        }
    }
}

// Helper function to format styled text (since we're not using console crate directly)
const fn styled_text(text: &str, _style: MessageStyle) -> &str {
    text
}

/// Auth command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct AuthArgs {
    /// Subcommand: status
    pub command: AuthCommand,
}

/// Auth subcommands
#[derive(Debug, Clone)]
pub enum AuthCommand {
    /// Show authentication status
    Status,
}

// Real credentials provider implementation
struct RealCredentialsProviderWrapper;

impl CredentialsProvider for RealCredentialsProviderWrapper {
    fn get_stored_credentials(&self) -> Result<StoredCredentials> {
        use keyring::Entry;

        let entry = Entry::new("ftl-cli", "default")
            .map_err(|e| anyhow::anyhow!("Failed to access keyring: {}", e))?;

        let stored_json = entry
            .get_password()
            .map_err(|e| anyhow::anyhow!("No matching entry found: {}", e))?;

        serde_json::from_str(&stored_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse credentials: {}", e))
    }
}

// Real clock implementation
struct RealClockWrapper;

impl Clock for RealClockWrapper {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Execute the auth command with default dependencies
#[allow(clippy::unused_async)]
pub async fn execute(args: AuthArgs) -> Result<()> {
    use ftl_common::RealUserInterface;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(AuthDependencies {
        ui: ui.clone(),
        credentials_provider: Arc::new(RealCredentialsProviderWrapper),
        clock: Arc::new(RealClockWrapper),
    });

    match args.command {
        AuthCommand::Status => {
            status_with_deps(&deps);
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
