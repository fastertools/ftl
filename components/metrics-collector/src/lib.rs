use ftl_sdk::{tools, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

mod collector;
mod aggregator;
mod exporters;
mod emission;

use collector::MetricsCollector;
use aggregator::MetricEvent;

#[derive(Deserialize, JsonSchema)]
struct EventInput {
    /// Metric event data from gateway
    event: MetricEvent,
}

#[derive(Deserialize, JsonSchema)]
struct MetricsInput {
    /// Optional format (prometheus or json)
    format: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct ToolMetricsInput {
    /// Name of the tool to get metrics for
    tool_name: String,
}

tools! {
    /// Receive metric event from gateway
    async fn events(input: EventInput) -> ToolResponse {
        let collector = MetricsCollector::instance();
        collector.record_event(input.event).await;
        ToolResponse::text("Event recorded")
    }

    /// Get all metrics in specified format
    async fn metrics(input: MetricsInput) -> ToolResponse {
        let collector = MetricsCollector::instance();
        match input.format.as_deref() {
            Some("json") => {
                let metrics = collector.get_all_metrics_json().await;
                ToolResponse::with_structured(
                    serde_json::to_string_pretty(&metrics).unwrap_or_default(),
                    metrics
                )
            }
            _ => {
                let prometheus = collector.get_prometheus_metrics().await;
                ToolResponse::text(prometheus)
            }
        }
    }

    /// Get metrics for a specific tool
    async fn tool_metrics(input: ToolMetricsInput) -> ToolResponse {
        let collector = MetricsCollector::instance();
        match collector.get_tool_metrics(&input.tool_name).await {
            Some(metrics) => ToolResponse::with_structured(
                serde_json::to_string_pretty(&metrics).unwrap_or_default(),
                metrics
            ),
            None => ToolResponse::text(format!("No metrics found for tool: {}", input.tool_name))
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::emission::{EmissionPipeline, EmissionConfig};
    use crate::exporters::otel::{OtelEmitter, OtelEmitterConfig};
    use crate::exporters::durable::{DurableEmitter, DurableEmitterConfig, RetryConfig};
    use crate::exporters::fallback::{FallbackEmitter, FallbackEmitterConfig};

    #[test]
    fn test_emission_pipeline_creation() {
        let config = EmissionConfig::default();
        let mut pipeline = EmissionPipeline::new(config);
        
        // Add all emitters
        let otel_emitter = OtelEmitter::new(OtelEmitterConfig::default());
        pipeline.add_emitter(Box::new(otel_emitter));
        
        let durable_emitter = DurableEmitter::new(
            DurableEmitterConfig::default(), 
            RetryConfig::default()
        );
        pipeline.add_emitter(Box::new(durable_emitter));
        
        let fallback_emitter = FallbackEmitter::new(FallbackEmitterConfig::default());
        pipeline.add_emitter(Box::new(fallback_emitter));
        
        // Test pipeline creation
        assert_eq!(pipeline.emitters.len(), 3); // Should have 3 emitters
        
        // Test emitter names
        assert_eq!(pipeline.emitters[0].name(), "otel");
        assert_eq!(pipeline.emitters[1].name(), "durable");
        assert_eq!(pipeline.emitters[2].name(), "fallback");
    }

    #[test]
    fn test_dual_emission_flow() {
        let config = EmissionConfig::default();
        let mut pipeline = EmissionPipeline::new(config);
        
        // Add emitters
        pipeline.add_emitter(Box::new(OtelEmitter::new(OtelEmitterConfig::default())));
        pipeline.add_emitter(Box::new(DurableEmitter::new(
            DurableEmitterConfig::default(), 
            RetryConfig::default()
        )));
        
        // Test that emitters were added
        assert_eq!(pipeline.emitters.len(), 2);
        
        // Test emitter names
        assert_eq!(pipeline.emitters[0].name(), "otel");
        assert_eq!(pipeline.emitters[1].name(), "durable");
        
        // WASM-compatible: Test creation only, async emission tested in integration
        // Note: Async emission testing would require async runtime in WASM context
    }

    #[test]
    fn test_fallback_circuit_breaker() {
        use crate::exporters::fallback::CircuitBreaker;
        
        let circuit = CircuitBreaker::new(3, 60); // 3 failures, 60 second reset
        
        // Initially should allow requests
        assert!(circuit.should_allow_request());
        
        // Record failures
        circuit.record_failure();
        circuit.record_failure();
        assert!(circuit.should_allow_request()); // Still under threshold
        
        circuit.record_failure();
        assert!(!circuit.should_allow_request()); // Now circuit is open
        
        // Record success should reset
        circuit.record_success();
        assert!(circuit.should_allow_request()); // Circuit closed again
    }
}