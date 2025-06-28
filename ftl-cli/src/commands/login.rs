use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use anyhow::Result;
use console::style;

use crate::common::config::FtlConfig;

pub async fn execute() -> Result<()> {
    // Check if spin is installed
    if which::which("spin").is_err() {
        anyhow::bail!(
            "Spin CLI not found. Please install it from: https://developer.fermyon.com/spin/install"
        );
    }

    println!("{} Logging in to FTL Edge...", style("→").cyan());
    println!();

    // Run spin aka auth login and capture output to parse username
    let mut child = Command::new("spin")
        .args(["aka", "auth", "login"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Read output line by line, looking for "Welcome, username."
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Print stdout and look for username
    let stdout_handle = std::thread::spawn(move || {
        let mut username = None;
        for line in stdout_reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
                if line.starts_with("Welcome, ") && line.ends_with(".") {
                    // Extract username from "Welcome, username."
                    let user = line
                        .trim_start_matches("Welcome, ")
                        .trim_end_matches(".")
                        .to_string();
                    username = Some(user);
                }
            }
        }
        username
    });

    // Print stderr
    for line in stderr_reader.lines() {
        if let Ok(line) = line {
            eprintln!("{}", line);
        }
    }

    let status = child.wait()?;
    let captured_username = stdout_handle.join().unwrap();

    if status.success() {
        println!();
        println!("{} Successfully logged in to FTL Edge!", style("✓").green());

        // Load existing config
        let mut config = FtlConfig::load().unwrap_or_default();

        // Save username if we captured it
        if let Some(username) = captured_username {
            config.username = Some(username.clone());
            config.save()?;
        }

        println!();
        println!("You can now:");
        println!("  • Deploy tools with: ftl deploy");
        println!("  • List your tools with: ftl list");
    } else {
        anyhow::bail!("Login failed");
    }

    Ok(())
}
