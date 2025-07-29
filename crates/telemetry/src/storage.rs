//! Telemetry storage utilities

use anyhow::Result;
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Get the size of telemetry logs in bytes
pub async fn get_log_size(log_dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    
    if !log_dir.exists() {
        return Ok(0);
    }
    
    let mut entries = fs::read_dir(log_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if let Ok(metadata) = entry.metadata().await {
            total_size += metadata.len();
        }
    }
    
    Ok(total_size)
}

/// List log files with their metadata
pub async fn list_log_files(log_dir: &Path) -> Result<Vec<LogFileInfo>> {
    let mut files = Vec::new();
    
    if !log_dir.exists() {
        return Ok(files);
    }
    
    let mut entries = fs::read_dir(log_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            if let Ok(metadata) = entry.metadata().await {
                files.push(LogFileInfo {
                    path,
                    size: metadata.len(),
                    modified: metadata.modified()?.into(),
                });
            }
        }
    }
    
    files.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(files)
}

/// Information about a log file
#[derive(Debug)]
pub struct LogFileInfo {
    /// File path
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Last modified time
    pub modified: DateTime<Local>,
}