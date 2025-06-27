use anyhow::Result;
use console::style;
use std::process::Command;
use std::io::{self, Write};

use crate::common::deploy_utils::infer_app_name;

pub async fn execute(name: Option<String>, yes: bool) -> Result<()> {
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

    // Confirm deletion unless --yes flag is provided
    if !yes {
        print!("{} Are you sure you want to delete '{}'? [y/N] ", 
            style("?").yellow(), 
            style(&app_name).bold()
        );
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        
        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    println!("{} Deleting tool/toolkit: {}", style("→").cyan(), style(&app_name).bold());

    // Run spin aka app delete with --app-name flag and --no-confirm
    let output = Command::new("spin")
        .args(["aka", "app", "delete", "--app-name", &app_name, "--no-confirm"])
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
                "Tool/toolkit '{}' not found. Use 'ftl list' to see deployed tools and toolkits.", 
                app_name
            );
        }
        anyhow::bail!("Failed to delete tool/toolkit:\n{}", stderr);
    }

    println!("{} Tool/toolkit '{}' deleted successfully", style("✓").green(), app_name);

    Ok(())
}