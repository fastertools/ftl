//! Configuration management system for FTL CLI
//!
//! This module provides a flexible, extensible configuration system that allows
//! different parts of the application to manage their own configuration sections
//! within a unified config file at ~/.ftl/config.toml.
//!
//! # Architecture
//!
//! The system is built around two core concepts:
//! - `ConfigSection`: A trait that any configuration section must implement
//! - `Config`: The main configuration manager that handles file I/O and section management
//!
//! # Example
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use ftl_common::config::{Config, ConfigSection};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct TelemetryConfig {
//!     installation_id: String,
//!     enabled: bool,
//! }
//!
//! impl ConfigSection for TelemetryConfig {
//!     fn section_name() -> &'static str {
//!         "telemetry"
//!     }
//! }
//!
//! // Load or create config
//! let mut config = Config::load()?;
//!
//! // Get a section
//! let telemetry = config.get_section::<TelemetryConfig>()?;
//!
//! // Update a section
//! let new_telemetry = TelemetryConfig {
//!     installation_id: "some-id".to_string(),
//!     enabled: true,
//! };
//! config.set_section(new_telemetry)?;
//! config.save()?;
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use toml::Value;

/// Trait that all configuration sections must implement
pub trait ConfigSection: Serialize + for<'de> Deserialize<'de> + Clone {
    /// Returns the name of this configuration section
    /// This will be used as the key in the top-level TOML table
    fn section_name() -> &'static str;
}

/// Main configuration manager
#[derive(Debug)]
pub struct Config {
    /// Path to the configuration file
    path: PathBuf,
    /// Raw TOML data stored as a map
    data: HashMap<String, Value>,
}

impl Config {
    /// Default configuration directory name
    pub const CONFIG_DIR: &'static str = ".ftl";

    /// Default configuration file name
    pub const CONFIG_FILE: &'static str = "config.toml";

    /// Load configuration from the default location (~/.ftl/config.toml)
    pub fn load() -> Result<Self> {
        let path = Self::default_config_path()?;
        Self::load_from_path(&path)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let data = if path.exists() {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file at {}", path.display()))?;

            if contents.trim().is_empty() {
                HashMap::new()
            } else {
                toml::from_str(&contents)
                    .with_context(|| format!("Failed to parse config file at {}", path.display()))?
            }
        } else {
            HashMap::new()
        };

        Ok(Self {
            path: path.to_path_buf(),
            data,
        })
    }

    /// Get the default configuration file path
    pub fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(Self::CONFIG_DIR).join(Self::CONFIG_FILE))
    }

    /// Get a configuration section
    pub fn get_section<T: ConfigSection>(&self) -> Result<Option<T>> {
        let section_name = T::section_name();

        match self.data.get(section_name) {
            Some(value) => {
                let section = value
                    .clone()
                    .try_into()
                    .with_context(|| format!("Failed to deserialize {section_name} section"))?;
                Ok(Some(section))
            }
            None => Ok(None),
        }
    }

    /// Set a configuration section
    pub fn set_section<T: ConfigSection>(&mut self, section: T) -> Result<()> {
        let section_name = T::section_name();
        let value = toml::Value::try_from(section)
            .with_context(|| format!("Failed to serialize {section_name} section"))?;

        self.data.insert(section_name.to_string(), value);
        Ok(())
    }

    /// Remove a configuration section
    pub fn remove_section<T: ConfigSection>(&mut self) -> Option<Value> {
        let section_name = T::section_name();
        self.data.remove(section_name)
    }

    /// Save the configuration to disk
    pub fn save(&self) -> Result<()> {
        // Ensure the directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory at {}", parent.display())
            })?;
        }

        // Serialize to pretty TOML
        let contents =
            toml::to_string_pretty(&self.data).context("Failed to serialize config data")?;

        fs::write(&self.path, contents)
            .with_context(|| format!("Failed to write config file at {}", self.path.display()))?;

        Ok(())
    }

    /// Get the path to the configuration file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if a section exists
    pub fn has_section<T: ConfigSection>(&self) -> bool {
        self.data.contains_key(T::section_name())
    }

    /// Get all section names
    pub fn section_names(&self) -> Vec<&str> {
        self.data.keys().map(String::as_str).collect()
    }

    /// Clear all configuration data
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// Helper function to load or create a configuration section
///
/// This function loads the configuration, gets or creates the specified section,
/// and returns both the section and whether it was newly created.
pub fn load_or_create_section<T: ConfigSection + Default>() -> Result<(T, bool)> {
    let mut config = Config::load()?;

    if let Some(section) = config.get_section::<T>()? {
        Ok((section, false))
    } else {
        let section = T::default();
        config.set_section(section.clone())?;
        config.save()?;
        Ok((section, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestSection {
        value: String,
        enabled: bool,
    }

    impl ConfigSection for TestSection {
        fn section_name() -> &'static str {
            "test"
        }
    }

    impl Default for TestSection {
        fn default() -> Self {
            Self {
                value: "default".to_string(),
                enabled: true,
            }
        }
    }

    #[test]
    fn test_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::load_from_path(&config_path).unwrap();
        assert!(config.section_names().is_empty());
    }

    #[test]
    fn test_section_management() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::load_from_path(&config_path).unwrap();

        // Test getting non-existent section
        assert!(config.get_section::<TestSection>().unwrap().is_none());

        // Test setting section
        let section = TestSection {
            value: "test_value".to_string(),
            enabled: false,
        };
        config.set_section(section.clone()).unwrap();

        // Test getting section
        let retrieved = config.get_section::<TestSection>().unwrap().unwrap();
        assert_eq!(retrieved, section);

        // Test has_section
        assert!(config.has_section::<TestSection>());

        // Test saving and reloading
        config.save().unwrap();
        let config2 = Config::load_from_path(&config_path).unwrap();
        let retrieved2 = config2.get_section::<TestSection>().unwrap().unwrap();
        assert_eq!(retrieved2, section);
    }

    #[test]
    fn test_multiple_sections() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct AnotherSection {
            name: String,
        }

        impl ConfigSection for AnotherSection {
            fn section_name() -> &'static str {
                "another"
            }
        }

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::load_from_path(&config_path).unwrap();

        config.set_section(TestSection::default()).unwrap();
        config
            .set_section(AnotherSection {
                name: "test".to_string(),
            })
            .unwrap();

        assert_eq!(config.section_names().len(), 2);
        assert!(config.has_section::<TestSection>());
        assert!(config.has_section::<AnotherSection>());
    }

    #[test]
    fn test_remove_section() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::load_from_path(&config_path).unwrap();

        // Add a section
        config.set_section(TestSection::default()).unwrap();
        assert!(config.has_section::<TestSection>());

        // Remove the section
        let removed = config.remove_section::<TestSection>();
        assert!(removed.is_some());
        assert!(!config.has_section::<TestSection>());

        // Save and reload to verify persistence
        config.save().unwrap();
        let config2 = Config::load_from_path(&config_path).unwrap();
        assert!(!config2.has_section::<TestSection>());
    }

    #[test]
    fn test_clear_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::load_from_path(&config_path).unwrap();

        // Add some sections
        config.set_section(TestSection::default()).unwrap();
        assert!(!config.section_names().is_empty());

        // Clear all sections
        config.clear();
        assert!(config.section_names().is_empty());

        // Save and verify
        config.save().unwrap();
        let config2 = Config::load_from_path(&config_path).unwrap();
        assert!(config2.section_names().is_empty());
    }

    #[test]
    fn test_load_or_create_with_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create a config at a specific path
        let mut config = Config::load_from_path(&config_path).unwrap();

        // First time - section doesn't exist
        assert!(config.get_section::<TestSection>().unwrap().is_none());

        // Add default section
        config.set_section(TestSection::default()).unwrap();
        config.save().unwrap();

        // Load again and verify
        let config2 = Config::load_from_path(&config_path).unwrap();
        let section = config2.get_section::<TestSection>().unwrap().unwrap();
        assert_eq!(section, TestSection::default());
    }

    #[test]
    fn test_empty_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create an empty file
        fs::write(&config_path, "").unwrap();

        // Should handle empty file gracefully
        let config = Config::load_from_path(&config_path).unwrap();
        assert!(config.section_names().is_empty());
    }

    #[test]
    fn test_malformed_json_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write malformed TOML
        fs::write(&config_path, "[invalid toml").unwrap();

        // Should return an error
        let result = Config::load_from_path(&config_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse config file")
        );
    }

    #[test]
    fn test_nested_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("deep/nested/path/config.json");

        let mut config = Config::load_from_path(&config_path).unwrap();
        config.set_section(TestSection::default()).unwrap();

        // Should create all parent directories
        config.save().unwrap();
        assert!(config_path.exists());
        assert!(config_path.parent().unwrap().exists());
    }
}
