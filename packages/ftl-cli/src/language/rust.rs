use std::{path::Path, process::Command};

use anyhow::{Context, Result};

use crate::{language::LanguageSupport, manifest::Manifest, templates};

pub struct RustSupport;

impl RustSupport {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageSupport for RustSupport {
    fn new_project(
        &self,
        name: &str,
        description: &str,
        _template: &str,
        path: &Path,
    ) -> Result<()> {
        // Use existing template generation logic
        templates::create_tool(name, description, path)?;
        Ok(())
    }

    fn build(&self, manifest: &Manifest, path: &Path) -> Result<()> {
        // Check for Rust toolchain
        self.validate_environment()?;

        // Build the project
        let output = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip1", "--release"])
            .current_dir(path)
            .output()
            .context("Failed to execute cargo build")?;

        if !output.status.success() {
            anyhow::bail!("Build failed:\n{}", String::from_utf8_lossy(&output.stderr));
        }

        // Run wasm-opt if available
        let wasm_path = path
            .join("target/wasm32-wasip1/release")
            .join(format!("{}.wasm", manifest.tool.name.replace('-', "_")));

        if wasm_path.exists() {
            self.optimize_wasm(&wasm_path)?;
        }

        Ok(())
    }

    fn test(&self, _manifest: &Manifest, path: &Path) -> Result<()> {
        let output = Command::new("cargo")
            .arg("test")
            .current_dir(path)
            .output()
            .context("Failed to execute cargo test")?;

        if !output.status.success() {
            anyhow::bail!("Tests failed:\n{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn validate_environment(&self) -> Result<()> {
        // Check if Rust is installed
        Command::new("rustc")
            .arg("--version")
            .output()
            .context("Rust is not installed. Please install Rust from https://rustup.rs")?;

        // Check if the wasm32-wasip1 target is installed
        let output = Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output()
            .context("Failed to check installed Rust targets")?;

        let installed_targets = String::from_utf8_lossy(&output.stdout);
        if !installed_targets.contains("wasm32-wasip1") {
            anyhow::bail!(
                "The wasm32-wasip1 target is not installed. Please run: rustup target add \
                 wasm32-wasip1"
            );
        }

        Ok(())
    }
}

impl RustSupport {
    fn optimize_wasm(&self, wasm_path: &Path) -> Result<()> {
        // Check if wasm-opt is available
        if Command::new("wasm-opt").arg("--version").output().is_ok() {
            println!("Optimizing WASM with wasm-opt...");

            let output = Command::new("wasm-opt")
                .args([
                    "-O3",
                    "--enable-simd",
                    "--enable-bulk-memory",
                    wasm_path.to_str().unwrap(),
                    "-o",
                    wasm_path.to_str().unwrap(),
                ])
                .output()
                .context("Failed to run wasm-opt")?;

            if !output.status.success() {
                eprintln!(
                    "Warning: wasm-opt optimization failed:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        Ok(())
    }
}
