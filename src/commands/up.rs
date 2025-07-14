use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use console::style;

use crate::common::spin_installer::check_and_install_spin;

pub async fn execute(path: Option<PathBuf>, port: u16, build: bool) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Validate project directory exists
    if !project_path.join("spin.toml").exists() {
        anyhow::bail!(
            "No spin.toml found. Not in a project directory? Run 'ftl init' to create a new project."
        );
    }

    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // Build command args
    let mut args = vec!["up"];

    if build {
        args.push("--build");
    }

    let listen_addr = format!("127.0.0.1:{port}");
    args.extend(["--listen", &listen_addr]);

    if build {
        println!("{} Building and starting server...", style("→").cyan());
    } else {
        println!("{} Starting server...", style("→").cyan());
    }
    println!();
    println!(
        "{} Server will start at http://{}",
        style("🌐").blue(),
        listen_addr
    );
    println!("{} Press Ctrl+C to stop", style("⏹").yellow());
    println!();

    // Run spin up with inherited stdio so user can see logs
    let mut child = Command::new(&spin_path)
        .args(&args)
        .current_dir(&project_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start spin up")?;

    // Create a flag to track if Ctrl+C was pressed
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let ctrlc_pressed_clone = ctrlc_pressed.clone();

    // Set up Ctrl+C handler
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        ctrlc_pressed_clone.store(true, Ordering::SeqCst);
    });

    // Wait for the child process to exit
    let status = child.wait()?;

    // Check if we should print the stopping message
    if ctrlc_pressed.load(Ordering::SeqCst) {
        println!();
        println!("{} Stopping server...", style("■").red());
    } else if !status.success() {
        anyhow::bail!("Spin exited with status: {}", status);
    }

    Ok(())
}
