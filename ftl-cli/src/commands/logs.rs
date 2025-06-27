use anyhow::Result;
use console::style;
use std::process::{Command, Stdio};

use crate::common::deploy_utils::infer_app_name;

pub async fn execute(name: Option<String>, _follow: bool, tail: Option<usize>) -> Result<()> {
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

    println!("{} Fetching logs for: {}", style("→").cyan(), style(&app_name).bold());
    
    let mut args = vec!["aka", "app", "logs", "--app-name", &app_name];
    
    // Add tail option if specified
    let tail_str;
    if let Some(lines) = tail {
        tail_str = lines.to_string();
        args.push("--tail");
        args.push(&tail_str);
    }
    
    // Note: spin aka app logs doesn't support --follow yet

    // Run spin aka app logs with inherited stdio for real-time output
    let mut child = Command::new("spin")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // Wait for the command to complete
    let status = child.wait()?;

    if !status.success() {
        // Error handling is done by spin itself with inherited stderr
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}