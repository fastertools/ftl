//! Path resolution for component deployment

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Create spin.toml with resolved component paths
///
/// Takes an FTL config and mappings from `ComponentResolver`,
/// returns the transpiled spin.toml content with local paths.
pub fn create_spin_toml_with_resolved_paths<S: std::hash::BuildHasher>(
    ftl_resolve: &ftl_resolve::FtlConfig,
    resolved_mappings: &HashMap<String, PathBuf, S>,
    project_path: &Path,
) -> Result<String> {
    let mut modified_config = ftl_resolve.clone();
    let abs_project_path = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());

    // Update MCP component paths if resolved
    if let Some(gateway_path) = resolved_mappings.get("mcp-gateway") {
        modified_config.mcp.gateway = gateway_path.to_string_lossy().to_string();
    } else {
        make_path_absolute(&mut modified_config.mcp.gateway, &abs_project_path);
    }

    if let Some(authorizer_path) = resolved_mappings.get("mcp-authorizer") {
        modified_config.mcp.authorizer = authorizer_path.to_string_lossy().to_string();
    } else {
        make_path_absolute(&mut modified_config.mcp.authorizer, &abs_project_path);
    }

    // Update user component paths
    for (component_name, component_config) in &mut modified_config.component {
        if let Some(resolved_path) = resolved_mappings.get(component_name) {
            // Registry component was resolved to local path
            component_config.repo = None;
            component_config.wasm = Some(resolved_path.to_string_lossy().to_string());
        } else if let Some(wasm_path) = &component_config.wasm {
            // Local component - make path absolute
            let mut abs_wasm = wasm_path.clone();
            make_path_absolute(&mut abs_wasm, &abs_project_path);
            component_config.wasm = Some(abs_wasm);
        }

        // Update component directory path if present
        if let Some(path) = &component_config.path {
            let mut abs_path = path.clone();
            make_path_absolute(&mut abs_path, &abs_project_path);
            component_config.path = Some(abs_path);
        }
    }

    ftl_resolve::transpile_ftl_to_spin(&modified_config)
}

fn make_path_absolute(path: &mut String, base: &Path) {
    // Skip if already absolute or is a registry reference
    if path.starts_with('/')
        || path.contains("://")
        || (path.contains('/')
            && !std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("wasm")))
    {
        return;
    }

    // Only make local WASM files absolute
    if std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("wasm"))
    {
        *path = base.join(&*path).to_string_lossy().to_string();
    }
}
