use std::process::Command;

use anyhow::{Context, Result};
use tracing::info;

use crate::common::{manifest_utils::validate_and_load_manifest, tool_paths::validate_tool_exists};

pub async fn execute(name: Option<String>) -> Result<()> {
    let tool_path = name.unwrap_or_else(|| ".".to_string());
    test_tool(&tool_path).await
}

async fn test_tool(tool_path: &str) -> Result<()> {
    // Validate tool exists and load manifest
    validate_tool_exists(tool_path)?;
    let manifest = validate_and_load_manifest(tool_path)?;

    info!("Testing tool: {}", manifest.tool.name);

    // Run tests using cargo test
    // Note: We run standard Rust unit tests, not in the WASM runtime
    // This tests the logic without the complexity of WASM environment
    // For full WASM component testing, consider using spin-test (experimental)
    let mut cmd = Command::new("cargo");
    cmd.current_dir(tool_path).arg("test").arg("--lib"); // Only run library tests, not integration tests

    // Add any features from the manifest
    if !manifest.build.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(manifest.build.features.join(","));
    }

    info!("Running tests for '{}'...", tool_path);
    println!("🧪 Running tests for '{tool_path}'...");

    let output = cmd.output().context("Failed to execute cargo test")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{stdout}");
        println!("✅ All tests passed for '{tool_path}'");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        println!("❌ Tests failed for '{tool_path}'");
        if !stdout.is_empty() {
            println!("\nOutput:\n{stdout}");
        }
        if !stderr.is_empty() {
            println!("\nErrors:\n{stderr}");
        }

        // Check if it's because no tests exist
        if stdout.contains("0 tests") || stderr.contains("could not find") {
            println!(
                "\n💡 No tests found. Your tool template includes example tests in src/lib.rs"
            );
            println!("   The tests verify basic tool functionality like name and description.");
            println!("\nTo add more tests:");
            println!("   1. Add #[test] functions to src/lib.rs");
            println!("   2. Test your tool's logic without needing WASM runtime");
            println!("   3. Use standard Rust testing patterns");
            println!("\nNote: These are unit tests that run in native Rust, not in WASM.");
            println!("For WASM runtime testing, consider spin-test (experimental).");
        }

        anyhow::bail!("Tests failed for '{}'", tool_path);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_missing_tool() {
        let result = test_tool("nonexistent_tool").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_missing_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let tool_dir = temp_dir.path().join("test_tool");
        fs::create_dir(&tool_dir).unwrap();

        let result = test_tool(tool_dir.to_str().unwrap()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No ftl.toml found"));
    }
}
