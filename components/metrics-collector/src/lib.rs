use ftl_sdk::{tools, ToolResponse};
use schemars::JsonSchema;
use serde::Deserialize;

mod aggregator;
mod collector;
mod emission;
mod exporters;

use aggregator::MetricEvent;
use collector::MetricsCollector;

#[derive(Deserialize, JsonSchema)]
struct EventInput {
    /// Metric event data from gateway
    event: MetricEvent,
}

tools! {
    /// Receive metric event from gateway
    async fn events(input: EventInput) -> ToolResponse {
        let collector = MetricsCollector::instance();
        collector.record_event(input.event).await;
        ToolResponse::text("Event recorded")
    }



}

#[cfg(test)]
mod tests {
    use crate::emission::{EmissionConfig, EmissionPipeline};
    use crate::exporters::durable::DurableEmitter;
    use crate::exporters::fallback::FallbackEmitter;
    use crate::exporters::otel::{OtelEmitter, OtelEmitterConfig};

    #[test]
    fn test_emission_pipeline_creation() {
        let config = EmissionConfig::default();
        let mut pipeline = EmissionPipeline::new(config);

        // Add all emitters
        let otel_emitter = OtelEmitter::new(OtelEmitterConfig::default());
        pipeline.add_emitter(Box::new(otel_emitter));

        let durable_emitter = DurableEmitter::new();
        pipeline.add_emitter(Box::new(durable_emitter));

        let fallback_emitter = FallbackEmitter::new();
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
        pipeline.add_emitter(Box::new(DurableEmitter::new()));

        // Test that emitters were added
        assert_eq!(pipeline.emitters.len(), 2);

        // Test emitter names
        assert_eq!(pipeline.emitters[0].name(), "otel");
        assert_eq!(pipeline.emitters[1].name(), "durable");

        // WASM-compatible: Test creation only, async emission tested in integration
        // Note: Async emission testing would require async runtime in WASM context
    }

    #[test]
    fn test_fallback_emitter_creation() {
        // Simple test to verify fallback emitter can be created
        let fallback_emitter = FallbackEmitter::new();
        assert_eq!(fallback_emitter.name(), "fallback");
    }
}
