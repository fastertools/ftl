use std::{path::Path, process::Command};

use anyhow::{Context, Result};
use tracing::info;

use crate::manifest::ToolManifest;

pub async fn execute(name: String) -> Result<()> {
    let tool_dir = Path::new(&name);
    if !tool_dir.exists() {
        anyhow::bail!("Tool directory '{}' not found", name);
    }

    println!("🔍 Validating tool: {name}");

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check for required files
    let manifest_path = tool_dir.join("ftl.toml");
    if !manifest_path.exists() {
        errors.push("Missing ftl.toml manifest file".to_string());
    } else {
        // Validate manifest
        match ToolManifest::load(&manifest_path) {
            Ok(manifest) => {
                info!("Manifest loaded successfully");

                // Validate manifest fields
                if manifest.tool.name.is_empty() {
                    errors.push("Tool name cannot be empty".to_string());
                }

                if manifest.tool.description.is_empty() {
                    warnings.push("Tool description is empty".to_string());
                }

                // Validate version format
                if !is_valid_version(&manifest.tool.version) {
                    errors.push(format!("Invalid version format: {}", manifest.tool.version));
                }
            }
            Err(e) => {
                errors.push(format!("Invalid ftl.toml: {e}"));
            }
        }
    }

    // Check Cargo.toml
    let cargo_path = tool_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        errors.push("Missing Cargo.toml file".to_string());
    } else {
        // Validate Cargo.toml
        match std::fs::read_to_string(&cargo_path) {
            Ok(content) => {
                let cargo_toml: toml::Value =
                    content.parse().context("Failed to parse Cargo.toml")?;

                // Check for required dependencies
                if let Some(deps) = cargo_toml.get("dependencies") {
                    if deps.get("ftl-core").is_none() {
                        errors.push("Missing ftl-core dependency in Cargo.toml".to_string());
                    }
                } else {
                    errors.push("No dependencies section in Cargo.toml".to_string());
                }

                // Check crate type
                if let Some(lib) = cargo_toml.get("lib") {
                    if let Some(crate_types) = lib.get("crate-type") {
                        let types = crate_types
                            .as_array()
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.as_str());

                        if types != Some("cdylib") {
                            errors.push(
                                "Library crate-type must be [\"cdylib\"] for WebAssembly"
                                    .to_string(),
                            );
                        }
                    } else {
                        errors.push("Missing crate-type in [lib] section".to_string());
                    }
                } else {
                    errors.push("Missing [lib] section in Cargo.toml".to_string());
                }
            }
            Err(e) => {
                errors.push(format!("Failed to read Cargo.toml: {e}"));
            }
        }
    }

    // Check source files
    let lib_path = tool_dir.join("src/lib.rs");
    if !lib_path.exists() {
        errors.push("Missing src/lib.rs file".to_string());
    } else {
        // Basic validation of lib.rs content
        match std::fs::read_to_string(&lib_path) {
            Ok(content) => {
                if !content.contains("impl Tool for") {
                    warnings.push("No Tool trait implementation found in src/lib.rs".to_string());
                }

                if !content.contains("ftl_mcp_server!") {
                    errors
                        .push("Missing ftl_mcp_server! macro invocation in src/lib.rs".to_string());
                }
            }
            Err(e) => {
                errors.push(format!("Failed to read src/lib.rs: {e}"));
            }
        }
    }

    // Try to run cargo check
    println!("\n📦 Running cargo check...");
    let check_result = Command::new("cargo")
        .current_dir(tool_dir)
        .arg("check")
        .arg("--target")
        .arg("wasm32-wasip1")
        .output();

    match check_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                errors.push(format!("Cargo check failed:\n{stderr}"));
            } else {
                println!("✅ Cargo check passed");
            }
        }
        Err(e) => {
            warnings.push(format!("Could not run cargo check: {e}"));
        }
    }

    // Print results
    println!("\n📋 Validation Results:");

    if errors.is_empty() && warnings.is_empty() {
        println!("✅ All checks passed!");
        Ok(())
    } else {
        if !warnings.is_empty() {
            println!("\n⚠️  Warnings ({})", warnings.len());
            for warning in warnings {
                println!("   - {warning}");
            }
        }

        if !errors.is_empty() {
            println!("\n❌ Errors ({})", errors.len());
            let error_count = errors.len();
            for error in errors {
                println!("   - {error}");
            }
            anyhow::bail!("Validation failed with {} error(s)", error_count);
        }

        Ok(())
    }
}

fn is_valid_version(version: &str) -> bool {
    // Basic semver validation
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_validation() {
        assert!(is_valid_version("1.0.0"));
        assert!(is_valid_version("0.0.1"));
        assert!(is_valid_version("10.20.30"));

        assert!(!is_valid_version("1.0"));
        assert!(!is_valid_version("1.0.0.0"));
        assert!(!is_valid_version("1.a.0"));
        assert!(!is_valid_version(""));
    }
}
