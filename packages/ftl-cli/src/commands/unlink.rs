use std::process::Command;

use anyhow::Result;
use console::style;

use crate::common::tool_paths::validate_tool_exists;

pub async fn execute(path: Option<String>) -> Result<()> {
    let tool_path = path.unwrap_or_else(|| ".".to_string());

    // Validate tool directory exists
    validate_tool_exists(&tool_path)?;

    // Check if spin is installed
    if which::which("spin").is_err() {
        anyhow::bail!(
            "Spin CLI not found. Please install it from: https://developer.fermyon.com/spin/install"
        );
    }

    // Check if .ftl/spin.toml exists
    let spin_toml = std::path::Path::new(&tool_path).join(".ftl/spin.toml");
    if !spin_toml.exists() {
        anyhow::bail!(".ftl/spin.toml not found. This tool doesn't appear to be built.");
    }

    println!("{} Unlinking tool from deployment...", style("→").cyan());

    // Run spin aka app unlink
    let output = Command::new("spin")
        .args(["aka", "app", "unlink", "-f", ".ftl/spin.toml"])
        .current_dir(&tool_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not logged in") || stderr.contains("authentication") {
            anyhow::bail!("Not authenticated with FTL Edge. Please run: ftl login");
        }
        if stderr.contains("not linked") || stderr.contains("No app linked") {
            anyhow::bail!("This tool is not linked to any deployment.");
        }
        anyhow::bail!("Failed to unlink tool:\n{}", stderr);
    }

    println!("{} Tool successfully unlinked", style("✓").green());
    println!();
    println!("The tool is no longer connected to any FTL Edge deployment.");
    println!("You can:");
    println!("  ftl link <name>    # Link to an existing deployment");
    println!("  ftl deploy         # Deploy as a new tool");

    Ok(())
}
