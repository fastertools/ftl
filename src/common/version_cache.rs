//! Refactored version cache with dependency injection for better testability

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::deps::{FileSystem, UserInterface, MessageStyle};

/// Helper function to check and prompt for update using default dependencies
pub async fn check_and_prompt_for_update() -> Result<()> {
    // Create real implementations
    struct RealHttpClient;
    #[async_trait::async_trait]
    impl HttpClient for RealHttpClient {
        async fn get(&self, url: &str, user_agent: &str) -> Result<String> {
            let response = reqwest::Client::new()
                .get(url)
                .header("User-Agent", user_agent)
                .send()
                .await?;
            
            response.text().await
                .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))
        }
    }
    
    struct RealEnvironment;
    impl Environment for RealEnvironment {
        fn get_var(&self, key: &str) -> Result<String, std::env::VarError> {
            std::env::var(key)
        }
        
        fn get_home_dir(&self) -> Option<PathBuf> {
            dirs::home_dir()
        }
        
        fn get_cargo_pkg_version(&self) -> &'static str {
            env!("CARGO_PKG_VERSION")
        }
        
        fn get_unix_timestamp(&self) -> u64 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    }
    
    struct RealUpdateExecutor;
    #[async_trait::async_trait]
    impl UpdateExecutor for RealUpdateExecutor {
        async fn execute(&self, _sudo: bool) -> Result<()> {
            use std::process::Command;
            
            let output = Command::new("cargo")
                .args(&["install", "ftl-cli", "--force"])
                .output()?;
                
            if !output.status.success() {
                anyhow::bail!("Failed to update: {}", String::from_utf8_lossy(&output.stderr));
            }
            
            Ok(())
        }
    }
    
    let ui = Arc::new(crate::ui::RealUserInterface);
    let deps = Arc::new(VersionCacheDependencies {
        file_system: Arc::new(crate::deps::RealFileSystem),
        http_client: Arc::new(RealHttpClient),
        environment: Arc::new(RealEnvironment),
        ui: ui.clone(),
        update_executor: Arc::new(RealUpdateExecutor),
    });
    
    let manager = VersionCacheManager::new(deps);
    manager.check_and_prompt_for_update().await
}

/// HTTP client trait for testability
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str, user_agent: &str) -> Result<String>;
}

/// Environment trait for testability
pub trait Environment: Send + Sync {
    fn get_var(&self, key: &str) -> Result<String, std::env::VarError>;
    fn get_home_dir(&self) -> Option<PathBuf>;
    fn get_cargo_pkg_version(&self) -> &'static str;
    fn get_unix_timestamp(&self) -> u64;
}

/// Update executor trait
#[async_trait::async_trait]
pub trait UpdateExecutor: Send + Sync {
    async fn execute(&self, sudo: bool) -> Result<()>;
}

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

impl VersionCache {
    pub fn new(current_version: String) -> Self {
        Self {
            last_check_timestamp: 0,
            current_version,
            latest_version: None,
            dismissed_version: None,
        }
    }

    /// Check if we should perform a version check today
    pub fn should_check_today(&self, now_secs: u64) -> bool {
        // Check if it's been more than 24 hours since last check
        now_secs - self.last_check_timestamp > 24 * 60 * 60
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
                semver::Version::parse(&self.current_version),
                semver::Version::parse(latest),
            ) {
                return latest_ver > current;
            }
        }
        false
    }

    /// Update the cache with new version information
    pub fn update_check(&mut self, now_secs: u64, current_version: String, latest_version: Option<String>) {
        self.last_check_timestamp = now_secs;
        self.current_version = current_version;
        self.latest_version = latest_version;
    }

    /// Mark a version as dismissed by the user
    pub fn dismiss_version(&mut self, version: String) {
        self.dismissed_version = Some(version);
    }
}

/// Dependencies for version cache operations
pub struct VersionCacheDependencies {
    pub file_system: Arc<dyn FileSystem>,
    pub http_client: Arc<dyn HttpClient>,
    pub environment: Arc<dyn Environment>,
    pub ui: Arc<dyn UserInterface>,
    pub update_executor: Arc<dyn UpdateExecutor>,
}

/// Version cache manager
pub struct VersionCacheManager {
    deps: Arc<VersionCacheDependencies>,
}

impl VersionCacheManager {
    pub fn new(deps: Arc<VersionCacheDependencies>) -> Self {
        Self { deps }
    }

    /// Get the path to the FTL cache directory
    pub fn get_cache_dir(&self) -> Result<PathBuf> {
        let cache_dir = if let Ok(xdg_cache) = self.deps.environment.get_var("XDG_CACHE_HOME") {
            PathBuf::from(xdg_cache)
        } else {
            let home = self.deps.environment.get_home_dir()
                .context("Could not determine home directory")?;
            home.join(".cache")
        };

        Ok(cache_dir.join("ftl"))
    }

    /// Get the path to the version cache file
    pub fn get_version_cache_path(&self) -> Result<PathBuf> {
        Ok(self.get_cache_dir()?.join("version_cache.json"))
    }

    /// Load version cache from disk
    pub fn load_version_cache(&self) -> Result<VersionCache> {
        let cache_path = self.get_version_cache_path()?;

        if !self.deps.file_system.exists(&cache_path) {
            return Ok(VersionCache::new(
                self.deps.environment.get_cargo_pkg_version().to_string()
            ));
        }

        let content = self.deps.file_system.read_to_string(&cache_path)
            .with_context(|| format!("Failed to read version cache from {}", cache_path.display()))?;

        let cache: VersionCache = serde_json::from_str(&content).with_context(|| {
            format!("Failed to parse version cache from {}", cache_path.display())
        })?;

        Ok(cache)
    }

    /// Save version cache to disk
    pub fn save_version_cache(&self, cache: &VersionCache) -> Result<()> {
        let cache_path = self.get_version_cache_path()?;
        let content = serde_json::to_string_pretty(cache)
            .context("Failed to serialize version cache")?;

        self.deps.file_system.write_string(&cache_path, &content)
            .with_context(|| format!("Failed to write version cache to {}", cache_path.display()))?;

        Ok(())
    }

    /// Check for latest version from crates.io
    pub async fn fetch_latest_version(&self) -> Result<String> {
        let user_agent = format!("ftl-cli/{}", self.deps.environment.get_cargo_pkg_version());
        
        let response = self.deps.http_client
            .get("https://crates.io/api/v1/crates/ftl-cli", &user_agent)
            .await?;

        let json: serde_json::Value = serde_json::from_str(&response)?;

        let latest_version = json
            .get("crate")
            .and_then(|c| c.get("newest_version"))
            .and_then(|v| v.as_str())
            .context("Could not parse latest version from crates.io response")?;

        Ok(latest_version.to_string())
    }

    /// Perform version check and prompt user if needed
    pub async fn check_and_prompt_for_update(&self) -> Result<()> {
        let mut cache = self.load_version_cache().unwrap_or_else(|_| {
            VersionCache::new(self.deps.environment.get_cargo_pkg_version().to_string())
        });

        let now_secs = self.deps.environment.get_unix_timestamp();

        // Only check if it's been more than 24 hours
        if !cache.should_check_today(now_secs) {
            // Still check if we should prompt for a previously found update
            if cache.should_prompt_for_update() {
                self.prompt_for_update(&mut cache).await?;
            }
            return Ok(());
        }

        // Perform version check
        match self.fetch_latest_version().await {
            Ok(latest_version) => {
                cache.update_check(
                    now_secs, 
                    self.deps.environment.get_cargo_pkg_version().to_string(),
                    Some(latest_version)
                );
                self.save_version_cache(&cache)?;

                // Prompt if there's a new version
                if cache.should_prompt_for_update() {
                    self.prompt_for_update(&mut cache).await?;
                }
            }
            Err(_) => {
                // Silently fail version check - don't interrupt user workflow
                cache.update_check(
                    now_secs,
                    self.deps.environment.get_cargo_pkg_version().to_string(),
                    None
                );
                let _ = self.save_version_cache(&cache);
            }
        }

        Ok(())
    }

    /// Prompt user about available update
    async fn prompt_for_update(&self, cache: &mut VersionCache) -> Result<()> {
        let latest = cache.latest_version.as_ref().unwrap();

        self.deps.ui.print("");
        self.deps.ui.print(&format!("🎉 A new version of FTL CLI is available!"));
        self.deps.ui.print(&format!("  Current version: {}", cache.current_version));
        self.deps.ui.print_styled(&format!("  Latest version:  {}", latest), MessageStyle::Green);
        self.deps.ui.print("");

        let should_update = self.deps.ui.prompt_select(
            "Would you like to update now?",
            &["Yes", "No"],
            1
        )? == 0;

        if should_update {
            self.deps.ui.print("");
            self.deps.update_executor.execute(false).await?;
        } else {
            // Ask if user wants to dismiss this version
            let should_dismiss = self.deps.ui.prompt_select(
                "Don't remind me about this version again?",
                &["Yes", "No"],
                1
            )? == 0;

            if should_dismiss {
                cache.dismiss_version(latest.clone());
                self.save_version_cache(cache)?;
            }
        }

        self.deps.ui.print("");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_cache_new() {
        let cache = VersionCache::new("0.1.0".to_string());
        assert_eq!(cache.current_version, "0.1.0");
        assert_eq!(cache.last_check_timestamp, 0);
        assert!(cache.latest_version.is_none());
        assert!(cache.dismissed_version.is_none());
    }

    #[test]
    fn test_should_check_today() {
        let cache = VersionCache::new("0.1.0".to_string());
        
        // Should check if never checked before
        assert!(cache.should_check_today(1000000));
        
        // Should not check if checked recently
        let mut cache = VersionCache::new("0.1.0".to_string());
        cache.last_check_timestamp = 1000000;
        assert!(!cache.should_check_today(1000000 + 3600)); // 1 hour later
        
        // Should check if more than 24 hours passed
        assert!(cache.should_check_today(1000000 + 25 * 3600)); // 25 hours later
    }

    #[test]
    fn test_should_prompt_for_update() {
        let mut cache = VersionCache::new("0.1.0".to_string());
        
        // No update available
        assert!(!cache.should_prompt_for_update());
        
        // Newer version available
        cache.latest_version = Some("0.2.0".to_string());
        assert!(cache.should_prompt_for_update());
        
        // Same version
        cache.latest_version = Some("0.1.0".to_string());
        assert!(!cache.should_prompt_for_update());
        
        // Older version (shouldn't happen but test anyway)
        cache.latest_version = Some("0.0.9".to_string());
        assert!(!cache.should_prompt_for_update());
        
        // Dismissed version
        cache.latest_version = Some("0.2.0".to_string());
        cache.dismissed_version = Some("0.2.0".to_string());
        assert!(!cache.should_prompt_for_update());
    }
}