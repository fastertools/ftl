//! Refactored spin installer with dependency injection for better testability

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::deps::{CommandExecutor, UserInterface, MessageStyle};

/// Spin installer trait (already defined in deps.rs)
use crate::deps::SpinInstaller;

/// Helper function to check and install spin using default dependencies
pub async fn check_and_install_spin() -> Result<PathBuf> {
    let ui = Arc::new(crate::ui::RealUserInterface);
    let deps = Arc::new(SpinInstallerDependencies {
        command_executor: Arc::new(crate::deps::RealCommandExecutor),
        ui: ui.clone(),
    });
    
    let installer = RealSpinInstallerV2::new(deps);
    let path = installer.check_and_install().await?;
    Ok(PathBuf::from(path))
}

/// Dependencies for the spin installer
pub struct SpinInstallerDependencies {
    pub command_executor: Arc<dyn CommandExecutor>,
    pub ui: Arc<dyn UserInterface>,
}

/// Production implementation of SpinInstaller
pub struct RealSpinInstallerV2 {
    deps: Arc<SpinInstallerDependencies>,
}

impl RealSpinInstallerV2 {
    pub fn new(deps: Arc<SpinInstallerDependencies>) -> Self {
        Self { deps }
    }
}

#[async_trait::async_trait]
impl SpinInstaller for RealSpinInstallerV2 {
    async fn check_and_install(&self) -> Result<String> {
        // Check if spin is available in PATH
        match self.deps.command_executor.check_command_exists("spin").await {
            Ok(_) => {
                // Spin exists, ensure akamai plugin is installed
                self.ensure_akamai_plugin().await?;
                Ok("spin".to_string())
            }
            Err(_) => {
                // Spin not found - emit warning
                self.deps.ui.print_styled("⚠️  FTL requires Spin to run WebAssembly tools.", MessageStyle::Warning);
                self.deps.ui.print("Please install Spin from: https://github.com/fermyon/spin");
                self.deps.ui.print("Or use your package manager (e.g., brew install fermyon/tap/spin)");
                
                anyhow::bail!("Spin not found. Please install it from https://github.com/fermyon/spin")
            }
        }
    }
}

impl RealSpinInstallerV2 {
    async fn ensure_akamai_plugin(&self) -> Result<()> {
        // Check if Akamai plugin is installed
        let output = self.deps.command_executor
            .execute("spin", &["plugin", "list"])
            .await
            .context("Failed to list Spin plugins")?;

        if output.success {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("aka") {
                return Ok(());
            }
        }

        // Install the plugin
        self.deps.ui.print("Installing Akamai plugin for Spin...");
        let install_output = self.deps.command_executor
            .execute("spin", &["plugin", "install", "aka"])
            .await
            .context("Failed to install Akamai plugin")?;

        if !install_output.success {
            let stderr = String::from_utf8_lossy(&install_output.stderr);
            self.deps.ui.print_styled(
                &format!("⚠️  Warning: Failed to install Akamai plugin: {}", stderr),
                MessageStyle::Warning
            );
            self.deps.ui.print("   You can install it manually with: spin plugin install aka");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spin_installer_creation() {
        use crate::test_helpers::*;
        use crate::ui::TestUserInterface;
        
        let deps = Arc::new(SpinInstallerDependencies {
            command_executor: Arc::new(MockCommandExecutorMock::new()),
            ui: Arc::new(TestUserInterface::new()),
        });
        
        let _installer = RealSpinInstallerV2::new(deps);
        // Just verify it can be created
    }
}

#[cfg(test)]
#[path = "spin_installer_tests_akamai.rs"]
mod akamai_tests;