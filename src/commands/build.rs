use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use console::style;

use crate::common::spin_installer::check_and_install_spin;

pub async fn execute(path: Option<PathBuf>, release: bool) -> Result<()> {
    let working_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Check if we're in a project directory (has spin.toml)
    if working_path.join("spin.toml").exists() {
        // Project-level build - use spin build
        println!(
            "{} Building project {}",
            style("→").cyan(),
            style(working_path.display()).bold()
        );

        let spin_path = check_and_install_spin().await?;
        
        let mut args = vec!["build"];
        if release {
            args.push("--release");
        }
        
        let mut child = Command::new(&spin_path)
            .args(&args)
            .current_dir(&working_path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to run spin build")?;

        let status = child.wait()?;
        if !status.success() {
            anyhow::bail!("Build failed");
        }

        println!("\n{} Project built successfully!", style("✓").green());
        return Ok(());
    }

    // Not in a Spin project directory
    anyhow::bail!("No spin.toml found. Run 'ftl build' from a project directory or use 'ftl init' to create a new project.");
}