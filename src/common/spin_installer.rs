use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

/// Get the path to spin if it exists in the system PATH
pub fn get_spin_path() -> Result<PathBuf> {
    // Check if spin is available in PATH
    if let Ok(system_spin_path) = which::which("spin") {
        return Ok(system_spin_path);
    }

    anyhow::bail!("Spin not found")
}

pub async fn check_and_install_spin() -> Result<PathBuf> {
    // Check if spin is available in PATH
    if let Ok(system_spin_path) = which::which("spin") {
        debug!("Found system Spin in PATH at: {:?}", system_spin_path);
        ensure_akamai_plugin(&system_spin_path)?;
        return Ok(system_spin_path);
    }

    // Spin not found - emit warning
    eprintln!("⚠️  FTL requires Spin to run WebAssembly tools.");
    eprintln!("Please install Spin from: https://github.com/fermyon/spin");
    eprintln!("Or use your package manager (e.g., brew install fermyon/tap/spin)");

    anyhow::bail!("Spin not found. Please install it from https://github.com/fermyon/spin")
}

fn ensure_akamai_plugin(spin_path: &PathBuf) -> Result<()> {
    // Check if Akamai plugin is installed
    let output = Command::new(spin_path)
        .args(["plugin", "list"])
        .output()
        .context("Failed to list Spin plugins")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("aka") {
            debug!("Akamai plugin is already installed");
            return Ok(());
        }
    }

    // Install the plugin
    info!("Installing Akamai plugin for Spin");
    let install_output = Command::new(spin_path)
        .args(["plugin", "install", "aka"])
        .output()
        .context("Failed to install Akamai plugin")?;

    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        eprintln!("⚠️  Warning: Failed to install Akamai plugin: {stderr}");
        eprintln!("   You can install it manually with: spin plugin install aka");
    } else {
        debug!("Akamai plugin installed successfully");
    }

    Ok(())
}
