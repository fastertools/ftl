//! Refactored auth command with dependency injection for better testability

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::deps::{MessageStyle, UserInterface};

/// Stored credentials structure
#[derive(Debug, Clone)]
pub struct StoredCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub authkit_domain: String,
}

/// Credentials provider trait
pub trait CredentialsProvider: Send + Sync {
    fn get_stored_credentials(&self) -> Result<StoredCredentials>;
}

/// Clock trait for time operations
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// Dependencies for the auth command
pub struct AuthDependencies {
    pub ui: Arc<dyn UserInterface>,
    pub credentials_provider: Arc<dyn CredentialsProvider>,
    pub clock: Arc<dyn Clock>,
}

/// Execute the auth status command with injected dependencies
pub async fn status_with_deps(deps: Arc<AuthDependencies>) -> Result<()> {
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

            Ok(())
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
            Ok(())
        }
    }
}

// Helper function to format styled text (since we're not using console crate directly)
fn styled_text(text: &str, _style: MessageStyle) -> &str {
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_text() {
        assert_eq!(styled_text("test", MessageStyle::Success), "test");
    }
}
