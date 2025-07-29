//! Telemetry configuration management

use anyhow::Result;
use ftl_common::config::{Config, ConfigSection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,
    
    /// Unique installation ID
    pub installation_id: String,
    
    /// Whether to upload telemetry (future feature)
    #[serde(default)]
    pub upload_enabled: bool,
    
    /// Log directory path
    #[serde(skip)]
    pub log_directory: PathBuf,
    
    /// Log retention in days
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

fn default_retention_days() -> u32 {
    30
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            enabled: true,
            installation_id: uuid::Uuid::new_v4().to_string(),
            upload_enabled: false,
            log_directory: home.join(".ftl").join("logs"),
            retention_days: default_retention_days(),
        }
    }
}

impl ConfigSection for TelemetryConfig {
    fn section_name() -> &'static str {
        "telemetry"
    }
}

impl TelemetryConfig {
    /// Load telemetry configuration
    pub fn load() -> Result<Self> {
        // Check environment variable first
        if std::env::var("FTL_TELEMETRY_DISABLED").is_ok() {
            return Ok(Self {
                enabled: false,
                ..Default::default()
            });
        }
        
        // Load from config file
        let config = Config::load()?;
        let mut telemetry_config = config.get_section::<TelemetryConfig>()?
            .unwrap_or_default();
        
        // Set the log directory based on home
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        telemetry_config.log_directory = home.join(".ftl").join("logs");
        
        Ok(telemetry_config)
    }
    
    /// Load telemetry configuration from a specific path
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let config = Config::load_from_path(path)?;
        let mut telemetry_config = config.get_section::<TelemetryConfig>()?
            .ok_or_else(|| anyhow::anyhow!("Telemetry configuration not found"))?;
        
        // Set the log directory relative to the config path
        if let Some(parent) = path.parent() {
            telemetry_config.log_directory = parent.join(".ftl").join("logs");
        }
        
        Ok(telemetry_config)
    }
    
    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled && !std::env::var("FTL_TELEMETRY_DISABLED").is_ok()
    }
    
    /// Save configuration
    pub fn save(&self) -> Result<()> {
        let mut config = Config::load()?;
        config.set_section(self.clone())?;
        config.save()?;
        Ok(())
    }
}