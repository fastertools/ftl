use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use console::style;

pub async fn execute(path: Option<PathBuf>) -> Result<()> {
    let component_path = path.unwrap_or_else(|| PathBuf::from("."));

    println!("{} Running tests", style("→").cyan());

    // Check if Makefile exists and has test target
    if component_path.join("Makefile").exists() {
        let output = Command::new("make")
            .arg("test")
            .current_dir(&component_path)
            .output()
            .context("Failed to run make test")?;

        if !output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Tests failed");
        }

        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        // Try to detect test framework
        if component_path.join("handler/Cargo.toml").exists() {
            // Rust component
            let output = Command::new("cargo")
                .arg("test")
                .current_dir(component_path.join("handler"))
                .output()
                .context("Failed to run cargo test")?;

            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                println!("{}", String::from_utf8_lossy(&output.stderr));
                anyhow::bail!("Tests failed");
            }
        } else if component_path.join("handler/package.json").exists() {
            // JavaScript/TypeScript component
            let output = Command::new("npm")
                .arg("test")
                .current_dir(component_path.join("handler"))
                .output()
                .context("Failed to run npm test")?;

            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                println!("{}", String::from_utf8_lossy(&output.stderr));
                anyhow::bail!("Tests failed");
            }
        } else {
            anyhow::bail!("Could not determine how to run tests for this component");
        }
    }

    println!();
    println!("{} All tests passed!", style("✓").green());

    Ok(())
}
