//! Refactored update command with dependency injection for better testability

use std::sync::Arc;

use anyhow::{Context, Result};
use semver::Version;

use crate::deps::{MessageStyle, UserInterface};

/// HTTP client trait for version checking
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str, user_agent: &str) -> Result<String>;
}

/// Command executor trait for running cargo install
pub trait CommandExecutor: Send + Sync {
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput>;
}

/// Command execution output
pub struct CommandOutput {
    pub success: bool,
    pub stderr: Vec<u8>,
}

/// Environment trait for getting current version
pub trait Environment: Send + Sync {
    fn get_cargo_pkg_version(&self) -> &'static str;
}

/// Dependencies for the update command
pub struct UpdateDependencies {
    pub ui: Arc<dyn UserInterface>,
    pub http_client: Arc<dyn HttpClient>,
    pub command_executor: Arc<dyn CommandExecutor>,
    pub environment: Arc<dyn Environment>,
}

/// Execute the update command with injected dependencies
pub async fn execute_with_deps(force: bool, deps: Arc<UpdateDependencies>) -> Result<()> {
    deps.ui
        .print_styled("→ Updating FTL CLI", MessageStyle::Cyan);

    let current_version = deps.environment.get_cargo_pkg_version();
    deps.ui
        .print(&format!("Current version: {current_version}"));

    if !force {
        // Check if we're already on the latest version
        match get_latest_version(&deps).await {
            Ok(latest_version) => {
                let current = Version::parse(current_version)?;
                let latest = Version::parse(&latest_version)?;

                if current >= latest {
                    deps.ui.print(&format!(
                        "{} Already on latest version ({})",
                        styled_text("✓", MessageStyle::Success),
                        current_version
                    ));
                    return Ok(());
                }

                deps.ui.print(&format!(
                    "Latest version available: {}",
                    styled_text(&latest_version, MessageStyle::Success)
                ));
            }
            Err(_) => {
                deps.ui.print(&format!(
                    "{} Could not check for latest version, proceeding with update",
                    styled_text("⚠", MessageStyle::Yellow)
                ));
            }
        }
    }

    deps.ui.print("→ Installing latest version...");

    // Use cargo install to update to latest version
    let install_output = deps
        .command_executor
        .execute("cargo", &["install", "ftl-cli", "--force"])?;

    if !install_output.success {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        anyhow::bail!("Failed to update FTL CLI:\n{}", stderr);
    }

    deps.ui.print(&format!(
        "{} FTL CLI updated successfully!",
        styled_text("✓", MessageStyle::Success)
    ));
    deps.ui.print("");
    deps.ui
        .print("Run 'ftl --version' to verify the new version");

    Ok(())
}

async fn get_latest_version(deps: &Arc<UpdateDependencies>) -> Result<String> {
    let url = "https://crates.io/api/v1/crates/ftl-cli";
    let user_agent = format!("ftl-cli/{}", deps.environment.get_cargo_pkg_version());

    let response = deps.http_client.get(url, &user_agent).await?;

    let json: serde_json::Value =
        serde_json::from_str(&response).context("Failed to parse crates.io response")?;

    let latest_version = json
        .get("crate")
        .and_then(|c| c.get("newest_version"))
        .and_then(|v| v.as_str())
        .context("Could not parse latest version from crates.io response")?;

    Ok(latest_version.to_string())
}

// Helper function to format styled text (since we're not using console crate directly)
const fn styled_text(text: &str, _style: MessageStyle) -> &str {
    text
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
