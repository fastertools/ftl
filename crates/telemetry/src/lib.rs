//! Telemetry infrastructure for FTL CLI
//!
//! This crate provides privacy-first telemetry collection for the FTL CLI.
//! All telemetry is local-only by default, with remote telemetry requiring
//! explicit opt-in from users.

pub mod config;
pub mod events;
pub mod logger;
pub mod notice;
pub mod privacy;

#[cfg(test)]
mod tests;

use anyhow::Result;
use std::path::PathBuf;

/// Main telemetry client
#[derive(Clone)]
pub struct TelemetryClient {
    config: config::TelemetryConfig,
    logger: logger::TelemetryLogger,
}

impl TelemetryClient {
    /// Create a new telemetry client
    pub fn new() -> Result<Self> {
        let config = config::TelemetryConfig::load()?;
        let logger = logger::TelemetryLogger::new(&config)?;
        
        Ok(Self { config, logger })
    }
    
    /// Create a telemetry client from a specific configuration
    pub fn from_config(config: config::TelemetryConfig) -> Result<Self> {
        let logger = logger::TelemetryLogger::new(&config)?;
        Ok(Self { config, logger })
    }
    
    /// Initialize telemetry and show first-run notice if needed
    pub async fn initialize() -> Result<Self> {
        // Check if we should show the notice
        if notice::should_show_notice()? {
            if notice::is_interactive() {
                notice::show_notice()?;
            } else {
                notice::show_notice_non_interactive()?;
            }
        }
        
        let client = Self::new()?;
        
        // Clean up old log files on startup
        if client.is_enabled() {
            if let Err(e) = client.logger.cleanup().await {
                tracing::debug!("Failed to clean up old telemetry logs: {}", e);
            }
        }
        
        Ok(client)
    }
    
    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.is_enabled()
    }
    
    /// Log an event
    pub async fn log_event(&self, event: events::TelemetryEvent) -> Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }
        
        self.logger.log(event).await
    }
    
    /// Get the telemetry log directory
    pub fn log_directory(&self) -> PathBuf {
        self.logger.log_directory()
    }
    
    /// Get the installation ID
    pub fn installation_id(&self) -> &str {
        &self.config.installation_id
    }
}