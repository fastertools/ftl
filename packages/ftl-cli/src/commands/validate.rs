use std::{path::Path, process::Command};

use anyhow::Result;
use tracing::info;

use crate::manifest::ToolManifest;

pub async fn execute(name: String) -> Result<()> {
    let tool_dir = Path::new(&name);
    if !tool_dir.exists() {
        anyhow::bail!("Tool directory '{name}' not found");
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
                    let version = &manifest.tool.version;
                    errors.push(format!("Invalid version format: {version}"));
                }
            }
            Err(e) => {
                errors.push(format!("Invalid ftl.toml: {e}"));
            }
        }
    }

    // Check spin.toml
    let spin_path = tool_dir.join("spin.toml");
    if !spin_path.exists() {
        errors.push("Missing spin.toml file".to_string());
    }

    // Check handler directory structure
    let handler_path = tool_dir.join("handler");
    if !handler_path.exists() {
        errors.push("Missing handler directory".to_string());
    } else {
        // Check for handler Cargo.toml or package.json
        let cargo_path = handler_path.join("Cargo.toml");
        let package_path = handler_path.join("package.json");
        
        if !cargo_path.exists() && !package_path.exists() {
            errors.push("Missing Cargo.toml or package.json in handler directory".to_string());
        }
        
        // Check for WIT file
        let wit_path = handler_path.join("wit/mcp.wit");
        if !wit_path.exists() {
            errors.push("Missing wit/mcp.wit file in handler directory".to_string());
        }
    }

    // Check handler source files
    let handler_src = handler_path.join("src");
    if !handler_src.exists() {
        errors.push("Missing src directory in handler".to_string());
    } else {
        // Check for main source file (lib.rs, index.ts, or index.js)
        let lib_rs = handler_src.join("lib.rs");
        let index_ts = handler_src.join("index.ts");
        let index_js = handler_src.join("index.js");
        
        if !lib_rs.exists() && !index_ts.exists() && !index_js.exists() {
            errors.push("Missing main source file (lib.rs, index.ts, or index.js) in handler/src".to_string());
        } else if lib_rs.exists() {
            // Validate Rust handler
            match std::fs::read_to_string(&lib_rs) {
                Ok(content) => {
                    if !content.contains("impl Guest for") {
                        warnings.push("No Guest trait implementation found in handler/src/lib.rs".to_string());
                    }
                    if !content.contains("wit_bindgen::generate!") {
                        errors.push("Missing wit_bindgen::generate! macro in handler/src/lib.rs".to_string());
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to read handler/src/lib.rs: {e}"));
                }
            }
        }
    }

    // Try to run build check based on handler type
    println!("\n📦 Running build check...");
    let cargo_path = handler_path.join("Cargo.toml");
    let package_path = handler_path.join("package.json");
    
    let check_result = if cargo_path.exists() {
        Command::new("cargo")
            .current_dir(&handler_path)
            .arg("check")
            .arg("--target")
            .arg("wasm32-wasip1")
            .output()
    } else if package_path.exists() {
        Command::new("npm")
            .current_dir(&handler_path)
            .arg("install")
            .arg("--dry-run")
            .output()
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No build file found"))
    };

    match check_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                errors.push(format!("Cargo check failed:\n{stderr}"));
            } else {
                println!("✅ Build check passed");
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
            anyhow::bail!("Validation failed with {error_count} error(s)");
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
