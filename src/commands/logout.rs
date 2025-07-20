//! Refactored logout command with dependency injection for better testability

use std::sync::Arc;

use anyhow::Result;

use crate::deps::{MessageStyle, UserInterface};

/// Credentials clearer trait
pub trait CredentialsClearer: Send + Sync {
    fn clear_stored_credentials(&self) -> Result<()>;
}

/// Dependencies for the logout command
pub struct LogoutDependencies {
    pub ui: Arc<dyn UserInterface>,
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

#[cfg(test)]
#[path = "logout_tests.rs"]
mod tests;
