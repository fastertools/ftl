use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use console::style;

use crate::common::spin_installer::check_and_install_spin;

pub async fn execute(path: Option<PathBuf>, port: u16, build: bool) -> Result<()> {
    let component_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Validate component directory exists
    if !component_path.join("spin.toml").exists() {
        anyhow::bail!("No spin.toml found. Not in a component or project directory?");
    }

    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // Build command args
    let mut args = vec!["up"];

    if build {
        args.push("--build");
    }

    let listen_addr = format!("127.0.0.1:{}", port);
    args.extend(["--listen", &listen_addr]);

    println!();
    println!("{} Server starting...", style("▶").green());
    println!();
    println!("{} Press Ctrl+C to stop", style("⏹").green());
    println!();

    // Run spin up with inherited stdio so user can see logs
    let mut child = Command::new(&spin_path)
        .args(&args)
        .current_dir(&component_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start spin up")?;

    // Wait for Ctrl+C
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("{} Stopping server...", style("■").red());
        }
        status = tokio::task::spawn_blocking(move || child.wait()) => {
            if let Ok(Ok(status)) = status {
                if !status.success() {
                    anyhow::bail!("Spin exited with status: {}", status);
                }
            }
        }
    }

    Ok(())
}
