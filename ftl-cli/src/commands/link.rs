use anyhow::Result;
use console::style;
use std::process::Command;

use crate::common::tool_paths::validate_tool_exists;

pub async fn execute(name: String, path: Option<String>) -> Result<()> {
    let tool_path = path.unwrap_or_else(|| ".".to_string());
    
    // Validate tool directory exists
    validate_tool_exists(&tool_path)?;
    
    // Check if spin is installed
    if which::which("spin").is_err() {
        anyhow::bail!(
            "Spin CLI not found. Please install it from: https://developer.fermyon.com/spin/install"
        );
    }

    println!("{} Linking tool to deployment: {}", style("→").cyan(), style(&name).bold());

    // Check if .ftl/spin.toml exists
    let spin_toml = std::path::Path::new(&tool_path).join(".ftl/spin.toml");
    if !spin_toml.exists() {
        anyhow::bail!(
            ".ftl/spin.toml not found. Please build the tool first with: ftl build"
        );
    }

    // Run spin aka app link with --app-name flag
    let output = Command::new("spin")
        .args(["aka", "app", "link", "--app-name", &name, "-f", ".ftl/spin.toml"])
        .current_dir(&tool_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not logged in") || stderr.contains("authentication") {
            anyhow::bail!(
                "Not authenticated with FTL Edge. Please run: ftl login"
            );
        }
        if stderr.contains("not found") || stderr.contains("does not exist") {
            anyhow::bail!(
                "Tool/toolkit '{}' not found in FTL Edge. Use 'ftl list' to see available deployments.", 
                name
            );
        }
        if stderr.contains("already linked") {
            anyhow::bail!(
                "This tool is already linked to a deployment. Use 'ftl unlink' first to unlink it."
            );
        }
        anyhow::bail!("Failed to link tool:\n{}", stderr);
    }

    println!("{} Tool successfully linked to '{}'", style("✓").green(), name);
    println!();
    println!("You can now:");
    println!("  ftl deploy         # Deploy updates to the linked tool");
    println!("  ftl logs {}      # View logs", name);
    println!("  ftl status {}    # Check status", name);

    Ok(())
}