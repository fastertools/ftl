use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::info;

/// Optimize a WASM binary using wasm-opt
pub fn optimize_wasm(wasm_path: &Path, optimization_flags: &[String]) -> Result<()> {
    // Check if wasm-opt is available
    if which::which("wasm-opt").is_err() {
        anyhow::bail!("wasm-opt not found. Install it with: cargo install wasm-opt");
    }
    
    let temp_path = wasm_path.with_extension("wasm.tmp");
    
    let mut cmd = Command::new("wasm-opt");
    cmd.arg(wasm_path)
        .arg("-o")
        .arg(&temp_path);
    
    // Add optimization flags
    for flag in optimization_flags {
        cmd.arg(flag);
    }
    
    info!("Optimizing WASM with flags: {:?}", optimization_flags);
    
    let output = cmd.output()
        .context("Failed to execute wasm-opt")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wasm-opt failed:\n{}", stderr);
    }
    
    // Replace original with optimized version
    std::fs::rename(&temp_path, wasm_path)
        .context("Failed to replace WASM with optimized version")?;
    
    Ok(())
}

/// Get the size of a file in bytes
pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for {}", path.display()))?;
    Ok(metadata.len())
}

/// Format file size in human-readable format
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_idx])
}