use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Get the path to a tool's manifest file
pub fn get_manifest_path<P: AsRef<Path>>(tool_path: P) -> PathBuf {
    tool_path.as_ref().join("ftl.toml")
}

/// Get the path to a tool's .ftl directory
pub fn get_ftl_dir<P: AsRef<Path>>(tool_path: P) -> PathBuf {
    tool_path.as_ref().join(".ftl")
}

/// Get the path to a tool's WASM binary
pub fn get_wasm_path<P: AsRef<Path>>(tool_path: P, tool_name: &str, profile: &str) -> PathBuf {
    let wasm_filename = format!("{}.wasm", tool_name.replace('-', "_"));
    let profile_dir = get_profile_dir(profile);

    tool_path
        .as_ref()
        .join("target")
        .join("wasm32-wasip1")
        .join(profile_dir)
        .join(wasm_filename)
}

/// Get the path to a tool's WASM binary based on language
pub fn get_wasm_path_for_language<P: AsRef<Path>>(
    tool_path: P,
    tool_name: &str,
    profile: &str,
    language: crate::language::Language,
) -> PathBuf {
    use crate::language::Language;

    match language {
        Language::Rust => {
            let wasm_filename = format!("{}.wasm", tool_name.replace('-', "_"));
            let profile_dir = get_profile_dir(profile);
            tool_path
                .as_ref()
                .join("target")
                .join("wasm32-wasip1")
                .join(profile_dir)
                .join(wasm_filename)
        }
        Language::JavaScript => {
            // JavaScript tools output to dist directory
            tool_path
                .as_ref()
                .join("dist")
                .join(format!("{tool_name}.wasm"))
        }
    }
}

/// Get the profile directory name based on the profile
pub fn get_profile_dir(profile: &str) -> &str {
    match profile {
        "dev" => "debug",
        profile => profile,
    }
}

/// Validate that a tool directory exists
pub fn validate_tool_exists<P: AsRef<Path>>(tool_path: P) -> Result<()> {
    let path = tool_path.as_ref();
    if !path.exists() {
        anyhow::bail!("Tool directory '{}' not found", path.display());
    }

    let manifest_path = get_manifest_path(path);
    if !manifest_path.exists() {
        anyhow::bail!(
            "No ftl.toml found in '{}'. Is this an FTL tool directory?",
            path.display()
        );
    }

    Ok(())
}

/// Get the path to spin.toml in the .ftl directory
pub fn get_spin_toml_path<P: AsRef<Path>>(tool_path: P) -> PathBuf {
    get_ftl_dir(tool_path).join("spin.toml")
}

/// Ensure the .ftl directory exists
pub fn ensure_ftl_dir<P: AsRef<Path>>(tool_path: P) -> Result<PathBuf> {
    let ftl_dir = get_ftl_dir(tool_path);
    std::fs::create_dir_all(&ftl_dir)
        .with_context(|| format!("Failed to create .ftl directory at {}", ftl_dir.display()))?;
    Ok(ftl_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_dir_mapping() {
        assert_eq!(get_profile_dir("dev"), "debug");
        assert_eq!(get_profile_dir("release"), "release");
        assert_eq!(get_profile_dir("custom"), "custom");
    }

    #[test]
    fn test_wasm_filename() {
        let path = get_wasm_path(".", "my-tool", "release");
        assert!(path.to_str().unwrap().contains("my_tool.wasm"));
    }
}
