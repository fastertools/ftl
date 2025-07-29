//! Tests for telemetry functionality

#[cfg(test)]
mod tests {
    use crate::{config::TelemetryConfig, events::*, TelemetryClient};
    use tempfile::TempDir;
    use ftl_common::config::Config;
    
    #[tokio::test]
    async fn test_telemetry_config_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut config = Config::load_from_path(&config_path).unwrap();
        let telemetry_config = TelemetryConfig::default();
        
        assert!(telemetry_config.enabled);
        assert!(!telemetry_config.installation_id.is_empty());
        
        // Save and reload
        config.set_section(telemetry_config.clone()).unwrap();
        config.save().unwrap();
        
        let loaded = Config::load_from_path(&config_path).unwrap();
        let loaded_telemetry = loaded.get_section::<TelemetryConfig>().unwrap().unwrap();
        assert_eq!(loaded_telemetry.installation_id, telemetry_config.installation_id);
    }
    
    #[tokio::test]
    async fn test_telemetry_config_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut config = Config::load_from_path(&config_path).unwrap();
        let mut telemetry_config = TelemetryConfig::default();
        telemetry_config.enabled = false;
        
        config.set_section(telemetry_config).unwrap();
        config.save().unwrap();
        
        // Load and verify
        let loaded_config = TelemetryConfig::load_from_path(&config_path).unwrap();
        assert!(!loaded_config.enabled);
        assert!(!loaded_config.is_enabled());
    }
    
    #[tokio::test]
    async fn test_event_creation() {
        let event = TelemetryEvent::command_executed(
            "build",
            vec!["--release".to_string()],
            "test-session".to_string(),
        );
        
        assert_eq!(event.event_type, EventType::CommandExecuted);
        assert_eq!(event.session_id, "test-session");
        assert_eq!(event.properties.get("command").unwrap(), "build");
    }
    
    #[tokio::test]
    async fn test_telemetry_client_with_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join(".ftl/logs");
        
        // Create config
        let mut config = Config::load_from_path(&config_path).unwrap();
        let telemetry_config = TelemetryConfig {
            enabled: true,
            installation_id: "test-id".to_string(),
            upload_enabled: false,
            log_directory: log_dir.clone(),
            retention_days: 7,
        };
        config.set_section(telemetry_config.clone()).unwrap();
        config.save().unwrap();
        
        // Create client with custom config
        let client = TelemetryClient::from_config(telemetry_config).unwrap();
        assert!(client.is_enabled());
        
        // Log an event
        let event = TelemetryEvent::command_executed(
            "test",
            vec![],
            "test-session".to_string(),
        );
        
        client.log_event(event).await.unwrap();
        
        // Verify log directory was created
        assert!(log_dir.exists());
    }
    
    #[test]
    fn test_event_types() {
        // Test command success event
        let success = TelemetryEvent::command_success("build", 1234, "session-1".to_string());
        assert_eq!(success.event_type, EventType::CommandSuccess);
        assert_eq!(success.properties.get("duration_ms").unwrap(), 1234);
        
        // Test command error event
        let error = TelemetryEvent::command_error("build", "compilation failed", "session-2".to_string());
        assert_eq!(error.event_type, EventType::CommandError);
        assert_eq!(error.properties.get("error").unwrap(), "compilation failed");
    }
}