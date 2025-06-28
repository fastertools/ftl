use std::path::Path;

use anyhow::{Context, Result};

use super::tool_paths::{get_manifest_path, validate_tool_exists};
use crate::manifest::ToolManifest;

/// Load a tool manifest from a tool directory
pub fn load_tool_manifest<P: AsRef<Path>>(tool_path: P) -> Result<ToolManifest> {
    let manifest_path = get_manifest_path(&tool_path);
    ToolManifest::load(&manifest_path)
        .with_context(|| format!("Failed to load manifest from '{}'", manifest_path.display()))
}

/// Validate that a tool exists and load its manifest
pub fn validate_and_load_manifest<P: AsRef<Path>>(tool_path: P) -> Result<ToolManifest> {
    validate_tool_exists(&tool_path)?;
    load_tool_manifest(tool_path)
}

/// Get the tool name from the manifest, handling "." as current directory
#[allow(dead_code)]
pub fn get_tool_name<P: AsRef<Path>>(tool_path: P) -> Result<String> {
    let manifest = load_tool_manifest(&tool_path)?;
    Ok(manifest.tool.name)
}

/// Load and validate a tool manifest, returning both the manifest and resolved
/// tool name
pub fn load_manifest_and_name<P: AsRef<Path>>(tool_path: P) -> Result<(ToolManifest, String)> {
    let manifest = validate_and_load_manifest(&tool_path)?;
    let tool_name = manifest.tool.name.clone();
    Ok((manifest, tool_name))
}
