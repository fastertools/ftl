//! Telemetry event definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Event ID
    pub id: String,
    
    /// Event type
    pub event_type: EventType,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Session ID
    pub session_id: String,
    
    /// Event properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Types of telemetry events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// CLI command executed
    CommandExecuted,
    
    /// Command completed successfully
    CommandSuccess,
    
    /// Command failed
    CommandError,
    
    /// Feature used
    FeatureUsed,
    
    /// Performance metric
    PerformanceMetric,
}

impl TelemetryEvent {
    /// Create a new telemetry event
    pub fn new(event_type: EventType, session_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            session_id,
            properties: HashMap::new(),
        }
    }
    
    /// Add a property to the event
    pub fn with_property<T: Serialize>(mut self, key: &str, value: T) -> Self {
        self.properties.insert(
            key.to_string(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Create a command executed event
    pub fn command_executed(command: &str, args: Vec<String>, session_id: String) -> Self {
        Self::new(EventType::CommandExecuted, session_id)
            .with_property("command", command)
            .with_property("args", args)
    }
    
    /// Create a command success event
    pub fn command_success(command: &str, duration_ms: u64, session_id: String) -> Self {
        Self::new(EventType::CommandSuccess, session_id)
            .with_property("command", command)
            .with_property("duration_ms", duration_ms)
    }
    
    /// Create a command error event
    pub fn command_error(command: &str, error: &str, session_id: String) -> Self {
        Self::new(EventType::CommandError, session_id)
            .with_property("command", command)
            .with_property("error", error)
    }
}