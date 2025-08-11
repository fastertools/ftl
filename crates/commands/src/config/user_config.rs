//! User configuration management for FTL CLI
//!
//! This module handles reading and writing user-specific configuration
//! stored in ~/.ftl/config.json

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User configuration stored in ~/.ftl/config.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    /// Selected organization for deployments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_org: Option<OrgSelection>,
}

/// Organization selection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgSelection {
    /// Organization ID from `WorkOS`
    pub id: String,
    /// Organization name for display
    pub name: String,
    /// When this selection was made
    pub selected_at: chrono::DateTime<chrono::Utc>,
}

impl UserConfig {
    /// Load user configuration from disk
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {}", config_path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config from {}", config_path.display()))
    }

    /// Save user configuration to disk
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Ensure the directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory {}", parent.display())
            })?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {}", config_path.display()))?;

        Ok(())
    }

    /// Get the path to the config file
    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".ftl").join("config.json"))
    }

    /// Set the selected organization
    pub fn set_selected_org(&mut self, id: String, name: String) {
        self.selected_org = Some(OrgSelection {
            id,
            name,
            selected_at: chrono::Utc::now(),
        });
    }

    /// Clear the selected organization
    pub fn clear_selected_org(&mut self) {
        self.selected_org = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_user_config_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Create a config with org selection
        let mut config = UserConfig::default();
        config.set_selected_org("org_123".to_string(), "Test Org".to_string());

        // Serialize to JSON
        let json = serde_json::to_string(&config).unwrap();
        fs::write(&config_path, json).unwrap();

        // Read back and verify
        let content = fs::read_to_string(&config_path).unwrap();
        let loaded: UserConfig = serde_json::from_str(&content).unwrap();

        assert!(loaded.selected_org.is_some());
        let org = loaded.selected_org.unwrap();
        assert_eq!(org.id, "org_123");
        assert_eq!(org.name, "Test Org");
    }

    #[test]
    fn test_default_config() {
        let config = UserConfig::default();
        assert!(config.selected_org.is_none());
    }
}
