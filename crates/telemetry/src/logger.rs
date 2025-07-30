//! Local telemetry logging

#[cfg(test)]
mod tests;

use crate::{config::TelemetryConfig, events::TelemetryEvent};
use anyhow::Result;
use chrono::{DateTime, Local};
use fs4::fs_std::FileExt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tokio::fs;
use tokio::task;

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
    
    /// Log an event to a local file with file locking
    pub async fn log(&self, event: TelemetryEvent) -> Result<()> {
        // Create log directory if it doesn't exist
        fs::create_dir_all(&self.log_dir).await?;
        
        // Create daily log file
        let date = Local::now().format("%Y-%m-%d");
        let log_file = self.log_dir.join(format!("{}.jsonl", date));
        
        // Serialize event before entering blocking section
        let json = serde_json::to_string(&event)?;
        let mut json_with_newline = json;
        json_with_newline.push('\n');
        
        // Use blocking task for file locking operations
        task::spawn_blocking(move || -> Result<()> {
            // Open file with exclusive lock
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)?;
            
            // Lock the file exclusively
            file.lock_exclusive()?;
            
            // Write the event
            file.write_all(json_with_newline.as_bytes())?;
            file.flush()?;
            
            // Unlock happens automatically when file is dropped
            Ok(())
        })
        .await??;
        
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
                            if let Err(e) = fs::remove_file(&path).await {
                                tracing::debug!("Failed to remove old telemetry log {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}