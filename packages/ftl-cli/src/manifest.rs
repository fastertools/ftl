use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::language::Language;

// Type alias for backwards compatibility
pub type Manifest = ToolManifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub tool: ToolConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub optimization: OptimizationConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub language: Language,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizationConfig {
    #[serde(default)]
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolkitManifest {
    pub toolkit: ToolkitConfig,
    pub tools: Vec<ToolkitTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<GatewayConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolkitConfig {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolkitTool {
    pub name: String,
    pub route: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    #[serde(default = "default_gateway_enabled")]
    pub enabled: bool,
    #[serde(default = "default_gateway_route")]
    pub route: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
}

fn default_gateway_enabled() -> bool {
    false
}

fn default_gateway_route() -> String {
    "/gateway".to_string()
}

fn default_profile() -> String {
    "release".to_string()
}

impl ToolManifest {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read manifest from {:?}", path.as_ref()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse manifest from {:?}", path.as_ref()))
    }

    pub fn validate(&self) -> Result<()> {
        // Validate tool name (should be lowercase with hyphens)
        if !self
            .tool
            .name
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
        {
            anyhow::bail!("Tool name must be lowercase with hyphens (e.g., my-tool)");
        }

        // Validate version
        semver::Version::parse(&self.tool.version).context("Invalid version format")?;

        Ok(())
    }
}

impl ToolkitManifest {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read toolkit manifest from {:?}", path.as_ref()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse toolkit manifest from {:?}", path.as_ref()))
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content =
            toml::to_string_pretty(self).context("Failed to serialize toolkit manifest")?;

        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write toolkit manifest to {:?}", path.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let manifest_str = r#"
[tool]
name = "json-query"
version = "1.0.0"
description = "Query JSON data"

[build]
profile = "release"
features = ["simd"]

[optimization]
flags = ["-O4", "--enable-simd"]

[runtime]
allowed_hosts = []
"#;

        let manifest: ToolManifest = toml::from_str(manifest_str).unwrap();
        assert_eq!(manifest.tool.name, "json-query");
        assert_eq!(manifest.build.profile, "release");
        assert_eq!(manifest.optimization.flags.len(), 2);
    }
}
