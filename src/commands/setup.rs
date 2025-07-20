//! Refactored setup command with dependency injection for better testability

use std::path::PathBuf;
use std::process::Output;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::deps::{MessageStyle, UserInterface};

/// Spin installer trait
pub trait SpinInstaller: Send + Sync {
    fn check_and_install(&self) -> Result<PathBuf>;
    fn get_spin_path(&self) -> Result<PathBuf>;
}

/// Command executor trait for setup command
pub trait SetupCommandExecutor: Send + Sync {
    fn execute(&self, command: &str, args: &[&str]) -> Result<Output>;
}

/// Environment trait for getting version info
pub trait Environment: Send + Sync {
    fn get_cargo_pkg_version(&self) -> &'static str;
}

/// Dependencies for the setup command
pub struct SetupDependencies {
    pub ui: Arc<dyn UserInterface>,
    pub spin_installer: Arc<dyn SpinInstaller>,
    pub command_executor: Arc<dyn SetupCommandExecutor>,
    pub environment: Arc<dyn Environment>,
}

/// Execute the templates subcommand with injected dependencies
pub fn templates_with_deps(
    force: bool,
    git: Option<&str>,
    branch: Option<&str>,
    dir: Option<&PathBuf>,
    tar: Option<&str>,
    deps: &Arc<SetupDependencies>,
) -> Result<()> {
    deps.ui
        .print_styled("→ Managing FTL templates", MessageStyle::Cyan);

    // Get spin path
    let spin_path = deps.spin_installer.check_and_install()?;
    let spin_str = spin_path.to_str().unwrap_or("spin");

    // Check if templates are already installed
    if !force {
        let list_output = deps
            .command_executor
            .execute(spin_str, &["templates", "list"])
            .context("Failed to list templates")?;

        let output_str = String::from_utf8_lossy(&list_output.stdout);
        let has_ftl_templates = output_str.contains("ftl-mcp-server")
            || output_str.contains("ftl-mcp-rust")
            || output_str.contains("ftl-mcp-ts");

        if has_ftl_templates {
            deps.ui.print_styled(
                "✓ ftl-mcp templates are already installed",
                MessageStyle::Success,
            );
            deps.ui.print("");
            deps.ui.print("Use --force to reinstall/update them");
            return Ok(());
        }
    }

    // Build install command based on provided options
    let mut args = vec!["templates", "install"];
    let source_info: String;

    if let Some(git_url) = git {
        source_info = format!("→ Installing templates from Git: {git_url}");
        args.push("--git");
        args.push(git_url);
        if let Some(branch_name) = branch {
            args.push("--branch");
            args.push(branch_name);
        }
    } else if let Some(dir_path) = &dir {
        source_info = format!(
            "→ Installing templates from directory: {}",
            dir_path.display()
        );
        let dir_str = dir_path.to_str().unwrap();
        args.push("--dir");
        args.push(dir_str);
    } else if let Some(tar_path) = tar {
        source_info = format!("→ Installing templates from tarball: {tar_path}");
        args.push("--tar");
        args.push(tar_path);
    } else {
        // Default: install from ftl-mcp repository
        let ftl_mcp_repo = "https://github.com/fastertools/ftl-mcp";
        source_info = format!("→ Installing ftl-mcp templates from {ftl_mcp_repo}");
        args.push("--git");
        args.push(ftl_mcp_repo);
    }

    deps.ui.print(&source_info);
    args.push("--upgrade");

    let install_output = deps
        .command_executor
        .execute(spin_str, &args)
        .context("Failed to install templates")?;

    if !install_output.status.success() {
        anyhow::bail!(
            "Failed to install templates:\n{}",
            String::from_utf8_lossy(&install_output.stderr)
        );
    }

    deps.ui
        .print_styled("✓ Templates installed successfully!", MessageStyle::Success);
    deps.ui.print("");

    // List installed ftl-mcp templates
    let list_output = deps
        .command_executor
        .execute(spin_str, &["templates", "list"])
        .context("Failed to list templates")?;

    let output_str = String::from_utf8_lossy(&list_output.stdout);
    deps.ui.print("Available ftl-mcp templates:");
    for line in output_str.lines() {
        if line.contains("ftl-mcp-") {
            deps.ui.print(&format!("  {}", line.trim()));
        }
    }

    Ok(())
}

/// Execute the info subcommand with injected dependencies
pub fn info_with_deps(deps: &Arc<SetupDependencies>) {
    deps.ui
        .print_styled("→ FTL Configuration", MessageStyle::Cyan);
    deps.ui.print("");

    // Show version
    deps.ui.print(&format!(
        "FTL CLI version: {}",
        deps.environment.get_cargo_pkg_version()
    ));
    deps.ui.print("");

    // Check spin installation
    if let Ok(spin_path) = deps.spin_installer.get_spin_path() {
        deps.ui.print(&format!(
            "Spin: {} {}",
            styled_text("✓", MessageStyle::Success),
            spin_path.display()
        ));

        // Get spin version
        if let Ok(output) = deps
            .command_executor
            .execute(spin_path.to_str().unwrap_or("spin"), &["--version"])
        {
            let version = String::from_utf8_lossy(&output.stdout);
            deps.ui.print(&format!("  Version: {}", version.trim()));
        }
    } else {
        deps.ui.print(&format!(
            "Spin: {} Not installed",
            styled_text("✗", MessageStyle::Error)
        ));
        deps.ui.print("  Run 'ftl setup templates' to install");
    }
    deps.ui.print("");

    // Check templates
    if let Ok(spin_path) = deps.spin_installer.get_spin_path() {
        if let Ok(output) = deps
            .command_executor
            .execute(spin_path.to_str().unwrap_or("spin"), &["templates", "list"])
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let ftl_templates: Vec<&str> = output_str
                .lines()
                .filter(|line| line.contains("ftl-mcp-"))
                .collect();

            if ftl_templates.is_empty() {
                deps.ui.print(&format!(
                    "ftl-mcp Templates: {} Not installed",
                    styled_text("✗", MessageStyle::Error)
                ));
                deps.ui.print("  Run 'ftl setup templates' to install");
            } else {
                deps.ui.print(&format!(
                    "ftl-mcp Templates: {} Installed",
                    styled_text("✓", MessageStyle::Success)
                ));
                for template in ftl_templates {
                    deps.ui.print(&format!("  - {}", template.trim()));
                }
            }
        }
    }
    deps.ui.print("");

    // Check for cargo-component
    match deps
        .command_executor
        .execute("cargo", &["component", "--version"])
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            deps.ui.print(&format!(
                "cargo-component: {} {}",
                styled_text("✓", MessageStyle::Success),
                version.trim()
            ));
        }
        _ => {
            deps.ui.print(&format!(
                "cargo-component: {} Not installed",
                styled_text("✗", MessageStyle::Error)
            ));
            deps.ui.print("  Required for building Rust components");
            deps.ui
                .print("  Will be installed automatically when building Rust components");
        }
    }
    deps.ui.print("");

    // Check for wkg
    match deps.command_executor.execute("wkg", &["--version"]) {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            deps.ui.print(&format!(
                "wkg: {} {}",
                styled_text("✓", MessageStyle::Success),
                version.trim()
            ));
        }
        _ => {
            deps.ui.print(&format!(
                "wkg: {} Not installed",
                styled_text("✗", MessageStyle::Error)
            ));
            deps.ui.print("  Required for 'ftl publish'");
            deps.ui
                .print("  Install from: https://github.com/bytecodealliance/wasm-pkg-tools");
        }
    }
}

// Helper function to format styled text (since we're not using console crate directly)
const fn styled_text(text: &str, _style: MessageStyle) -> &str {
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
