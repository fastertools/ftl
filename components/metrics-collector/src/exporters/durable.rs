use crate::aggregator::MetricEvent;
use crate::emission::{EmissionResult, MetricsEmitter};
use serde::{Deserialize, Serialize};

/// CloudTrail-like platform event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEvent {
    /// Event timestamp
    pub event_time: u64,
    /// Service that emitted the event (e.g., "ftl-cli", "ftl-gateway")
    pub event_source: String,
    /// Event name (e.g., "ToolInvoked", "MetricCollected")
    pub event_name: String,
    /// User identity (if available)
    pub user_identity: Option<String>,
    /// Source IP (if available)
    pub source_ip_address: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Event payload
    pub event_payload: serde_json::Value,
}

impl PlatformEvent {
    pub fn from_metric_event(event: &MetricEvent) -> Self {
        let event_payload = serde_json::json!({
            "tool_name": event.tool_name,
            "duration_ms": event.duration_ms,
            "success": event.success,
            "metadata": event.metadata
        });

        Self {
            event_time: event.timestamp,
            event_source: event.component_name.clone(),
            event_name: event
                .metadata
                .get("event_name")
                .cloned()
                .unwrap_or_else(|| "ToolInvoked".to_string()),
            user_identity: event.metadata.get("user_id").cloned(),
            source_ip_address: event.metadata.get("source_ip").cloned(),
            request_id: event.metadata.get("request_id").cloned(),
            event_payload,
        }
    }
}

/// Simple durable emitter that outputs to console for now
pub struct DurableEmitter;

impl DurableEmitter {
    pub fn new() -> Self {
        Self
    }

    async fn emit_to_console(&self, platform_event: &PlatformEvent) -> EmissionResult {
        println!(
            "PLATFORM_EVENT: {}",
            serde_json::to_string_pretty(platform_event)
                .unwrap_or_else(|_| "Failed to serialize event".to_string())
        );
        EmissionResult::Success
    }
}

impl MetricsEmitter for DurableEmitter {
    fn emit_event(
        &self,
        event: MetricEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmissionResult> + Send + '_>> {
        Box::pin(async move {
            let platform_event = PlatformEvent::from_metric_event(&event);
            self.emit_to_console(&platform_event).await
        })
    }

    fn name(&self) -> &'static str {
        "durable"
    }

    fn health_check(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move { true })
    }
}
