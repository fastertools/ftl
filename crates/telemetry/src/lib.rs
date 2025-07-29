//! Telemetry infrastructure for FTL CLI
//!
//! This crate provides privacy-first telemetry collection for the FTL CLI.
//! All telemetry is local-only by default, with remote telemetry requiring
//! explicit opt-in from users.

pub mod config;
pub mod events;
pub mod logger;
pub mod storage;

#[cfg(test)]
mod tests;

use anyhow::Result;
use std::path::PathBuf;

/// Main telemetry client
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
}