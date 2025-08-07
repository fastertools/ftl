use anyhow::{Context, Result};
use std::process::Command;

/// Registry components for Spin manifest generation
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryComponents {
    /// Full registry URL (e.g., "ghcr.io/fastertools/my-tool")
    pub registry_url: String,
    /// Semantic version (e.g., "1.0.0", "2.1.3-alpha")
    pub version: String,
}

/// Parse image name and tag, defaulting to "latest" if no tag specified
pub fn parse_image_and_tag(image_name: &str) -> (String, String) {
    if let Some(pos) = image_name.rfind(':') {
        let image = image_name[..pos].to_string();
        let tag = image_name[pos + 1..].to_string();
        (image, tag)
    } else {
        (image_name.to_string(), "latest".to_string())
    }
}

/// Resolve a component reference to a full registry URL
///
/// # Arguments
/// * `component` - Component reference (e.g., "my-tool:1.0.0" or "ghcr.io/org/tool:1.0.0")
/// * `default_registry` - Default registry to use for short references (e.g., "ghcr.io/fastertools")
///
/// # Returns
/// Full registry URL with version
pub fn resolve_registry_url(component: &str, default_registry: Option<&str>) -> String {
    // Check if it's a local file
    if component.to_lowercase().ends_with(".wasm") {
        return component.to_string();
    }

    // Check if it already has a registry domain
    if component.contains("://")
        || component.starts_with("ghcr.io/")
        || component.starts_with("docker.io/")
        || component.contains(".amazonaws.com/")
    {
        return component.to_string();
    }

    // Check if it has an organization/user prefix
    let (image_without_tag, _tag) = parse_image_and_tag(component);
    if image_without_tag.matches('/').count() >= 2 {
        // Already has registry/org/repo format
        return component.to_string();
    }

    // Use default registry
    let default = default_registry.unwrap_or("ghcr.io/fastertools");

    // Handle Docker Hub library images (no org prefix)
    if default.starts_with("docker.io") && !image_without_tag.contains('/') {
        format!("docker.io/library/{component}")
    } else {
        format!("{default}/{component}")
    }
}

/// Check if crane CLI tool is installed and available
fn check_crane_available() -> Result<()> {
    let output = Command::new("crane")
        .arg("version")
        .output()
        .context("Failed to execute crane. Please ensure crane is installed: https://github.com/google/go-containerregistry/blob/main/cmd/crane/README.md")?;

    if !output.status.success() {
        anyhow::bail!("crane command failed. Please ensure crane is installed and working");
    }

    Ok(())
}

/// Check if wkg CLI tool is installed and available
fn check_wkg_available() -> Result<()> {
    let output = Command::new("wkg")
        .arg("--version")
        .output()
        .context("Failed to execute wkg. Please ensure wkg is installed: https://github.com/bytecodealliance/wasm-pkg-tools")?;

    if !output.status.success() {
        anyhow::bail!("wkg command failed. Please ensure wkg is installed and working");
    }

    Ok(())
}

/// Check if a component exists in a registry using crane
///
/// # Arguments
/// * `image_ref` - Full image reference (e.g., "ghcr.io/org/image:tag")
pub fn verify_component_exists(image_ref: &str) -> Result<bool> {
    check_crane_available()?;

    let output = Command::new("crane")
        .arg("manifest")
        .arg(image_ref)
        .output()
        .context("Failed to execute crane manifest")?;

    Ok(output.status.success())
}

/// List all available tags for a component using crane
///
/// # Arguments
/// * `repository` - Full repository path (e.g., "ghcr.io/org/image")
pub fn list_component_tags(repository: &str) -> Result<Vec<String>> {
    check_crane_available()?;

    let output = Command::new("crane")
        .arg("ls")
        .arg(repository)
        .output()
        .context("Failed to execute crane ls")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list tags: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tags: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(tags)
}

/// Pull a WebAssembly component from a registry to a local file
///
/// # Arguments
/// * `registry_url` - Full registry URL with tag (e.g., "ghcr.io/org/component:1.0.0")
/// * `output_path` - Path where the WASM file should be saved
pub fn pull_component(registry_url: &str, output_path: &str) -> Result<()> {
    check_wkg_available()?;

    let output = Command::new("wkg")
        .args(["oci", "pull", registry_url, "-o", output_path])
        .output()
        .context("Failed to execute wkg oci pull")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to pull component: {}", stderr);
    }

    Ok(())
}

/// Push a WebAssembly component to a registry
///
/// # Arguments
/// * `registry_url` - Full registry URL with tag (e.g., "ghcr.io/org/component:1.0.0")
/// * `wasm_path` - Path to the WASM file to push
pub fn push_component(registry_url: &str, wasm_path: &str) -> Result<()> {
    check_wkg_available()?;

    let output = Command::new("wkg")
        .args(["oci", "push", registry_url, wasm_path])
        .output()
        .context("Failed to execute wkg oci push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to push component: {}", stderr);
    }

    Ok(())
}

/// Resolve "latest" tag to the actual latest semantic version from registry
pub fn resolve_latest_version(repository: &str) -> Result<String> {
    use semver::Version;

    let tags = list_component_tags(repository)?;

    // Filter and parse semantic versions
    let mut semver_tags: Vec<Version> = Vec::new();

    for tag in tags {
        // Skip non-semver tags
        if tag == "latest" || tag == "main" || tag == "master" || tag == "dev" || tag == "edge" {
            continue;
        }

        // Try to parse as semver (with or without 'v' prefix)
        let clean_tag = tag.strip_prefix('v').unwrap_or(&tag);
        if let Ok(version) = Version::parse(clean_tag) {
            semver_tags.push(version);
        }
    }

    if semver_tags.is_empty() {
        anyhow::bail!("No semantic versions found for component at {}", repository);
    }

    // Sort versions and get the latest
    semver_tags.sort();
    let latest_version = semver_tags.last().unwrap();

    Ok(latest_version.to_string())
}

/// Validate and normalize a semantic version
pub fn validate_and_normalize_semver(version: &str) -> Result<String> {
    use semver::Version;

    // Remove common prefixes
    let clean_version = version.trim_start_matches('v');

    // Check for common non-semver tags first
    match clean_version {
        "latest" | "main" | "master" | "stable" | "dev" | "edge" => {
            anyhow::bail!(
                "Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)",
                version
            )
        }
        _ => {}
    }

    // Try to parse as semver
    match Version::parse(clean_version) {
        Ok(v) => Ok(v.to_string()),
        Err(_) => {
            // If it's not valid semver, try to make it valid (only if it looks like a number)
            if clean_version.chars().all(|c| c.is_numeric() || c == '.') {
                if clean_version.matches('.').count() == 0 {
                    // Single number like "1" -> "1.0.0"
                    Ok(format!("{clean_version}.0.0"))
                } else if clean_version.matches('.').count() == 1 {
                    // Two numbers like "1.2" -> "1.2.0"
                    Ok(format!("{clean_version}.0"))
                } else {
                    anyhow::bail!(
                        "Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)",
                        version
                    )
                }
            } else {
                anyhow::bail!(
                    "Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)",
                    version
                )
            }
        }
    }
}

/// Get registry components for a given image name
///
/// This function resolves the image to a full registry URL and validates the version.
/// If the version is "latest", it resolves to the actual latest semantic version.
pub fn get_registry_components(
    image_name: &str,
    default_registry: Option<&str>,
) -> Result<RegistryComponents> {
    let full_url = resolve_registry_url(image_name, default_registry);
    let (repository, tag) = parse_image_and_tag(&full_url);

    let version = if tag == "latest" {
        resolve_latest_version(&repository)?
    } else {
        validate_and_normalize_semver(&tag)?
    };

    // Verify the component exists
    let versioned_url = format!("{repository}:{version}");
    if !verify_component_exists(&versioned_url)? {
        // Try with 'v' prefix
        let versioned_url_with_v = format!("{repository}:v{version}");
        if !verify_component_exists(&versioned_url_with_v)? {
            anyhow::bail!(
                "Component '{}' with version '{}' not found in registry",
                repository,
                version
            );
        }
    }

    Ok(RegistryComponents {
        registry_url: repository,
        version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_registry_url() {
        // Test with full URLs
        assert_eq!(
            resolve_registry_url("ghcr.io/org/tool:1.0.0", None),
            "ghcr.io/org/tool:1.0.0"
        );

        // Test with short names and default registry
        assert_eq!(
            resolve_registry_url("my-tool:1.0.0", Some("ghcr.io/myorg")),
            "ghcr.io/myorg/my-tool:1.0.0"
        );

        // Test Docker Hub library handling
        assert_eq!(
            resolve_registry_url("nginx", Some("docker.io")),
            "docker.io/library/nginx"
        );

        // Test local WASM file
        assert_eq!(
            resolve_registry_url("target/my-component.wasm", None),
            "target/my-component.wasm"
        );
    }

    #[test]
    fn test_parse_image_and_tag() {
        assert_eq!(
            parse_image_and_tag("nginx"),
            ("nginx".to_string(), "latest".to_string())
        );
        assert_eq!(
            parse_image_and_tag("nginx:1.21"),
            ("nginx".to_string(), "1.21".to_string())
        );
        assert_eq!(
            parse_image_and_tag("user/app:v1.0"),
            ("user/app".to_string(), "v1.0".to_string())
        );
    }

    #[test]
    fn test_validate_and_normalize_semver() {
        // Valid semantic versions
        assert_eq!(validate_and_normalize_semver("1.0.0").unwrap(), "1.0.0");
        assert_eq!(validate_and_normalize_semver("v1.0.0").unwrap(), "1.0.0");
        assert_eq!(
            validate_and_normalize_semver("2.1.3-alpha").unwrap(),
            "2.1.3-alpha"
        );

        // Auto-completion of versions
        assert_eq!(validate_and_normalize_semver("1").unwrap(), "1.0.0");
        assert_eq!(validate_and_normalize_semver("1.2").unwrap(), "1.2.0");

        // Invalid versions should return error
        assert!(validate_and_normalize_semver("latest").is_err());
        assert!(validate_and_normalize_semver("main").is_err());
    }
}
