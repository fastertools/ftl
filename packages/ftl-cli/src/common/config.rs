use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FtlConfig {
    pub username: Option<String>,
}

impl FtlConfig {
    /// Get the path to the global FTL config directory
    fn config_dir() -> Result<PathBuf> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".ftl"))
    }

    /// Get the path to the config file
    fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    /// Load config from disk
    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;

        if !config_file.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_file).with_context(|| {
            let display = config_file.display();
            format!("Failed to read config from {display}")
        })?;

        let config =
            serde_json::from_str(&content).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        fs::create_dir_all(&config_dir).with_context(|| {
            let display = config_dir.display();
            format!("Failed to create config directory at {display}")
        })?;

        let config_file = Self::config_file()?;
        let content = serde_json::to_string_pretty(self)?;

        fs::write(&config_file, content).with_context(|| {
            let display = config_file.display();
            format!("Failed to write config to {display}")
        })?;

        Ok(())
    }

    /// Get the app prefix for deployments
    pub fn get_app_prefix(&self) -> String {
        self.username
            .as_ref()
            .map(|u| format!("{u}-"))
            .unwrap_or_else(|| "ftl-".to_string())
    }
}
