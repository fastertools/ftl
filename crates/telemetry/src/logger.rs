//! Local telemetry logging

#[cfg(test)]
mod tests;

use crate::{config::TelemetryConfig, events::TelemetryEvent};
use anyhow::Result;
use chrono::{DateTime, Local};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Telemetry logger for local file storage
#[derive(Clone)]
pub struct TelemetryLogger {
    log_dir: PathBuf,
    retention_days: u32,
}

impl TelemetryLogger {
    /// Create a new telemetry logger
    pub fn new(config: &TelemetryConfig) -> Result<Self> {
        let log_dir = config.log_directory.join(&config.installation_id);
        Ok(Self { 
            log_dir,
            retention_days: config.retention_days,
        })
    }
    
    /// Get the log directory
    pub fn log_directory(&self) -> PathBuf {
        self.log_dir.clone()
    }
    
    /// Log an event to a local file
    pub async fn log(&self, event: TelemetryEvent) -> Result<()> {
        // Create log directory if it doesn't exist
        fs::create_dir_all(&self.log_dir).await?;
        
        // Create daily log file
        let date = Local::now().format("%Y-%m-%d");
        let log_file = self.log_dir.join(format!("{}.jsonl", date));
        
        // Append event to log file
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .await?;
        
        let json = serde_json::to_string(&event)?;
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        
        Ok(())
    }
    
    /// Clean up old log files (older than retention_days)
    pub async fn cleanup(&self) -> Result<()> {
        let cutoff = Local::now() - chrono::Duration::days(self.retention_days as i64);
        
        let mut entries = fs::read_dir(&self.log_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time: DateTime<Local> = modified.into();
                        if modified_time < cutoff {
                            let _ = fs::remove_file(path).await;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}