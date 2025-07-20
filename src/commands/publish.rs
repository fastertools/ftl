//! Refactored publish command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;

use crate::deps::{FileSystem, MessageStyle, UserInterface};

/// Build executor trait
#[async_trait::async_trait]
pub trait BuildExecutor: Send + Sync {
    async fn execute(&self, path: Option<PathBuf>, release: bool) -> Result<()>;
}

/// Spin installer trait
#[async_trait::async_trait]
pub trait SpinInstaller: Send + Sync {
    async fn check_and_install_spin(&self) -> Result<PathBuf>;
}

/// Process executor trait for running commands with working directory
pub trait ProcessExecutor: Send + Sync {
    fn execute(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<ProcessOutput>;
}

/// Process execution output
pub struct ProcessOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Configuration for the publish command
pub struct PublishConfig {
    pub path: Option<PathBuf>,
    pub registry: Option<String>,
    pub tag: Option<String>,
}

/// Dependencies for the publish command
pub struct PublishDependencies {
    pub ui: Arc<dyn UserInterface>,
    pub file_system: Arc<dyn FileSystem>,
    pub process_executor: Arc<dyn ProcessExecutor>,
    pub spin_installer: Arc<dyn SpinInstaller>,
    pub build_executor: Arc<dyn BuildExecutor>,
}

/// Execute the publish command with injected dependencies
pub async fn execute_with_deps(
    config: PublishConfig,
    deps: Arc<PublishDependencies>,
) -> Result<()> {
    let project_path = config.path.unwrap_or_else(|| PathBuf::from("."));

    deps.ui
        .print_styled("→ Publishing project", MessageStyle::Cyan);

    // Validate we're in a Spin project directory
    let spin_toml_path = project_path.join("spin.toml");
    if !deps.file_system.exists(&spin_toml_path) {
        anyhow::bail!(
            "No spin.toml found. Not in a project directory? Run 'ftl init' to create a new project."
        );
    }

    // Install spin if needed
    let spin_path = deps.spin_installer.check_and_install_spin().await?;

    // Build the project first
    deps.ui.print("→ Building project...");
    deps.build_executor
        .execute(Some(project_path.clone()), true)
        .await?;

    // Prepare registry push arguments
    let mut args = vec!["registry", "push"];

    if let Some(registry_url) = config.registry.as_ref() {
        args.push("--registry");
        args.push(registry_url);
    }

    if let Some(version_tag) = config.tag.as_ref() {
        args.push("--tag");
        args.push(version_tag);
    }

    deps.ui.print("→ Publishing to registry...");

    let output =
        deps.process_executor
            .execute(&spin_path.to_string_lossy(), &args, Some(&project_path))?;

    if !output.success {
        anyhow::bail!("Publishing failed:\n{}\n{}", output.stdout, output.stderr);
    }

    deps.ui
        .print_styled("✓ Project published successfully!", MessageStyle::Success);

    // Print any useful output from spin
    if !output.stdout.trim().is_empty() {
        deps.ui.print(&output.stdout);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_config_defaults() {
        let config = PublishConfig {
            path: None,
            registry: None,
            tag: None,
        };
        assert!(config.path.is_none());
    }
}
