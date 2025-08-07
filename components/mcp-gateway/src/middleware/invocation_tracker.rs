use super::context::MiddlewareContext;
use super::types::{Middleware, MiddlewareError};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{self, Request};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    pub enabled: bool,
    pub collector_url: String,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collector_url: "http://ftl-metrics.spin.internal/events".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MetricEvent {
    pub timestamp: u64,
    pub tool_name: String,
    pub component_name: String,
    pub duration_ms: f64,
    pub success: bool,
    pub request_size: Option<usize>,
    pub metadata: HashMap<String, String>,
}

pub struct InvocationTracker {
    config: TrackerConfig,
}

impl InvocationTracker {
    pub fn new(config: TrackerConfig) -> Self {
        Self { config }
    }
}

impl Middleware for InvocationTracker {
    async fn pre_process(&self, ctx: &mut MiddlewareContext) -> Result<(), MiddlewareError> {
        eprintln!(
            "InvocationTracker::pre_process - tool: {}, component: {}",
            ctx.tool_name, ctx.component_name
        );
        Ok(())
    }

    async fn post_process(&self, ctx: &mut MiddlewareContext) -> Result<(), MiddlewareError> {
        eprintln!(
            "InvocationTracker::post_process - enabled: {}, tool: {}",
            self.config.enabled, ctx.tool_name
        );

        if !self.config.enabled {
            eprintln!("InvocationTracker disabled, skipping");
            return Ok(());
        }

        // Extract tenant/OIDC context from extensions if available
        let mut metadata = HashMap::new();

        // Check for tenant ID (would be set by upstream auth middleware)
        if let Some(tenant_id) = ctx.extensions.get::<String>() {
            metadata.insert("tenant_id".to_string(), tenant_id.clone());
        }

        // Check for authenticated user context from authorizer (NO PII)
        if let Some(user_id) = ctx.metadata.additional.get("user_id") {
            metadata.insert("user_id".to_string(), user_id.clone());
        }
        if let Some(auth_provider) = ctx.metadata.additional.get("auth_provider") {
            metadata.insert("auth_provider".to_string(), auth_provider.clone());
        }

        // Create metric event from context
        let event = MetricEvent {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            tool_name: ctx.tool_name.clone(),
            component_name: ctx.component_name.clone(),
            duration_ms: ctx.timing.tool_duration().unwrap_or_default().as_secs_f64() * 1000.0,
            success: ctx.is_success(),
            request_size: ctx.request_size(),
            metadata,
        };

        // Fire and forget - call ftl-metrics component directly
        // POST to /events tool with proper input format
        let event_input = serde_json::json!({
            "event": event
        });

        let req = Request::builder()
            .method(http::Method::Post)
            .uri(&self.config.collector_url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&event_input).unwrap_or_default())
            .build();

        // Fire and forget for sub-1ms performance
        let response_result: Result<spin_sdk::http::Response, _> = spin_sdk::http::send(req).await;
        match response_result {
            Ok(_response) => {
                // Metrics sent successfully
            }
            Err(_e) => {
                // Metrics collection failed, continuing normally
            }
        }

        Ok(())
    }

    async fn shutdown(&self) -> Result<(), MiddlewareError> {
        // Nothing to cleanup
        Ok(())
    }
}
