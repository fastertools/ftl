//! Refactored publish command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;

use ftl_common::SpinInstaller;
use ftl_runtime::deps::{FileSystem, MessageStyle, UserInterface};

/// Build executor trait
#[async_trait::async_trait]
pub trait BuildExecutor: Send + Sync {
    /// Execute a build with optional path and release mode
    async fn execute(&self, path: Option<PathBuf>, release: bool) -> Result<()>;
}

/// Process executor trait for running commands with working directory
pub trait ProcessExecutor: Send + Sync {
    /// Execute a command with optional working directory
    fn execute(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<ProcessOutput>;
}

/// Process execution output
pub struct ProcessOutput {
    /// Whether the process exited successfully
    pub success: bool,
    /// Standard output from the process
    pub stdout: String,
    /// Standard error from the process
    pub stderr: String,
}

/// Configuration for the publish command
pub struct PublishConfig {
    /// Path to the toolbox
    pub path: Option<PathBuf>,
    /// Registry URL to publish to
    pub registry: Option<String>,
    /// Version tag for the published package
    pub tag: Option<String>,
}

/// Dependencies for the publish command
pub struct PublishDependencies {
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// File system operations
    pub file_system: Arc<dyn FileSystem>,
    /// Process executor for running commands
    pub process_executor: Arc<dyn ProcessExecutor>,
    /// Spin CLI installer
    pub spin_installer: Arc<dyn SpinInstaller>,
    /// Build executor for building before publish
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

    // For deploy and publish, we need actual spin.toml in the project directory  
    // since these commands package the project for upload
    if deps.file_system.exists(&project_path.join("ftl.toml")) {
        crate::config::transpiler::ensure_spin_toml(&deps.file_system, &project_path)?;
    }

    // Validate we're in a Spin project directory
    let spin_toml_path = project_path.join("spin.toml");
    if !deps.file_system.exists(&spin_toml_path) {
        anyhow::bail!(
            "No spin.toml or ftl.toml found. Not in a project directory? Run 'ftl init' to create a new project."
        );
    }

    // Install spin if needed
    let spin_path = deps.spin_installer.check_and_install().await?;

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

    let output = deps
        .process_executor
        .execute(&spin_path, &args, Some(&project_path))?;

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

/// Publish command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct PublishArgs {
    /// Path to the toolbox
    pub path: Option<PathBuf>,
    /// Registry to publish to
    pub registry: Option<String>,
    /// Version tag for the published package
    pub tag: Option<String>,
}

// Build executor wrapper
struct BuildExecutorWrapper;

#[async_trait::async_trait]
impl BuildExecutor for BuildExecutorWrapper {
    async fn execute(&self, path: Option<PathBuf>, release: bool) -> Result<()> {
        use crate::commands::build;

        let args = build::BuildArgs { path, release };

        build::execute(args).await
    }
}

// Process executor wrapper
struct ProcessExecutorWrapper;

impl ProcessExecutor for ProcessExecutorWrapper {
    fn execute(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<ProcessOutput> {
        use std::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?;

        Ok(ProcessOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

// Spin installer wrapper
struct SpinInstallerWrapper;

#[async_trait::async_trait]
impl SpinInstaller for SpinInstallerWrapper {
    async fn check_and_install(&self) -> Result<String> {
        let path = ftl_common::check_and_install_spin().await?;
        Ok(path.to_string_lossy().to_string())
    }
}

/// Execute the publish command with default dependencies
pub async fn execute(args: PublishArgs) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_runtime::deps::RealFileSystem;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(PublishDependencies {
        ui: ui.clone(),
        file_system: Arc::new(RealFileSystem),
        process_executor: Arc::new(ProcessExecutorWrapper),
        spin_installer: Arc::new(SpinInstallerWrapper),
        build_executor: Arc::new(BuildExecutorWrapper),
    });

    let config = PublishConfig {
        path: args.path,
        registry: args.registry,
        tag: args.tag,
    };

    execute_with_deps(config, deps).await
}

#[cfg(test)]
#[path = "publish_tests.rs"]
mod tests;
