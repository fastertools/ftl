use std::{path::Path, process::Command};

use anyhow::{Context, Result};

use crate::{
    common::spin_installer::check_and_install_spin,
    language::{LanguageSupport, PackageManager},
    manifest::Manifest,
};

pub struct JavaScriptSupport;

impl JavaScriptSupport {
    pub fn new() -> Self {
        Self
    }

}

impl LanguageSupport for JavaScriptSupport {
    fn new_project(
        &self,
        _name: &str,
        _description: &str,
        _template: &str,
        _path: &Path,
    ) -> Result<()> {
        // This is now handled by spin templates in the new command
        anyhow::bail!("Direct project creation is deprecated. Use 'ftl new' command instead.")
    }

    fn build(&self, _manifest: &Manifest, path: &Path) -> Result<()> {
        // Get spin path using blocking runtime
        let spin_path = tokio::runtime::Handle::try_current()
            .ok()
            .and_then(|handle| {
                tokio::task::block_in_place(|| handle.block_on(check_and_install_spin()).ok())
            })
            .unwrap_or_else(|| {
                // If no runtime exists, create one
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                rt.block_on(check_and_install_spin())
                    .expect("Failed to install Spin")
            });

        // Run spin build with spin.toml from .ftl directory
        let spin_toml_path = path.join(".ftl/spin.toml");

        // Ensure the spin.toml exists
        if !spin_toml_path.exists() {
            anyhow::bail!("spin.toml not found at: {}", spin_toml_path.display());
        }

        // Convert to absolute path to avoid issues with relative paths
        let absolute_spin_toml = spin_toml_path
            .canonicalize()
            .context("Failed to resolve spin.toml path")?;

        let output = Command::new(&spin_path)
            .args(["build", "-f", absolute_spin_toml.to_str().unwrap()])
            .current_dir(path)
            .output()
            .context("Failed to run spin build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Build failed:\n{stderr}");
        }

        Ok(())
    }

    fn test(&self, _manifest: &Manifest, path: &Path) -> Result<()> {
        let pm = PackageManager::detect(path);
        let test_cmd = pm.run_command("test");
        let mut cmd_parts = test_cmd.split_whitespace();
        let output = Command::new(cmd_parts.next().unwrap())
            .args(cmd_parts)
            .current_dir(path)
            .output()
            .context("Failed to run tests")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            if !stdout.is_empty() {
                println!("\nOutput:\n{stdout}");
            }
            if !stderr.is_empty() {
                println!("\nErrors:\n{stderr}");
            }

            anyhow::bail!("Tests failed");
        }

        println!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }

    fn validate_environment(&self) -> Result<()> {
        // Check if Node.js is installed
        let output = Command::new("node")
            .arg("--version")
            .output()
            .context("Node.js is not installed. Please install Node.js from https://nodejs.org")?;

        let version = String::from_utf8_lossy(&output.stdout);
        let version_parts: Vec<&str> = version.trim().trim_start_matches('v').split('.').collect();

        if let Some(major) = version_parts.first().and_then(|v| v.parse::<u32>().ok()) {
            if major < 18 {
                let version = version.trim();
                anyhow::bail!(
                    "Node.js version {version} is too old. Please install Node.js 18 or later."
                );
            }
        }

        Ok(())
    }
}
