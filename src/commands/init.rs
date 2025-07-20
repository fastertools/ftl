//! Refactored init command with dependency injection for better testability

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, ensure};

use crate::deps::{CommandExecutor, FileSystem, SpinInstaller, UserInterface};

/// Init command configuration
pub struct InitConfig {
    pub name: Option<String>,
    pub here: bool,
}

/// Dependencies for the init command
pub struct InitDependencies {
    pub file_system: Arc<dyn FileSystem>,
    pub command_executor: Arc<dyn CommandExecutor>,
    pub ui: Arc<dyn UserInterface>,
    pub spin_installer: Arc<dyn SpinInstaller>,
}

/// Execute the init command with injected dependencies
pub async fn execute_with_deps(config: InitConfig, deps: Arc<InitDependencies>) -> Result<()> {
    let InitConfig { mut name, here } = config;

    // Install Spin if needed
    let spin_path = deps.spin_installer.check_and_install().await?;
    deps.ui.print(&format!("Using Spin at: {spin_path}"));

    // Get project name
    if name.is_none() && !here {
        name = Some(deps.ui.prompt_input("Project name", Some("my-project"))?);
    }

    // Validate name
    if let Some(ref project_name) = name {
        validate_project_name(project_name)?;
    }

    // Check directory
    let target_dir = if here {
        ".".to_string()
    } else {
        name.as_ref().unwrap().clone()
    };

    if !here && deps.file_system.exists(Path::new(&target_dir)) {
        anyhow::bail!("Directory '{}' already exists", target_dir);
    }

    if here && !is_directory_empty(&deps.file_system) {
        anyhow::bail!("Current directory is not empty. Use --here only in an empty directory.");
    }

    // Check templates are installed
    check_templates_installed(&deps.command_executor, &spin_path).await?;

    // Create project
    create_project(&deps.command_executor, &spin_path, &target_dir).await?;

    // Success message
    deps.ui.print("");
    deps.ui.print("✅ MCP project initialized!");
    deps.ui.print("");
    deps.ui.print("Next steps:");

    if !here {
        deps.ui.print(&format!("  cd {target_dir} &&"));
    }

    deps.ui
        .print("  ftl add           # Add a tool to the project");
    deps.ui.print("  ftl build         # Build the project");
    deps.ui
        .print("  ftl up            # Start local dev server");
    deps.ui.print("");
    deps.ui.print("The project will be available at:");
    deps.ui.print("  http://localhost:3000/mcp");
    deps.ui.print("");

    Ok(())
}

/// Validate project name
fn validate_project_name(name: &str) -> Result<()> {
    ensure!(!name.is_empty(), "Project name cannot be empty");

    ensure!(
        name.chars()
            .all(|c| c.is_lowercase() || c.is_numeric() || c == '-'),
        "Project name must be lowercase alphanumeric with hyphens"
    );

    ensure!(
        !name.starts_with('-') && !name.ends_with('-'),
        "Project name cannot start or end with hyphens"
    );

    ensure!(
        !name.contains("--"),
        "Project name cannot contain consecutive hyphens"
    );

    Ok(())
}

/// Check if current directory is empty
fn is_directory_empty(fs: &Arc<dyn FileSystem>) -> bool {
    let common_files = [
        "./Cargo.toml",
        "./package.json",
        "./spin.toml",
        "./.git",
        "./src",
        "./components",
        "./node_modules",
    ];

    for file in &common_files {
        if fs.exists(Path::new(file)) {
            return false;
        }
    }

    true
}

/// Check if ftl-mcp templates are installed
async fn check_templates_installed(
    executor: &Arc<dyn CommandExecutor>,
    spin_path: &str,
) -> Result<()> {
    let output = executor
        .execute(spin_path, &["templates", "list"])
        .await
        .context("Failed to list Spin templates")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("ftl-mcp-server") {
        anyhow::bail!("ftl-mcp templates not installed. Run 'ftl setup templates' first.");
    }

    Ok(())
}

/// Create the project using spin new
async fn create_project(
    executor: &Arc<dyn CommandExecutor>,
    spin_path: &str,
    target_dir: &str,
) -> Result<()> {
    let output = executor
        .execute(
            spin_path,
            &["new", "-t", "ftl-mcp-server", "-a", target_dir],
        )
        .await?;

    if !output.success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create project: {}", stderr);
    }

    Ok(())
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
