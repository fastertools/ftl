//! Component resolver using wkg
//!
//! This module resolves registry-referenced components by pulling them
//! locally using wkg. This ensures all components are available locally
//! before transpiling to Spin TOML format.

use crate::ftl_resolve::FtlConfig;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

/// Check if wkg CLI tool is installed and available
pub fn check_wkg_available() -> Result<()> {
    let output = Command::new("wkg")
        .arg("--version")
        .output()
        .context("Failed to execute wkg. Please ensure wkg is installed: https://github.com/bytecodealliance/wasm-pkg-tools")?;

    if !output.status.success() {
        anyhow::bail!("wkg command failed. Please ensure wkg is installed and working");
    }

    Ok(())
}

/// Resolve a registry component reference to a local file
///
/// Downloads the component using wkg and returns the path to the downloaded file.
/// Files are placed in the current directory with a deterministic name.
pub fn resolve_registry_component(
    registry_ref: &str,
    output_dir: &Path,
    no_cache: bool,
) -> Result<PathBuf> {
    // Parse the registry reference
    let (registry_url, filename) = parse_registry_ref(registry_ref)?;

    // Create output path
    let output_path = output_dir.join(&filename);

    // Check if file already exists (can skip download unless no_cache is set)
    if !no_cache && output_path.exists() {
        return Ok(output_path);
    }

    // If no_cache is set and file exists, remove it
    if no_cache && output_path.exists() {
        std::fs::remove_file(&output_path)
            .with_context(|| format!("Failed to remove cached file: {}", output_path.display()))?;
    }

    // Pull the component using wkg
    // Use stdin/stdout/stderr configuration to prevent deadlocks in parallel execution
    let output = Command::new("wkg")
        .args([
            "oci",
            "pull",
            &registry_url,
            "-o",
            output_path.to_str().unwrap(),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("Failed to execute wkg oci pull")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to pull component {}: {}", registry_ref, stderr);
    }

    Ok(output_path)
}

/// Parse a registry reference into URL and suggested filename
fn parse_registry_ref(registry_ref: &str) -> Result<(String, String)> {
    // Registry references are in format: registry.domain/namespace/package:version
    // or registry.domain/package:version

    // Find the last colon to separate version
    let version_sep = registry_ref
        .rfind(':')
        .ok_or_else(|| anyhow::anyhow!("Invalid registry reference: missing version"))?;

    let package_part = &registry_ref[..version_sep];
    let version = &registry_ref[version_sep + 1..];

    // Extract package name from the path
    let package_name = package_part
        .rsplit('/')
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid registry reference: missing package name"))?;

    // Create a filename from package name and version
    let filename = format!("{}-{}.wasm", package_name, version);

    Ok((registry_ref.to_string(), filename))
}

/// Resolve all components in an FTL configuration
///
/// Returns a map of component names to their resolved local paths.
/// Registry components are downloaded in parallel for better performance.
pub fn resolve_all_components(
    ftl_resolve: &FtlConfig,
    project_dir: &Path,
    no_cache: bool,
) -> Result<HashMap<String, PathBuf>> {
    let output_dir = project_dir.join(".ftl").join("wasm");

    // Create output directory if needed
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).context("Failed to create .ftl/wasm directory")?;
    }

    // Collect all components that need to be resolved
    let mut components_to_resolve: Vec<(String, String)> = Vec::new();

    // Add MCP components if they are registry references
    if is_registry_ref(&ftl_resolve.mcp.gateway) {
        components_to_resolve.push(("mcp-gateway".to_string(), ftl_resolve.mcp.gateway.clone()));
    }

    if is_registry_ref(&ftl_resolve.mcp.authorizer) {
        components_to_resolve.push((
            "mcp-authorizer".to_string(),
            ftl_resolve.mcp.authorizer.clone(),
        ));
    }

    // Add user components
    for (name, component) in &ftl_resolve.component {
        if let Some(repo) = &component.repo {
            components_to_resolve.push((name.clone(), repo.clone()));
        }
    }

    // If there are components to resolve, check wkg is available first
    if !components_to_resolve.is_empty() {
        // Only check wkg if we might need to download something
        // Quick check if all components are already cached
        let all_cached = if !no_cache {
            components_to_resolve.iter().all(|(_, registry_ref)| {
                if let Ok((_, filename)) = parse_registry_ref(registry_ref) {
                    output_dir.join(&filename).exists()
                } else {
                    false
                }
            })
        } else {
            false
        };

        if !all_cached {
            check_wkg_available()?;
        }

        // Report what we're about to do
        if no_cache {
            if components_to_resolve.len() == 1 {
                eprintln!("Downloading 1 component (ignoring cache)...");
            } else {
                eprintln!(
                    "Downloading {} components in parallel (ignoring cache)...",
                    components_to_resolve.len()
                );
            }
        } else if all_cached {
            if components_to_resolve.len() == 1 {
                eprintln!("Using cached component...");
            } else {
                eprintln!("Using {} cached components...", components_to_resolve.len());
            }
        } else if components_to_resolve.len() == 1 {
            eprintln!("Resolving 1 component...");
        } else {
            eprintln!(
                "Resolving {} components in parallel...",
                components_to_resolve.len()
            );
        }
    }

    // Resolve all components in parallel
    let resolved = Mutex::new(HashMap::new());
    let errors = Mutex::new(Vec::new());

    components_to_resolve
        .par_iter()
        .for_each(|(name, registry_ref)| {
            // Check if using cache
            let using_cache = if !no_cache {
                if let Ok((_, filename)) = parse_registry_ref(registry_ref) {
                    output_dir.join(&filename).exists()
                } else {
                    false
                }
            } else {
                false
            };

            match resolve_registry_component(registry_ref, &output_dir, no_cache) {
                Ok(path) => {
                    if using_cache {
                        eprintln!("  ✓ Using cached {}", name);
                    } else {
                        eprintln!("  ✓ Downloaded {}", name);
                    }
                    resolved.lock().unwrap().insert(name.clone(), path);
                }
                Err(e) => {
                    eprintln!("  ✗ Failed to resolve {}", name);
                    errors
                        .lock()
                        .unwrap()
                        .push(format!("Failed to resolve {}: {}", name, e));
                }
            }
        });

    // Check if there were any errors
    let errors = errors.into_inner().unwrap();
    if !errors.is_empty() {
        anyhow::bail!("Failed to resolve components:\n{}", errors.join("\n"));
    }

    Ok(resolved.into_inner().unwrap())
}

/// Check if a string is a registry reference
fn is_registry_ref(s: &str) -> bool {
    // Registry references contain a domain/path and a version separated by colon
    s.contains('/') && s.contains(':') && !s.ends_with(".wasm")
}

/// Resolve and transpile an FTL configuration
///
/// This function resolves all registry components to local files,
/// then transpiles the configuration to Spin TOML format with
/// the resolved local paths.
pub fn resolve_and_transpile(
    ftl_resolve: &FtlConfig,
    project_dir: &Path,
    no_cache: bool,
) -> Result<String> {
    // Resolve all components
    let resolved_components = resolve_all_components(ftl_resolve, project_dir, no_cache)?;

    // Transpile with resolved paths
    crate::transpiler::create_spin_toml_with_resolved_paths(
        ftl_resolve,
        &resolved_components,
        project_dir,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_registry_ref() {
        let (url, filename) = parse_registry_ref("ghcr.io/fastertools/mcp-gateway:1.0.0").unwrap();
        assert_eq!(url, "ghcr.io/fastertools/mcp-gateway:1.0.0");
        assert_eq!(filename, "mcp-gateway-1.0.0.wasm");

        let (url, filename) = parse_registry_ref("docker.io/myorg/myapp:2.1.0-alpha").unwrap();
        assert_eq!(url, "docker.io/myorg/myapp:2.1.0-alpha");
        assert_eq!(filename, "myapp-2.1.0-alpha.wasm");
    }

    #[test]
    fn test_is_registry_ref() {
        assert!(is_registry_ref("ghcr.io/fastertools/mcp-gateway:1.0.0"));
        assert!(is_registry_ref("docker.io/library/nginx:latest"));
        assert!(!is_registry_ref("./local/path/to/component.wasm"));
        assert!(!is_registry_ref("component.wasm"));
        assert!(!is_registry_ref("/absolute/path/to/component.wasm"));
    }
}
