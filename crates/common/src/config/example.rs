//! Example usage of the FTL configuration system
//!
//! This file demonstrates how to use the config system for different use cases.

#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::{Config, ConfigSection, load_or_create_section};

/// Example: Telemetry configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub installation_id: String,
    pub telemetry_enabled: bool,
}

impl ConfigSection for TelemetryConfig {
    fn section_name() -> &'static str {
        "telemetry"
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            installation_id: uuid::Uuid::new_v4().to_string(),
            telemetry_enabled: true,
        }
    }
}

/// Example: Authentication configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

impl ConfigSection for AuthConfig {
    fn section_name() -> &'static str {
        "auth"
    }
}

/// Example: Registry configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub default_registry: String,
    pub registries: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub url: String,
    pub auth_required: bool,
}

impl ConfigSection for RegistryConfig {
    fn section_name() -> &'static str {
        "registry"
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            default_registry: "ghcr.io".to_string(),
            registries: vec![
                RegistryEntry {
                    name: "ghcr.io".to_string(),
                    url: "https://ghcr.io".to_string(),
                    auth_required: true,
                },
            ],
        }
    }
}

/// Example 1: Basic usage - loading and saving a section
pub fn example_basic_usage() -> Result<()> {
    // Load the configuration
    let mut config = Config::load()?;

    // Get or create telemetry config
    let telemetry = match config.get_section::<TelemetryConfig>()? {
        Some(t) => t,
        None => {
            let new_telemetry = TelemetryConfig::default();
            config.set_section(new_telemetry.clone())?;
            config.save()?;
            new_telemetry
        }
    };

    println!("Installation ID: {}", telemetry.installation_id);
    println!("Telemetry enabled: {}", telemetry.telemetry_enabled);

    Ok(())
}

/// Example 2: Using the load_or_create helper
pub fn example_load_or_create() -> Result<()> {
    // This automatically creates the section if it doesn't exist
    let (telemetry, was_created) = load_or_create_section::<TelemetryConfig>()?;

    if was_created {
        println!("Created new telemetry configuration");
    } else {
        println!("Loaded existing telemetry configuration");
    }

    println!("Installation ID: {}", telemetry.installation_id);

    Ok(())
}

/// Example 3: Managing multiple sections
pub fn example_multiple_sections() -> Result<()> {
    let mut config = Config::load()?;

    // Set telemetry config
    config.set_section(TelemetryConfig::default())?;

    // Set registry config
    config.set_section(RegistryConfig::default())?;

    // Set auth config
    config.set_section(AuthConfig {
        access_token: Some("example-token".to_string()),
        refresh_token: None,
        expires_at: None,
    })?;

    // Save all sections at once
    config.save()?;

    // List all sections
    println!("Config sections: {:?}", config.section_names());

    Ok(())
}

/// Example 4: Updating a specific field
pub fn example_update_field() -> Result<()> {
    let mut config = Config::load()?;

    // Get current telemetry config
    let mut telemetry = config
        .get_section::<TelemetryConfig>()?
        .unwrap_or_default();

    // Update a field
    telemetry.telemetry_enabled = false;

    // Save the updated section
    config.set_section(telemetry)?;
    config.save()?;

    println!("Telemetry has been disabled");

    Ok(())
}

/// Example 5: Checking if a section exists
pub fn example_check_section_exists() -> Result<()> {
    let config = Config::load()?;

    if config.has_section::<AuthConfig>() {
        println!("User is authenticated");
        let auth = config.get_section::<AuthConfig>()?.unwrap();
        if auth.access_token.is_some() {
            println!("Access token is present");
        }
    } else {
        println!("User is not authenticated");
    }

    Ok(())
}

/// Example 6: Removing a section
pub fn example_remove_section() -> Result<()> {
    let mut config = Config::load()?;

    // Remove auth section (e.g., on logout)
    if config.has_section::<AuthConfig>() {
        config.remove_section::<AuthConfig>();
        config.save()?;
        println!("Authentication data removed");
    }

    Ok(())
}

/// Example 7: Custom configuration path
pub fn example_custom_path() -> Result<()> {
    use std::path::Path;

    // Load from a custom path
    let custom_path = Path::new("/tmp/my-app/config.json");
    let mut config = Config::load_from_path(custom_path)?;

    // Use it normally
    config.set_section(TelemetryConfig::default())?;
    config.save()?;

    println!("Config saved to: {:?}", config.path());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_examples_compile() {
        // This test just ensures our examples compile
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // We don't actually run the examples in tests as they interact with the filesystem
        // but having them here ensures they continue to compile correctly
    }
}