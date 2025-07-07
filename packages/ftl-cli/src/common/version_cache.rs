use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use console::style;
use dialoguer::Confirm;
use semver::Version;
use serde::{Deserialize, Serialize};

const VERSION_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionCache {
    /// Timestamp of last version check (Unix timestamp)
    pub last_check_timestamp: u64,
    /// Current version when last checked
    pub current_version: String,
    /// Latest version found during last check
    pub latest_version: Option<String>,
    /// Version that user dismissed (won't prompt again for this version)
    pub dismissed_version: Option<String>,
}

impl Default for VersionCache {
    fn default() -> Self {
        Self {
            last_check_timestamp: 0,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            latest_version: None,
            dismissed_version: None,
        }
    }
}

impl VersionCache {
    /// Check if we should perform a version check today
    pub fn should_check_today(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check if it's been more than 24 hours since last check
        now - self.last_check_timestamp > 24 * 60 * 60
    }

    /// Check if there's a new version available that we should prompt about
    pub fn should_prompt_for_update(&self) -> bool {
        if let Some(latest) = &self.latest_version {
            // Don't prompt if user dismissed this version
            if let Some(dismissed) = &self.dismissed_version {
                if dismissed == latest {
                    return false;
                }
            }

            // Check if latest is newer than current
            if let (Ok(current), Ok(latest_ver)) = (
                Version::parse(&self.current_version),
                Version::parse(latest),
            ) {
                return latest_ver > current;
            }
        }
        false
    }

    /// Update the cache with new version information
    pub fn update_check(&mut self, latest_version: Option<String>) {
        self.last_check_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.current_version = env!("CARGO_PKG_VERSION").to_string();
        self.latest_version = latest_version;
    }

    /// Mark a version as dismissed by the user
    pub fn dismiss_version(&mut self, version: String) {
        self.dismissed_version = Some(version);
    }
}

/// Get the path to the FTL cache directory
pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg_cache)
    } else {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        home.join(".cache")
    };
    
    Ok(cache_dir.join("ftl"))
}

/// Get the path to the version cache file
pub fn get_version_cache_path() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("version_cache.json"))
}

/// Load version cache from disk
pub fn load_version_cache() -> Result<VersionCache> {
    let cache_path = get_version_cache_path()?;
    
    if !cache_path.exists() {
        // Create the cache directory if it doesn't exist
        let cache_dir = get_cache_dir()?;
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .with_context(|| format!("Failed to create cache directory {}", cache_dir.display()))?;
        }
        return Ok(VersionCache::default());
    }
    
    let content = fs::read_to_string(&cache_path)
        .with_context(|| format!("Failed to read version cache from {}", cache_path.display()))?;
    
    let cache: VersionCache = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse version cache from {}", cache_path.display()))?;
    
    Ok(cache)
}

/// Save version cache to disk
pub fn save_version_cache(cache: &VersionCache) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    
    // Create cache directory if it doesn't exist
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory {}", cache_dir.display()))?;
    }
    
    let cache_path = get_version_cache_path()?;
    let content = serde_json::to_string_pretty(cache)
        .context("Failed to serialize version cache")?;
    
    fs::write(&cache_path, content)
        .with_context(|| format!("Failed to write version cache to {}", cache_path.display()))?;
    
    Ok(())
}

/// Check for latest version from crates.io
pub async fn fetch_latest_version() -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(VERSION_CHECK_TIMEOUT)
        .build()?;
    
    let response = client
        .get("https://crates.io/api/v1/crates/ftl-cli")
        .header("User-Agent", format!("ftl-cli/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch crate information from crates.io");
    }

    let json: serde_json::Value = response.json().await?;
    
    let latest_version = json
        .get("crate")
        .and_then(|c| c.get("newest_version"))
        .and_then(|v| v.as_str())
        .context("Could not parse latest version from crates.io response")?;

    Ok(latest_version.to_string())
}

/// Perform version check and prompt user if needed
pub async fn check_and_prompt_for_update() -> Result<()> {
    let mut cache = load_version_cache().unwrap_or_default();
    
    // Only check if it's been more than 24 hours
    if !cache.should_check_today() {
        // Still check if we should prompt for a previously found update
        if cache.should_prompt_for_update() {
            prompt_for_update(&mut cache).await?;
        }
        return Ok(());
    }
    
    // Perform version check
    match fetch_latest_version().await {
        Ok(latest_version) => {
            cache.update_check(Some(latest_version));
            save_version_cache(&cache)?;
            
            // Prompt if there's a new version
            if cache.should_prompt_for_update() {
                prompt_for_update(&mut cache).await?;
            }
        }
        Err(_) => {
            // Silently fail version check - don't interrupt user workflow
            cache.update_check(None);
            let _ = save_version_cache(&cache);
        }
    }
    
    Ok(())
}

/// Prompt user about available update
async fn prompt_for_update(cache: &mut VersionCache) -> Result<()> {
    let latest = cache.latest_version.as_ref().unwrap();
    
    println!();
    println!("{} A new version of FTL CLI is available!", style("🎉").cyan());
    println!("  Current version: {}", style(&cache.current_version).dim());
    println!("  Latest version:  {}", style(latest).green());
    println!();
    
    let should_update = Confirm::new()
        .with_prompt("Would you like to update now?")
        .default(false)
        .interact()?;
    
    if should_update {
        println!();
        crate::commands::update::execute(false).await?;
    } else {
        // Ask if user wants to dismiss this version
        let should_dismiss = Confirm::new()
            .with_prompt("Don't remind me about this version again?")
            .default(false)
            .interact()?;
        
        if should_dismiss {
            cache.dismiss_version(latest.clone());
            save_version_cache(cache)?;
        }
    }
    
    println!();
    Ok(())
}