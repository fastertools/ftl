use std::process::Command;

use anyhow::Result;
use console::style;

pub async fn execute() -> Result<()> {
    // Check if spin is installed
    if which::which("spin").is_err() {
        anyhow::bail!(
            "Spin CLI not found. Please install it from: https://developer.fermyon.com/spin/install"
        );
    }

    println!(
        "{} Listing deployed tools and toolkits...",
        style("→").cyan()
    );
    println!();

    // Run spin aka app list
    let output = Command::new("spin").args(["aka", "app", "list"]).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not logged in") || stderr.contains("authentication") {
            anyhow::bail!("Not authenticated with FTL Edge. Please run: ftl login");
        }
        anyhow::bail!("Failed to list tools and toolkits:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check if there are no apps
    if stdout.trim().is_empty() || stdout.contains("No apps") {
        println!("No tools or toolkits deployed yet.");
        println!();
        println!("Deploy your first tool with:");
        println!("  ftl deploy <tool-name>");
    } else {
        // Print the output as-is (spin aka app list has nice formatting)
        print!("{stdout}");
    }

    Ok(())
}
