//! Refactored logout command with dependency injection for better testability

use std::sync::Arc;

use anyhow::Result;

use ftl_runtime::deps::{MessageStyle, UserInterface};

/// Credentials clearer trait
pub trait CredentialsClearer: Send + Sync {
    /// Clear stored credentials from the credential store
    fn clear_stored_credentials(&self) -> Result<()>;
}

/// Dependencies for the logout command
pub struct LogoutDependencies {
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// Credentials clearer for removing stored credentials
    pub credentials_clearer: Arc<dyn CredentialsClearer>,
}

/// Execute the logout command with injected dependencies
pub fn execute_with_deps(deps: &Arc<LogoutDependencies>) -> Result<()> {
    deps.ui
        .print_styled("→ Logging out of FTL", MessageStyle::Cyan);
    deps.ui.print("");

    match deps.credentials_clearer.clear_stored_credentials() {
        Ok(()) => {
            deps.ui.print(&format!(
                "✅ {} Successfully logged out!",
                styled_text("Success!", MessageStyle::Success)
            ));
            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("No matching entry found") {
                deps.ui.print("Not currently logged in.");
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

// Helper function to format styled text (since we're not using console crate directly)
const fn styled_text(text: &str, _style: MessageStyle) -> &str {
    text
}

/// Logout command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct LogoutArgs {
    // Logout takes no arguments
}

// Real credentials clearer wrapper
struct RealCredentialsClearer;

impl CredentialsClearer for RealCredentialsClearer {
    fn clear_stored_credentials(&self) -> Result<()> {
        use keyring::Entry;

        let entry = Entry::new("ftl-cli", "default")
            .map_err(|e| anyhow::anyhow!("Failed to access keyring: {}", e))?;

        entry
            .delete_credential()
            .map_err(|e| anyhow::anyhow!("Failed to clear credentials: {}", e))
    }
}

/// Execute the logout command with default dependencies
#[allow(clippy::unused_async)]
pub async fn execute(_args: LogoutArgs) -> Result<()> {
    use ftl_common::RealUserInterface;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(LogoutDependencies {
        ui: ui.clone(),
        credentials_clearer: Arc::new(RealCredentialsClearer),
    });

    execute_with_deps(&deps)
}

#[cfg(test)]
#[path = "logout_tests.rs"]
mod tests;
