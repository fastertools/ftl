use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use console::style;

pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let working_path = path.unwrap_or_else(|| PathBuf::from("."));

    println!("{} Running tests", style("→").cyan());

    // Check if we're in a project directory with spin.toml
    if working_path.join("spin.toml").exists() {
        // In a project directory - run tests for all tools
        println!("{} Testing all tools in project", style("→").dim());

        // Read directory entries to find tool directories
        let entries = std::fs::read_dir(&working_path)?;
        let mut any_tests_run = false;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check if this is a tool directory (has Cargo.toml or package.json)
                if path.join("Cargo.toml").exists() || path.join("package.json").exists() {
                    println!(
                        "\n{} Testing {}",
                        style("→").cyan(),
                        path.file_name().unwrap().to_string_lossy()
                    );
                    run_tool_tests(&path)?;
                    any_tests_run = true;
                }
            }
        }

        if !any_tests_run {
            println!("{} No tools found to test", style("ℹ").yellow());
        }
    } else {
        // Try to run tests in current directory as a single tool
        run_tool_tests(&working_path)?;
    }

    println!();
    println!("{} All tests passed!", style("✓").green());

    Ok(())
}

fn run_tool_tests(tool_path: &PathBuf) -> Result<()> {
    // Check if Makefile exists and has test target
    if tool_path.join("Makefile").exists() {
        let output = Command::new("make")
            .arg("test")
            .current_dir(tool_path)
            .output()
            .context("Failed to run make test")?;

        if !output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Tests failed");
        }

        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else if tool_path.join("Cargo.toml").exists() {
        // Rust tool
        let output = Command::new("cargo")
            .arg("test")
            .current_dir(tool_path)
            .output()
            .context("Failed to run cargo test")?;

        println!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Tests failed");
        }
    } else if tool_path.join("package.json").exists() {
        // JavaScript/TypeScript tool
        let output = Command::new("npm")
            .arg("test")
            .current_dir(tool_path)
            .output()
            .context("Failed to run npm test")?;

        println!("{}", String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Tests failed");
        }
    } else {
        println!(
            "{} No test configuration found for this tool",
            style("⚠").yellow()
        );
    }

    Ok(())
}
