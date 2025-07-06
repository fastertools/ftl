use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use console::style;

use crate::common::spin_installer::check_and_install_spin;

pub async fn execute(environment: Option<String>) -> Result<()> {
    println!(
        "{} Deploying project{}",
        style("→").cyan(),
        environment
            .as_ref()
            .map(|e| format!(" to {}", e))
            .unwrap_or_default()
    );

    // Check if we're in a Spin project directory
    if !PathBuf::from("spin.toml").exists() {
        anyhow::bail!("No spin.toml found. Not in a project directory?");
    }

    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // Build the project first
    println!("{} Building project...", style("→").dim());
    let build_output = Command::new(&spin_path)
        .args(["build"])
        .output()
        .context("Failed to build project")?;

    if !build_output.status.success() {
        anyhow::bail!(
            "Build failed:\n{}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Deploy using spin deploy
    println!("{} Deploying to FTL...", style("→").dim());
    let mut deploy_args = vec!["deploy"];

    if let Some(env) = &environment {
        deploy_args.extend(["--environment-name", env]);
    }

    let deploy_output = Command::new(&spin_path)
        .args(&deploy_args)
        .output()
        .context("Failed to deploy project")?;

    if !deploy_output.status.success() {
        let stderr = String::from_utf8_lossy(&deploy_output.stderr);

        if stderr.contains("not logged in") {
            anyhow::bail!("Not logged in to FTL. Run 'spin login' first.");
        }

        anyhow::bail!("Deploy failed:\n{}", stderr);
    }

    // Parse deployment URL
    let output_str = String::from_utf8_lossy(&deploy_output.stdout);
    if let Some(url_line) = output_str.lines().find(|l| l.contains("https://")) {
        println!();
        println!("{} Project deployed successfully!", style("✓").green());
        println!("  URL: {}", style(url_line.trim()).cyan());
    } else {
        println!("{} Project deployed successfully!", style("✓").green());
    }

    Ok(())
}
