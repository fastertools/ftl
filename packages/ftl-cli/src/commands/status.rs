use std::process::Command;

use anyhow::Result;
use console::style;

use crate::common::deploy_utils::infer_app_name;

pub async fn execute(name: Option<String>) -> Result<()> {
    // Check if spin is installed
    if which::which("spin").is_err() {
        anyhow::bail!(
            "Spin CLI not found. Please install it from: https://developer.fermyon.com/spin/install"
        );
    }

    // Get the app name - either provided or inferred from current directory
    let app_name = match name {
        Some(n) => n,
        None => infer_app_name(".")?,
    };

    println!(
        "{} Getting status for: {}",
        style("→").cyan(),
        style(&app_name).bold()
    );
    println!();

    // Run spin aka app status with --app-name flag
    let output = Command::new("spin")
        .args(["aka", "app", "status", "--app-name", &app_name])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not logged in") || stderr.contains("authentication") {
            anyhow::bail!("Not authenticated with FTL Edge. Please run: ftl login");
        }
        if stderr.contains("not found") || stderr.contains("does not exist") {
            anyhow::bail!(
                "Tool/toolkit '{}' not found. Use 'ftl list' to see deployed tools and toolkits.",
                app_name
            );
        }
        anyhow::bail!("Failed to get tool/toolkit status:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{stdout}");

    Ok(())
}
