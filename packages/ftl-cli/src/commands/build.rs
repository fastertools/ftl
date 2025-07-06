use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use console::style;

use crate::{common::spin_installer::check_and_install_spin, manifest::ComponentManifest};

pub async fn execute(path: Option<PathBuf>, release: bool) -> Result<()> {
    let working_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Check if we're in a project directory (has spin.toml) or component directory (has ftl.toml)
    if working_path.join("spin.toml").exists() {
        // Project-level build - use spin build
        println!(
            "{} Building project {}",
            style("→").cyan(),
            style(working_path.display()).bold()
        );

        let spin_path = check_and_install_spin().await?;
        let mut child = Command::new(&spin_path)
            .args(["build"])
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

    // Component-level build
    println!(
        "{} Building component {}",
        style("→").cyan(),
        style(working_path.display()).bold()
    );

    // Validate component directory exists
    if !working_path.join("ftl.toml").exists() {
        anyhow::bail!("No ftl.toml or spin.toml found. Not in a component or project directory?");
    }

    let component_path = working_path;

    // Load component manifest
    let manifest = ComponentManifest::load(&component_path)?;

    // Run build based on detected build system
    println!("{} Running build...", style("▶").green());
    println!();

    let status = if component_path.join("Makefile").exists() {
        // Use make if available
        Command::new("make")
            .arg("build")
            .current_dir(&component_path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to run make build")?
            .wait()?
    } else if component_path.join("handler/Cargo.toml").exists() {
        // Rust component
        let profile = if release { "release" } else { "debug" };
        Command::new("cargo")
            .args(["component", "build", "--target", "wasm32-wasip1"])
            .arg(format!("--{}", profile))
            .current_dir(component_path.join("handler"))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to run cargo build")?
            .wait()?
    } else if component_path.join("handler/package.json").exists() {
        // JavaScript/TypeScript component
        // First install dependencies
        println!("{} Installing dependencies...", style("→").dim());
        let npm_install_status = Command::new("npm")
            .arg("install")
            .current_dir(component_path.join("handler"))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to run npm install")?
            .wait()?;

        if !npm_install_status.success() {
            anyhow::bail!("npm install failed");
        }

        // Then build
        println!("\n{} Building component...", style("→").dim());
        Command::new("npm")
            .args(["run", "build"])
            .current_dir(component_path.join("handler"))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to run npm build")?
            .wait()?
    } else {
        anyhow::bail!("Unable to determine build system for component");
    };

    if !status.success() {
        anyhow::bail!("Build failed");
    }

    // Verify output exists
    let wasm_path = find_wasm_output(&component_path, &manifest)?;
    let size = std::fs::metadata(&wasm_path)?.len();

    println!("\n{} Component built successfully!", style("✓").green());
    println!("  Output: {}", style(wasm_path.display()).dim());
    println!("  Size: {}", style(format_size(size)).yellow());

    Ok(())
}

fn find_wasm_output(component_path: &Path, manifest: &ComponentManifest) -> Result<PathBuf> {
    // Check common output locations based on language
    let possible_paths = vec![
        // Rust
        component_path
            .join("handler/target/wasm32-wasip1/release")
            .join(format!(
                "{}.wasm",
                manifest.component.name.replace('-', "_")
            )),
        component_path
            .join("handler/target/wasm32-wasip1/debug")
            .join(format!(
                "{}.wasm",
                manifest.component.name.replace('-', "_")
            )),
        // JavaScript/TypeScript
        component_path.join("handler/dist/handler.wasm"),
        component_path
            .join("handler/dist")
            .join(format!("{}.wasm", manifest.component.name)),
    ];

    for path in possible_paths {
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!("Could not find built WASM file. Build may have failed.")
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
