//! Refactored setup command with dependency injection for better testability

use std::path::PathBuf;
use std::process::Output;
use std::sync::Arc;

use anyhow::{Context, Result};

use ftl_runtime::deps::{MessageStyle, UserInterface};

/// Setup-specific spin installer trait
pub trait SetupSpinInstaller: Send + Sync {
    /// Check if Spin is installed and install if needed
    fn check_and_install(&self) -> Result<PathBuf>;
    /// Get the path to the Spin executable
    fn get_spin_path(&self) -> Result<PathBuf>;
}

/// Command executor trait for setup command
pub trait SetupCommandExecutor: Send + Sync {
    /// Execute a command and return its output
    fn execute(&self, command: &str, args: &[&str]) -> Result<Output>;
}

/// Environment trait for getting version info
pub trait Environment: Send + Sync {
    /// Get the cargo package version
    fn get_cargo_pkg_version(&self) -> &'static str;
}

/// Dependencies for the setup command
pub struct SetupDependencies {
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// Spin installer
    pub spin_installer: Arc<dyn SetupSpinInstaller>,
    /// Command executor
    pub command_executor: Arc<dyn SetupCommandExecutor>,
    /// Environment information
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
            || output_str.contains("ftl-mcp-ts")
            || output_str.contains("ftl-mcp-python")
            || output_str.contains("ftl-mcp-go");

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
        let ftl_mcp_repo = "https://github.com/fastertools/ftl-cli";
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
    if let Ok(spin_path) = deps.spin_installer.get_spin_path()
        && let Ok(output) = deps
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

/// Setup command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct SetupArgs {
    /// Subcommand
    pub command: SetupCommand,
}

/// Setup subcommands
#[derive(Debug, Clone)]
pub enum SetupCommand {
    /// Install and manage FTL templates
    Templates {
        /// Force reinstall even if templates exist
        force: bool,
        /// Install from a Git repository
        git: Option<String>,
        /// Git branch to use
        branch: Option<String>,
        /// Install from a local directory
        dir: Option<PathBuf>,
        /// Install from a tarball
        tar: Option<String>,
    },
    /// Show FTL configuration info
    Info,
}

// Real spin installer wrapper
struct RealSetupSpinInstaller;

impl SetupSpinInstaller for RealSetupSpinInstaller {
    fn check_and_install(&self) -> Result<PathBuf> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { ftl_common::check_and_install_spin().await })
        })
    }

    fn get_spin_path(&self) -> Result<PathBuf> {
        match which::which("spin") {
            Ok(path) => Ok(path),
            Err(_) => anyhow::bail!("Spin not found in PATH"),
        }
    }
}

// Real command executor wrapper
struct RealSetupCommandExecutor;

impl SetupCommandExecutor for RealSetupCommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<Output> {
        use std::process::Command;

        Command::new(command)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))
    }
}

// Real environment wrapper
struct RealEnvironment;

impl Environment for RealEnvironment {
    fn get_cargo_pkg_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// Execute the setup command with default dependencies
#[allow(clippy::unused_async)]
pub async fn execute(args: SetupArgs) -> Result<()> {
    use ftl_common::RealUserInterface;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(SetupDependencies {
        ui: ui.clone(),
        spin_installer: Arc::new(RealSetupSpinInstaller),
        command_executor: Arc::new(RealSetupCommandExecutor),
        environment: Arc::new(RealEnvironment),
    });

    match args.command {
        SetupCommand::Templates {
            force,
            git,
            branch,
            dir,
            tar,
        } => templates_with_deps(
            force,
            git.as_deref(),
            branch.as_deref(),
            dir.as_ref(),
            tar.as_deref(),
            &deps,
        ),
        SetupCommand::Info => {
            info_with_deps(&deps);
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "setup_tests.rs"]
mod tests;
