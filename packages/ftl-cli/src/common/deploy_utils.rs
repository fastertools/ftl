use anyhow::Result;

use crate::common::{config::FtlConfig, manifest_utils::load_manifest_and_name};

/// Infer the deployed app name from the current directory
/// This combines the username prefix with the tool name
pub fn infer_app_name(tool_path: &str) -> Result<String> {
    // Load manifest to get tool name
    let (_manifest, tool_name) = load_manifest_and_name(tool_path)?;

    // Load config to get username prefix
    let config = FtlConfig::load().unwrap_or_default();

    // Generate the full app name
    Ok(format!("{}{}", config.get_app_prefix(), tool_name))
}
