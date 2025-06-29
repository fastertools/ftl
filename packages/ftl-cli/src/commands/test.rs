use anyhow::Result;
use tracing::info;

use crate::{
    common::{manifest_utils::validate_and_load_manifest, tool_paths::validate_tool_exists},
    language::get_language_support,
};

pub async fn execute(name: Option<String>) -> Result<()> {
    let tool_path = name.unwrap_or_else(|| ".".to_string());
    test_tool(&tool_path).await
}

async fn test_tool(tool_path: &str) -> Result<()> {
    // Validate tool exists and load manifest
    validate_tool_exists(tool_path)?;
    let manifest = validate_and_load_manifest(tool_path)?;

    info!("Testing tool: {}", manifest.tool.name);
    println!("🧪 Running tests for '{tool_path}'...");

    // Get language support and run tests
    let language_support = get_language_support(manifest.tool.language);

    // Use the language-specific test implementation
    match language_support.test(&manifest, std::path::Path::new(tool_path)) {
        Ok(_) => {
            println!("✅ All tests passed for '{tool_path}'");
            Ok(())
        }
        Err(e) => {
            println!("❌ Tests failed for '{tool_path}'");

            // Provide helpful message for missing tests
            let error_msg = e.to_string();
            if error_msg.contains("0 tests") || error_msg.contains("could not find") {
                println!(
                    "\n💡 No tests found. Your tool template includes example tests in {}",
                    match manifest.tool.language {
                        crate::language::Language::Rust => "src/lib.rs",
                        crate::language::Language::JavaScript => "src/index.test.js",
                    }
                );
                println!("   The tests verify basic tool functionality like name and description.");
                println!("\nTo add more tests:");
                match manifest.tool.language {
                    crate::language::Language::Rust => {
                        println!("   1. Add #[test] functions to src/lib.rs");
                        println!("   2. Test your tool's logic without needing WASM runtime");
                        println!("   3. Use standard Rust testing patterns");
                    }
                    crate::language::Language::JavaScript => {
                        println!("   1. Add test files matching *.test.js or *.spec.js");
                        println!("   2. Use your preferred JavaScript testing framework");
                        println!("   3. Tests run in Node.js, not in the WASM runtime");
                    }
                }
                println!("\nNote: These are unit tests that run natively, not in WASM.");
                println!("For WASM runtime testing, consider spin-test (experimental).");
            }

            Err(e)
        }
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No ftl.toml found")
        );
    }
}
