//! Refactored test command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::deps::{MessageStyle, UserInterface};

/// Directory operations trait
pub trait DirectoryReader: Send + Sync {
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn is_dir(&self, path: &Path) -> Result<bool>;
}

/// File existence checker trait
pub trait FileChecker: Send + Sync {
    fn exists(&self, path: &Path) -> Result<bool>;
}

/// Command executor trait for test command
pub trait TestCommandExecutor: Send + Sync {
    fn execute(&self, command: &str, args: &[&str], working_dir: Option<&str>) -> Result<Output>;
}

/// Dependencies for the test command
pub struct TestDependencies {
    pub ui: Arc<dyn UserInterface>,
    pub directory_reader: Arc<dyn DirectoryReader>,
    pub file_checker: Arc<dyn FileChecker>,
    pub command_executor: Arc<dyn TestCommandExecutor>,
}

/// Execute the test command with injected dependencies
pub fn execute_with_deps(path: Option<PathBuf>, deps: &Arc<TestDependencies>) -> Result<()> {
    let working_path = path.unwrap_or_else(|| PathBuf::from("."));

    deps.ui.print_styled("→ Running tests", MessageStyle::Cyan);

    // Check if we're in a project directory with spin.toml
    if deps.file_checker.exists(&working_path.join("spin.toml"))? {
        // In a project directory - run tests for all tools
        deps.ui.print("→ Testing all tools in project");

        // Read directory entries to find tool directories
        let entries = deps.directory_reader.read_dir(&working_path)?;
        let mut any_tests_run = false;

        for entry in entries {
            if deps.directory_reader.is_dir(&entry)? {
                // Check if this is a tool directory (has Cargo.toml or package.json)
                if deps.file_checker.exists(&entry.join("Cargo.toml"))?
                    || deps.file_checker.exists(&entry.join("package.json"))?
                {
                    let tool_name = entry
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    deps.ui.print("");
                    deps.ui
                        .print_styled(&format!("→ Testing {tool_name}"), MessageStyle::Cyan);

                    run_tool_tests(&entry, deps)?;
                    any_tests_run = true;
                }
            }
        }

        if !any_tests_run {
            deps.ui
                .print_styled("ℹ No tools found to test", MessageStyle::Yellow);
        }
    } else {
        // Try to run tests in current directory as a single tool
        run_tool_tests(&working_path, deps)?;
    }

    deps.ui.print("");
    deps.ui
        .print_styled("✓ All tests passed!", MessageStyle::Success);

    Ok(())
}

fn run_tool_tests(tool_path: &Path, deps: &Arc<TestDependencies>) -> Result<()> {
    // Check if Makefile exists and has test target
    if deps.file_checker.exists(&tool_path.join("Makefile"))? {
        let output = deps
            .command_executor
            .execute("make", &["test"], tool_path.to_str())
            .context("Failed to run make test")?;

        if !output.status.success() {
            deps.ui.print(&String::from_utf8_lossy(&output.stdout));
            deps.ui.print(&String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Tests failed");
        }

        deps.ui.print(&String::from_utf8_lossy(&output.stdout));
    } else if deps.file_checker.exists(&tool_path.join("Cargo.toml"))? {
        // Rust tool
        let output = deps
            .command_executor
            .execute("cargo", &["test"], tool_path.to_str())
            .context("Failed to run cargo test")?;

        deps.ui.print(&String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            deps.ui.print(&String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Tests failed");
        }
    } else if deps.file_checker.exists(&tool_path.join("package.json"))? {
        // JavaScript/TypeScript tool
        let output = deps
            .command_executor
            .execute("npm", &["test"], tool_path.to_str())
            .context("Failed to run npm test")?;

        deps.ui.print(&String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            deps.ui.print(&String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Tests failed");
        }
    } else {
        deps.ui.print_styled(
            "⚠ No test configuration found for this tool",
            MessageStyle::Yellow,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_with_deps_signature() {
        // Just verify the function signature compiles
        let _ = execute_with_deps;
    }
}
