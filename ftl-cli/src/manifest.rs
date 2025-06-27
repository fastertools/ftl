use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_memory_limit")]
    pub memory_limit: String,
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            memory_limit: default_memory_limit(),
            allowed_hosts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolkitManifest {
    pub toolkit: ToolkitConfig,
    pub tools: Vec<ToolkitTool>,
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

fn default_profile() -> String {
    "release".to_string()
}

fn default_memory_limit() -> String {
    "10MB".to_string()
}

impl ToolManifest {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read manifest from {:?}", path.as_ref()))?;
        
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse manifest from {:?}", path.as_ref()))
    }
    
    #[allow(dead_code)]
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize manifest")?;
        
        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write manifest to {:?}", path.as_ref()))
    }
    
    pub fn validate(&self) -> Result<()> {
        // Validate tool name (should be lowercase with hyphens)
        if !self.tool.name.chars().all(|c| c.is_lowercase() || c == '-' || c.is_numeric()) {
            anyhow::bail!("Tool name must be lowercase with hyphens (e.g., my-tool)");
        }
        
        // Validate version
        semver::Version::parse(&self.tool.version)
            .context("Invalid version format")?;
        
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
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize toolkit manifest")?;
        
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
memory_limit = "10MB"
allowed_hosts = []
"#;
        
        let manifest: ToolManifest = toml::from_str(manifest_str).unwrap();
        assert_eq!(manifest.tool.name, "json-query");
        assert_eq!(manifest.build.profile, "release");
        assert_eq!(manifest.optimization.flags.len(), 2);
    }
}