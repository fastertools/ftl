use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::language::Language;

/// Component manifest (ftl.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentManifest {
    pub component: ComponentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
}

impl ComponentManifest {
    pub fn load<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let manifest_path = base_path.as_ref().join("ftl.toml");
        let content = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read ftl.toml from {:?}", manifest_path))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse ftl.toml from {:?}", manifest_path))
    }
}
