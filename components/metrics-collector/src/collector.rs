use crate::aggregator::{MetricEvent, MetricsAggregator};
use crate::emission::{EmissionConfig, EmissionPipeline};
use crate::exporters::durable::DurableEmitter;
use crate::exporters::fallback::FallbackEmitter;
use crate::exporters::otel::{OtelEmitter, OtelEmitterConfig};
use serde_json::Value;
use std::sync::OnceLock;

static COLLECTOR: OnceLock<MetricsCollector> = OnceLock::new();

pub struct MetricsCollector {
    aggregator: MetricsAggregator,
    emission_pipeline: EmissionPipeline,
}

impl MetricsCollector {
    pub fn instance() -> &'static Self {
        COLLECTOR.get_or_init(|| {
            // Create emission pipeline with configured emitters
            let emission_config = EmissionConfig::default();
            let mut pipeline = EmissionPipeline::new(emission_config.clone());

            // Add OTEL emitter if enabled
            if emission_config.otel_enabled {
                let otel_config = OtelEmitterConfig::default();
                let otel_emitter = OtelEmitter::new(otel_config);
                pipeline.add_emitter(Box::new(otel_emitter));
            }

            // Add Durable emitter if enabled
            if emission_config.durable_enabled {
                let durable_emitter = DurableEmitter::new();
                pipeline.add_emitter(Box::new(durable_emitter));
            }

            // Add Fallback emitter if enabled
            if emission_config.fallback_enabled {
                let fallback_emitter = FallbackEmitter::new();
                pipeline.add_emitter(Box::new(fallback_emitter));
            }

            Self {
                aggregator: MetricsAggregator::new(10_000), // Max 10k metrics
                emission_pipeline: pipeline,
            }
        })
    }

    pub async fn record_event(&self, event: MetricEvent) {
        // Emit to external systems (OTEL + Durable) first
        let emission_results = self.emission_pipeline.emit_event(&event).await;

        // Log emission results for debugging
        for (emitter_name, result) in emission_results {
            match result {
                crate::emission::EmissionResult::Success => {
                    // Silent success
                }
                crate::emission::EmissionResult::Failed(err) => {
                    eprintln!("Emission failed for {}: {}", emitter_name, err);
                }
                crate::emission::EmissionResult::Fallback(reason) => {
                    eprintln!("Emission fallback for {}: {}", emitter_name, reason);
                }
            }
        }

        // Continue with local aggregation for pull-based metrics
        // In WASM, async works natively without any special executors
        self.aggregator.record_event(event).await;
    }

    pub async fn get_all_metrics_json(&self) -> Value {
        // Get metrics asynchronously
        let metrics = self.aggregator.get_all_metrics().await;
        serde_json::to_value(metrics).unwrap_or(serde_json::json!({}))
    }
}
