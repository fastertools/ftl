use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use console::style;

use crate::manifest::ComponentManifest;

pub async fn execute(
    path: Option<PathBuf>,
    registry: Option<String>,
    tag: Option<String>,
) -> Result<()> {
    let component_path = path.unwrap_or_else(|| PathBuf::from("."));

    println!("{} Publishing component", style("→").cyan());

    // Validate component directory exists
    if !component_path.join("ftl.toml").exists() {
        anyhow::bail!("No ftl.toml found. Not in a component directory?");
    }

    // Load component manifest
    let manifest = ComponentManifest::load(&component_path)?;
    let version = tag.as_ref().unwrap_or(&manifest.component.version);

    // Use make registry-push if Makefile exists
    if component_path.join("Makefile").exists() {
        println!("{} Using Makefile to publish...", style("→").dim());

        let output = Command::new("make")
            .arg("registry-push")
            .current_dir(&component_path)
            .output()
            .context("Failed to run make registry-push")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Check if wkg is missing
            if stderr.contains("wkg: command not found") || stderr.contains("wkg: not found") {
                anyhow::bail!(
                    "The 'wkg' tool is required for publishing. Install it from: https://github.com/bytecodealliance/wasm-pkg-tools"
                );
            }

            anyhow::bail!("Publishing failed:\n{}\n{}", stdout, stderr);
        }

        // Parse the output to get the published URL
        let output_str = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = output_str.lines().find(|l| l.contains("Pushing ghcr.io/")) {
            println!();
            println!("{} Component published!", style("✓").green());
            println!("  {}", style(line.trim_start_matches("Pushing ")).cyan());
        } else {
            println!("{} Component published successfully!", style("✓").green());
        }
    } else {
        // Manual publish flow
        let registry_url = registry.as_deref().unwrap_or("ghcr.io");

        // Get username from git config
        let username_output = Command::new("git")
            .args(["config", "user.name"])
            .output()
            .context("Failed to get git username")?;

        let username = String::from_utf8_lossy(&username_output.stdout)
            .trim()
            .to_lowercase()
            .replace(' ', "-");

        if username.is_empty() {
            anyhow::bail!("Could not determine username from git config");
        }

        // Build component first
        println!("{} Building component...", style("→").dim());
        crate::commands::build::execute(Some(component_path.clone()), true).await?;

        // Find the built WASM file
        let wasm_path = find_wasm_file(&component_path, &manifest)?;

        // Construct package URL
        let package_name = format!(
            "{}/{}/{}:{}",
            registry_url, username, manifest.component.name, version
        );

        println!("{} Publishing to {}...", style("→").dim(), package_name);

        // Use wkg to push
        let output = Command::new("wkg")
            .args(["oci", "push", &package_name, wasm_path.to_str().unwrap()])
            .output()
            .context("Failed to run wkg")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            if stderr.contains("not found") && stderr.contains("wkg") {
                anyhow::bail!(
                    "The 'wkg' tool is required for publishing. Install it from: https://github.com/bytecodealliance/wasm-pkg-tools"
                );
            }

            anyhow::bail!("Publishing failed:\n{}", stderr);
        }

        println!();
        println!("{} Component published!", style("✓").green());
        println!("  {}", style(&package_name).cyan());
    }

    println!();
    println!("{} Next steps:", style("→").dim());
    println!(
        "  - Use 'ftl project add {}' to add this component to a project",
        manifest.component.name
    );
    println!(
        "  - Share the registry URL: {}",
        tag.as_ref()
            .map(|t| format!("{}:{}", manifest.component.name, t))
            .unwrap_or_else(|| manifest.component.name.clone())
    );

    Ok(())
}

fn find_wasm_file(component_path: &Path, manifest: &ComponentManifest) -> Result<PathBuf> {
    // Check common locations
    let candidates = vec![
        // Rust
        component_path
            .join("handler/target/wasm32-wasip1/release")
            .join(format!(
                "{}.wasm",
                manifest.component.name.replace('-', "_")
            )),
        // JS/TS
        component_path.join("handler/dist/handler.wasm"),
        component_path
            .join("handler/dist")
            .join(format!("{}.wasm", manifest.component.name)),
    ];

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!("Could not find built WASM file. Did you run 'ftl build'?")
}
