use std::process::Command;

use anyhow::{Context, Result};
use console::style;
use semver::Version;

pub async fn execute(force: bool) -> Result<()> {
    println!("{} Updating FTL CLI", style("→").cyan());

    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", style(current_version).dim());

    if !force {
        // Check if we're already on the latest version
        match get_latest_version().await {
            Ok(latest_version) => {
                let current = Version::parse(current_version)?;
                let latest = Version::parse(&latest_version)?;

                if current >= latest {
                    println!(
                        "{} Already on latest version ({})",
                        style("✓").green(),
                        current_version
                    );
                    return Ok(());
                }

                println!(
                    "Latest version available: {}",
                    style(&latest_version).green()
                );
            }
            Err(_) => {
                println!(
                    "{} Could not check for latest version, proceeding with update",
                    style("⚠").yellow()
                );
            }
        }
    }

    println!("{} Installing latest version...", style("→").dim());

    // Use cargo install to update to latest version
    let mut install_cmd = Command::new("cargo");
    install_cmd.args(["install", "ftl-cli", "--force"]);

    let install_output = install_cmd
        .output()
        .context("Failed to run cargo install")?;

    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        anyhow::bail!("Failed to update FTL CLI:\n{}", stderr);
    }

    println!("{} FTL CLI updated successfully!", style("✓").green());
    println!();
    println!("Run 'ftl --version' to verify the new version");

    Ok(())
}

async fn get_latest_version() -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://crates.io/api/v1/crates/ftl-cli")
        .header(
            "User-Agent",
            format!("ftl-cli/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch crate information from crates.io");
    }

    let json: serde_json::Value = response.json().await?;

    let latest_version = json
        .get("crate")
        .and_then(|c| c.get("newest_version"))
        .and_then(|v| v.as_str())
        .context("Could not parse latest version from crates.io response")?;

    Ok(latest_version.to_string())
}
