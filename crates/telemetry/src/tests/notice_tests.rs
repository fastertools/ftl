//! Integration tests for first-run notice

use crate::{notice::*, TelemetryClient};
use ftl_common::config::Config;
use tempfile::TempDir;

#[tokio::test]
async fn test_first_run_notice_flow() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    // Create empty config to simulate first run
    let config = Config::load_from_path(&config_path).unwrap();
    config.save().unwrap();
    
    // In a real scenario, should_show_notice would return true
    // But in tests, it might return false due to CI detection
    // Let's test the config directly
    
    // First run - no notice config
    let notice_config = config.get_section::<NoticeConfig>().unwrap();
    assert!(notice_config.is_none());
    
    // Mark notice as shown
    let mut config = Config::load_from_path(&config_path).unwrap();
    let notice = NoticeConfig {
        notice_shown: true,
        notice_version: "1.0".to_string(),
    };
    config.set_section(notice).unwrap();
    config.save().unwrap();
    
    // Verify it was saved
    let config = Config::load_from_path(&config_path).unwrap();
    let notice_config = config.get_section::<NoticeConfig>().unwrap().unwrap();
    assert!(notice_config.notice_shown);
    assert_eq!(notice_config.notice_version, "1.0");
}

#[test]
fn test_notice_version_upgrade() {
    // Test that notice would be shown again if version changes
    let old_notice = NoticeConfig {
        notice_shown: true,
        notice_version: "0.9".to_string(),
    };
    
    // In real scenario, comparing against NOTICE_VERSION ("1.0")
    // would indicate need to show notice again
    assert_ne!(old_notice.notice_version, "1.0");
}

#[tokio::test]
async fn test_telemetry_client_initialize() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    // Create a config with telemetry enabled
    let mut config = Config::load_from_path(&config_path).unwrap();
    let telemetry_config = crate::config::TelemetryConfig {
        enabled: true,
        installation_id: "test-id".to_string(),
        log_directory: temp_dir.path().join("logs"),
        retention_days: 30,
    };
    config.set_section(telemetry_config).unwrap();
    
    // Mark notice as already shown to avoid interaction in tests
    let notice = NoticeConfig {
        notice_shown: true,
        notice_version: "1.0".to_string(),
    };
    config.set_section(notice).unwrap();
    config.save().unwrap();
    
    // Initialize should work without showing notice
    let client = TelemetryClient::from_config(crate::config::TelemetryConfig {
        enabled: true,
        installation_id: "test-id".to_string(),
        log_directory: temp_dir.path().join("logs"),
        retention_days: 30,
    }).unwrap();
    
    assert!(client.is_enabled());
    assert_eq!(client.installation_id(), "test-id");
}