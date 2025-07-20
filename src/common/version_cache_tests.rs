//! Unit tests for the version cache

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;

use crate::common::version_cache::*;
use crate::deps::*;
use crate::ui::TestUserInterface;
use crate::test_helpers::*;

// Mock implementations
struct MockHttpClient {
    response: Option<String>,
    should_fail: bool,
}

impl MockHttpClient {
    fn new() -> Self {
        Self { 
            response: Some(r#"{"crate": {"newest_version": "0.2.0"}}"#.to_string()),
            should_fail: false,
        }
    }
    
    fn with_response(mut self, response: String) -> Self {
        self.response = Some(response);
        self
    }
    
    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, _url: &str, _user_agent: &str) -> Result<String, anyhow::Error> {
        if self.should_fail {
            Err(anyhow::anyhow!("Network error"))
        } else {
            Ok(self.response.clone().unwrap_or_else(|| {
                r#"{"crate": {"newest_version": "0.2.0"}}"#.to_string()
            }))
        }
    }
}

struct MockEnvironment {
    vars: std::collections::HashMap<String, String>,
    home_dir: Option<PathBuf>,
    cargo_pkg_version: &'static str,
    unix_timestamp: u64,
}

impl MockEnvironment {
    fn new() -> Self {
        Self {
            vars: std::collections::HashMap::new(),
            home_dir: Some(PathBuf::from("/home/user")),
            cargo_pkg_version: "0.1.0",
            unix_timestamp: 1000000,
        }
    }
    
    fn with_var(mut self, key: &str, value: &str) -> Self {
        self.vars.insert(key.to_string(), value.to_string());
        self
    }
    
    fn with_no_home(mut self) -> Self {
        self.home_dir = None;
        self
    }
    
    fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.unix_timestamp = timestamp;
        self
    }
}

impl Environment for MockEnvironment {
    fn get_var(&self, key: &str) -> Result<String, std::env::VarError> {
        self.vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }
    
    fn get_home_dir(&self) -> Option<PathBuf> {
        self.home_dir.clone()
    }
    
    fn get_cargo_pkg_version(&self) -> &'static str {
        self.cargo_pkg_version
    }
    
    fn get_unix_timestamp(&self) -> u64 {
        self.unix_timestamp
    }
}

struct MockUpdateExecutor {
    should_fail: bool,
}

impl MockUpdateExecutor {
    fn new() -> Self {
        Self { should_fail: false }
    }
    
    fn with_failure() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait::async_trait]
impl UpdateExecutor for MockUpdateExecutor {
    async fn execute(&self, _sudo: bool) -> Result<(), anyhow::Error> {
        if self.should_fail {
            Err(anyhow::anyhow!("Update failed"))
        } else {
            Ok(())
        }
    }
}

// Clock trait is no longer used in version_cache, removed MockClock

struct TestFixture {
    file_system: MockFileSystemMock,
    http_client: Arc<MockHttpClient>,
    environment: Arc<MockEnvironment>,
    ui: Arc<dyn UserInterface>,
    update_executor: Arc<MockUpdateExecutor>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            file_system: MockFileSystemMock::new(),
            http_client: Arc::new(MockHttpClient::new()),
            environment: Arc::new(MockEnvironment::new()),
            ui: Arc::new(TestUserInterface::new()) as Arc<dyn UserInterface>,
            update_executor: Arc::new(MockUpdateExecutor::new()),
        }
    }
    
    fn to_deps(self) -> Arc<VersionCacheDependencies> {
        Arc::new(VersionCacheDependencies {
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            http_client: self.http_client as Arc<dyn HttpClient>,
            environment: self.environment as Arc<dyn Environment>,
            ui: self.ui as Arc<dyn UserInterface>,
            update_executor: self.update_executor as Arc<dyn UpdateExecutor>,
        })
    }
}

#[test]
fn test_get_cache_dir_with_xdg() {
    let mut fixture = TestFixture::new();
    fixture.environment = Arc::new(
        MockEnvironment::new().with_var("XDG_CACHE_HOME", "/custom/cache")
    );
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let cache_dir = manager.get_cache_dir().unwrap();
    assert_eq!(cache_dir, PathBuf::from("/custom/cache/ftl"));
}

#[test]
fn test_get_cache_dir_default() {
    let fixture = TestFixture::new();
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let cache_dir = manager.get_cache_dir().unwrap();
    assert_eq!(cache_dir, PathBuf::from("/home/user/.cache/ftl"));
}

#[test]
fn test_get_cache_dir_no_home() {
    let mut fixture = TestFixture::new();
    fixture.environment = Arc::new(MockEnvironment::new().with_no_home());
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let result = manager.get_cache_dir();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Could not determine home directory"));
}

#[test]
fn test_load_version_cache_not_exists() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache file doesn't exist
    fixture.file_system
        .expect_exists()
        .returning(|_| false);
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let cache = manager.load_version_cache().unwrap();
    assert_eq!(cache.current_version, "0.1.0");
    assert_eq!(cache.last_check_timestamp, 0);
}

#[test]
fn test_load_version_cache_exists() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache file exists
    fixture.file_system
        .expect_exists()
        .returning(|_| true);
    
    fixture.file_system
        .expect_read_to_string()
        .returning(|_| Ok(r#"{
            "last_check_timestamp": 999999,
            "current_version": "0.1.0",
            "latest_version": "0.2.0",
            "dismissed_version": null
        }"#.to_string()));
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let cache = manager.load_version_cache().unwrap();
    assert_eq!(cache.last_check_timestamp, 999999);
    assert_eq!(cache.current_version, "0.1.0");
    assert_eq!(cache.latest_version, Some("0.2.0".to_string()));
}

#[test]
fn test_load_version_cache_invalid_json() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache file exists with invalid JSON
    fixture.file_system
        .expect_exists()
        .returning(|_| true);
    
    fixture.file_system
        .expect_read_to_string()
        .returning(|_| Ok("invalid json".to_string()));
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let result = manager.load_version_cache();
    assert!(result.is_err());
}

#[test]
fn test_save_version_cache() {
    let mut fixture = TestFixture::new();
    
    // Mock: write succeeds
    fixture.file_system
        .expect_write_string()
        .withf(|path: &Path, content: &str| {
            path == Path::new("/home/user/.cache/ftl/version_cache.json") &&
            content.contains("\"current_version\": \"0.1.0\"")
        })
        .times(1)
        .returning(|_, _| Ok(()));
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let cache = VersionCache::new("0.1.0".to_string());
    let result = manager.save_version_cache(&cache);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fetch_latest_version_success() {
    let mut fixture = TestFixture::new();
    fixture.http_client = Arc::new(
        MockHttpClient::new().with_response(
            r#"{"crate": {"newest_version": "0.3.0"}}"#.to_string()
        )
    );
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let version = manager.fetch_latest_version().await.unwrap();
    assert_eq!(version, "0.3.0");
}

#[tokio::test]
async fn test_fetch_latest_version_invalid_json() {
    let mut fixture = TestFixture::new();
    fixture.http_client = Arc::new(
        MockHttpClient::new().with_response("invalid json".to_string())
    );
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let result = manager.fetch_latest_version().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_check_and_prompt_no_check_needed() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache exists with recent check
    fixture.file_system
        .expect_exists()
        .returning(|_| true);
    
    fixture.file_system
        .expect_read_to_string()
        .returning(|_| Ok(r#"{
            "last_check_timestamp": 999000,
            "current_version": "0.1.0",
            "latest_version": "0.1.0",
            "dismissed_version": null
        }"#.to_string()));
    
    // Set timestamp to only 1 hour after last check
    fixture.environment = Arc::new(MockEnvironment::new().with_timestamp(999000 + 3600));
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let result = manager.check_and_prompt_for_update().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_check_and_prompt_with_update_available() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache doesn't exist (first run)
    fixture.file_system
        .expect_exists()
        .times(1)
        .returning(|_| false);
    
    // Mock: save cache after update check
    fixture.file_system
        .expect_write_string()
        .times(1)
        .returning(|_, _| Ok(()));
    
    // Mock: user chooses not to update and not to dismiss
    fixture.ui = Arc::new(TestUserInterface::new());
    
    // Timestamp is already set to 1000000 in new()
    
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let result = manager.check_and_prompt_for_update().await;
    assert!(result.is_ok());
    
    // The fixture UI would have shown the update prompt
    // We just verify the function completes without error
}

#[tokio::test]
async fn test_check_and_prompt_with_dismissed_version() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache exists with dismissed version
    fixture.file_system
        .expect_exists()
        .returning(|_| true);
    
    fixture.file_system
        .expect_read_to_string()
        .returning(|_| Ok(r#"{
            "last_check_timestamp": 0,
            "current_version": "0.1.0",
            "latest_version": "0.2.0",
            "dismissed_version": "0.2.0"
        }"#.to_string()));
    
    // Mock: save cache after check
    fixture.file_system
        .expect_write_string()
        .times(1)
        .returning(|_, _| Ok(()));
    
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let result = manager.check_and_prompt_for_update().await;
    assert!(result.is_ok());
    
    // Since the version was dismissed, no update prompt should have been shown
    // We just verify the function completes without error
}

#[tokio::test]
async fn test_check_and_prompt_network_failure() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache doesn't exist
    fixture.file_system
        .expect_exists()
        .times(1)
        .returning(|_| false);
    
    // Mock: network request fails
    fixture.http_client = Arc::new(MockHttpClient::new().with_failure());
    
    // Mock: save cache even on failure
    fixture.file_system
        .expect_write_string()
        .times(1)
        .returning(|_, _| Ok(()));
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    // Should not fail even if network request fails
    let result = manager.check_and_prompt_for_update().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_check_and_prompt_user_chooses_update() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache doesn't exist
    fixture.file_system
        .expect_exists()
        .times(1)
        .returning(|_| false);
    
    // Mock: save cache after update check
    fixture.file_system
        .expect_write_string()
        .times(1)
        .returning(|_, _| Ok(()));
    
    // Create a custom TestUserInterface that returns "Yes" for update prompt
    struct TestUIWithUpdateChoice {
        inner: TestUserInterface,
        prompt_count: Arc<Mutex<usize>>,
    }
    
    impl UserInterface for TestUIWithUpdateChoice {
        fn create_spinner(&self) -> Box<dyn crate::deps::ProgressIndicator> {
            self.inner.create_spinner()
        }
        
        fn create_multi_progress(&self) -> Box<dyn crate::deps::MultiProgressManager> {
            self.inner.create_multi_progress()
        }
        
        fn print(&self, message: &str) {
            self.inner.print(message);
        }
        
        fn print_styled(&self, message: &str, style: crate::deps::MessageStyle) {
            self.inner.print_styled(message, style);
        }
        
        fn is_interactive(&self) -> bool {
            self.inner.is_interactive()
        }
        
        fn prompt_input(&self, prompt: &str, default: Option<&str>) -> Result<String, anyhow::Error> {
            self.inner.prompt_input(prompt, default)
        }
        
        fn prompt_select(&self, prompt: &str, _items: &[&str], _default: usize) -> Result<usize, anyhow::Error> {
            let mut count = self.prompt_count.lock().unwrap();
            *count += 1;
            
            if prompt.contains("Would you like to update now?") {
                Ok(0) // Select "Yes"
            } else {
                Ok(1) // Select "No" for other prompts
            }
        }
        
        fn clear_screen(&self) {
            self.inner.clear_screen();
        }
    }
    
    let test_ui = Arc::new(TestUIWithUpdateChoice {
        inner: TestUserInterface::new(),
        prompt_count: Arc::new(Mutex::new(0)),
    });
    
    fixture.ui = test_ui.clone() as Arc<dyn UserInterface>;
    fixture.update_executor = Arc::new(MockUpdateExecutor::new()); // Should succeed
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let result = manager.check_and_prompt_for_update().await;
    assert!(result.is_ok());
    
    // Verify update was attempted
    let output = test_ui.inner.get_output();
    assert!(output.iter().any(|s| s.contains("A new version of FTL CLI is available")));
}

#[tokio::test]
async fn test_check_and_prompt_user_dismisses_version() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache doesn't exist
    fixture.file_system
        .expect_exists()
        .times(1)
        .returning(|_| false);
    
    // Mock: save cache twice - once after check, once after dismiss
    fixture.file_system
        .expect_write_string()
        .times(2)
        .returning(|_, _| Ok(()));
    
    // Create a custom TestUserInterface for this test
    struct TestUIWithDismissChoice {
        inner: TestUserInterface,
        prompt_count: Arc<Mutex<usize>>,
    }
    
    impl UserInterface for TestUIWithDismissChoice {
        fn create_spinner(&self) -> Box<dyn crate::deps::ProgressIndicator> {
            self.inner.create_spinner()
        }
        
        fn create_multi_progress(&self) -> Box<dyn crate::deps::MultiProgressManager> {
            self.inner.create_multi_progress()
        }
        
        fn print(&self, message: &str) {
            self.inner.print(message);
        }
        
        fn print_styled(&self, message: &str, style: crate::deps::MessageStyle) {
            self.inner.print_styled(message, style);
        }
        
        fn is_interactive(&self) -> bool {
            self.inner.is_interactive()
        }
        
        fn prompt_input(&self, prompt: &str, default: Option<&str>) -> Result<String, anyhow::Error> {
            self.inner.prompt_input(prompt, default)
        }
        
        fn prompt_select(&self, prompt: &str, _items: &[&str], _default: usize) -> Result<usize, anyhow::Error> {
            let mut count = self.prompt_count.lock().unwrap();
            *count += 1;
            
            if prompt.contains("Would you like to update now?") {
                Ok(1) // Select "No"
            } else if prompt.contains("Don't remind me about this version again?") {
                Ok(0) // Select "Yes" to dismiss
            } else {
                Ok(1)
            }
        }
        
        fn clear_screen(&self) {
            self.inner.clear_screen();
        }
    }
    
    let test_ui = Arc::new(TestUIWithDismissChoice {
        inner: TestUserInterface::new(),
        prompt_count: Arc::new(Mutex::new(0)),
    });
    
    fixture.ui = test_ui.clone() as Arc<dyn UserInterface>;
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let result = manager.check_and_prompt_for_update().await;
    assert!(result.is_ok());
    
    // Verify version was shown and dismiss was offered
    let output = test_ui.inner.get_output();
    assert!(output.iter().any(|s| s.contains("A new version of FTL CLI is available")));
}

#[tokio::test]
async fn test_update_executor_failure() {
    let mut fixture = TestFixture::new();
    
    // Mock: cache doesn't exist
    fixture.file_system
        .expect_exists()
        .times(1)
        .returning(|_| false);
    
    // Mock: save cache
    fixture.file_system
        .expect_write_string()
        .times(1)
        .returning(|_, _| Ok(()));
    
    // Create UI that chooses to update
    struct TestUIUpdateFails {
        inner: TestUserInterface,
    }
    
    impl UserInterface for TestUIUpdateFails {
        fn create_spinner(&self) -> Box<dyn crate::deps::ProgressIndicator> {
            self.inner.create_spinner()
        }
        
        fn create_multi_progress(&self) -> Box<dyn crate::deps::MultiProgressManager> {
            self.inner.create_multi_progress()
        }
        
        fn print(&self, message: &str) {
            self.inner.print(message);
        }
        
        fn print_styled(&self, message: &str, style: crate::deps::MessageStyle) {
            self.inner.print_styled(message, style);
        }
        
        fn is_interactive(&self) -> bool {
            self.inner.is_interactive()
        }
        
        fn prompt_input(&self, prompt: &str, default: Option<&str>) -> Result<String, anyhow::Error> {
            self.inner.prompt_input(prompt, default)
        }
        
        fn prompt_select(&self, _prompt: &str, _items: &[&str], _default: usize) -> Result<usize, anyhow::Error> {
            Ok(0) // Always select first option ("Yes" to update)
        }
        
        fn clear_screen(&self) {
            self.inner.clear_screen();
        }
    }
    
    let test_ui = Arc::new(TestUIUpdateFails {
        inner: TestUserInterface::new(),
    });
    
    fixture.ui = test_ui as Arc<dyn UserInterface>;
    fixture.update_executor = Arc::new(MockUpdateExecutor::with_failure());
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    // Should fail because update executor fails
    let result = manager.check_and_prompt_for_update().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Update failed"));
}

#[test]
fn test_version_cache_update_check() {
    let mut cache = VersionCache::new("0.1.0".to_string());
    
    cache.update_check(2000000, "0.1.1".to_string(), Some("0.2.0".to_string()));
    
    assert_eq!(cache.last_check_timestamp, 2000000);
    assert_eq!(cache.current_version, "0.1.1");
    assert_eq!(cache.latest_version, Some("0.2.0".to_string()));
}

#[test]
fn test_version_cache_dismiss_version() {
    let mut cache = VersionCache::new("0.1.0".to_string());
    cache.latest_version = Some("0.2.0".to_string());
    
    // Should prompt before dismissal
    assert!(cache.should_prompt_for_update());
    
    // Dismiss the version
    cache.dismiss_version("0.2.0".to_string());
    
    // Should not prompt after dismissal
    assert!(!cache.should_prompt_for_update());
}

#[test]
fn test_save_version_cache_write_failure() {
    let mut fixture = TestFixture::new();
    
    // Mock: write fails
    fixture.file_system
        .expect_write_string()
        .times(1)
        .returning(|_, _| Err(anyhow::anyhow!("Permission denied")));
    
    let deps = fixture.to_deps();
    let manager = VersionCacheManager::new(deps);
    
    let cache = VersionCache::new("0.1.0".to_string());
    let result = manager.save_version_cache(&cache);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to write version cache"));
}

#[test]
fn test_version_cache_with_invalid_semver() {
    let mut cache = VersionCache::new("not-a-version".to_string());
    cache.latest_version = Some("also-not-a-version".to_string());
    
    // Should not prompt for update when versions can't be parsed
    assert!(!cache.should_prompt_for_update());
}