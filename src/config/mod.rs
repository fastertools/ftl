use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub mod registry;

use registry::RegistryConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct FtlConfig {
    pub version: String,
    pub default_registry: String,
    pub registries: Vec<RegistryConfig>,
}

impl FtlConfig {
    /// Create a new default configuration
    pub fn default() -> Self {
        Self {
            version: "1".to_string(),
            default_registry: "ghcr".to_string(),
            registries: vec![
                RegistryConfig::new_ghcr("ghcr".to_string(), "fastertools".to_string()),
                RegistryConfig::new_docker("docker".to_string()),
                {
                    let mut ecr = RegistryConfig::new_ecr("ecr".to_string(), None, None);
                    ecr.enabled = false; // Disabled by default as it requires AWS credentials
                    ecr
                },
            ],
        }
    }

    /// Get the path to the user's FTL config directory
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".ftl"))
    }

    /// Get the path to the registries config file
    pub fn registries_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("registries.toml"))
    }

    /// Load configuration from disk, creating default if it doesn't exist
    pub fn load() -> Result<Self> {
        let path = Self::registries_path()?;
        
        if !path.exists() {
            // Create default config if it doesn't exist
            let config = Self::default();
            config.save()?;
            
            println!("→ Created default registry configuration at {}", path.display());
            println!("→ Use 'ftl registries list' to manage your registries");
            
            return Ok(config);
        }

        let contents = fs::read_to_string(&path)
            .context("Failed to read registry configuration")?;
        
        let config: Self = toml::from_str(&contents)
            .context("Failed to parse registry configuration")?;
        
        Ok(config)
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::registries_path()?;
        
        // Ensure the config directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        
        fs::write(&path, contents)
            .context("Failed to write configuration")?;
        
        Ok(())
    }

    /// Get a registry by name
    pub fn get_registry(&self, name: &str) -> Option<&RegistryConfig> {
        self.registries.iter().find(|r| r.name == name)
    }

    /// Get all enabled registries sorted by priority
    pub fn enabled_registries(&self) -> Vec<&RegistryConfig> {
        let mut registries: Vec<_> = self.registries
            .iter()
            .filter(|r| r.enabled)
            .collect();
        
        registries.sort_by_key(|r| r.priority);
        registries
    }

    /// Add a new registry
    pub fn add_registry(&mut self, registry: RegistryConfig) -> Result<()> {
        // Check for duplicate names
        if self.registries.iter().any(|r| r.name == registry.name) {
            anyhow::bail!("Registry '{}' already exists", registry.name);
        }
        
        self.registries.push(registry);
        Ok(())
    }

    /// Remove a registry by name
    pub fn remove_registry(&mut self, name: &str) -> Result<()> {
        let initial_len = self.registries.len();
        self.registries.retain(|r| r.name != name);
        
        if self.registries.len() == initial_len {
            anyhow::bail!("Registry '{}' not found", name);
        }
        
        // Update default if we removed it
        if self.default_registry == name {
            self.default_registry = self.registries
                .first()
                .map(|r| r.name.clone())
                .unwrap_or_default();
        }
        
        Ok(())
    }

    /// Set the default registry
    pub fn set_default(&mut self, name: &str) -> Result<()> {
        if !self.registries.iter().any(|r| r.name == name) {
            anyhow::bail!("Registry '{}' not found", name);
        }
        
        self.default_registry = name.to_string();
        Ok(())
    }

    /// Enable or disable a registry
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        let registry = self.registries
            .iter_mut()
            .find(|r| r.name == name)
            .context(format!("Registry '{}' not found", name))?;
        
        registry.enabled = enabled;
        Ok(())
    }

    /// Set registry priority
    pub fn set_priority(&mut self, name: &str, priority: u32) -> Result<()> {
        let registry = self.registries
            .iter_mut()
            .find(|r| r.name == name)
            .context(format!("Registry '{}' not found", name))?;
        
        registry.priority = priority;
        Ok(())
    }
}

/// Load project-specific configuration if it exists
#[allow(dead_code)]
pub fn load_project_config(project_root: &Path) -> Option<FtlConfig> {
    let config_path = project_root.join(".ftl").join("registries.toml");
    
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(contents) => {
                match toml::from_str(&contents) {
                    Ok(config) => Some(config),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse project config: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read project config: {}", e);
                None
            }
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::registry::RegistryType;

    #[test]
    fn test_default_config() {
        let config = FtlConfig::default();
        assert_eq!(config.version, "1");
        assert_eq!(config.default_registry, "ghcr");
        assert_eq!(config.registries.len(), 3);
    }

    #[test]
    fn test_enabled_registries() {
        let config = FtlConfig::default();
        let enabled = config.enabled_registries();
        assert_eq!(enabled.len(), 2); // ghcr and docker are enabled by default
        assert_eq!(enabled[0].name, "ghcr"); // Priority 1
        assert_eq!(enabled[1].name, "docker"); // Priority 2
    }

    #[test]
    fn test_add_registry() {
        let mut config = FtlConfig::default();
        let new_registry = RegistryConfig {
            name: "custom".to_string(),
            registry_type: RegistryType::Custom,
            enabled: true,
            priority: 4,
            display_url: Some("https://registry.example.com".to_string()),
            config: serde_json::json!({
                "url_pattern": "registry.example.com/{image_name}:latest"
            }),
        };
        
        config.add_registry(new_registry).unwrap();
        assert_eq!(config.registries.len(), 4);
        
        // Should fail on duplicate
        let duplicate = RegistryConfig {
            name: "ghcr".to_string(),
            registry_type: RegistryType::Ghcr,
            enabled: true,
            priority: 5,
            display_url: Some("https://github.com/orgs/fastertools/packages".to_string()),
            config: serde_json::json!({}),
        };
        
        assert!(config.add_registry(duplicate).is_err());
    }

    #[test]
    fn test_remove_registry() {
        let mut config = FtlConfig::default();
        
        config.remove_registry("docker").unwrap();
        assert_eq!(config.registries.len(), 2);
        
        // Should fail on non-existent
        assert!(config.remove_registry("nonexistent").is_err());
    }

    #[test]
    fn test_set_default() {
        let mut config = FtlConfig::default();
        
        config.set_default("docker").unwrap();
        assert_eq!(config.default_registry, "docker");
        
        // Should fail on non-existent
        assert!(config.set_default("nonexistent").is_err());
    }
}