use crate::aggregator::MetricEvent;
use crate::emission::{EmissionResult, MetricsEmitter};

/// Simple fallback emitter that stores to backup storage when primary emitters fail
pub struct FallbackEmitter;

impl FallbackEmitter {
    pub fn new() -> Self {
        Self
    }

    /// Store event to backup storage (postgres/KV) - stubbed for now
    async fn store_to_backup(&self, event: &MetricEvent) -> EmissionResult {
        // STUB: Future implementation will write to postgres/KV backup store
        // For now, just indicate that fallback would have been triggered
        let _ = event; // Acknowledge the payload
        EmissionResult::Success
    }
}

impl MetricsEmitter for FallbackEmitter {
    fn emit_event(&self, event: MetricEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmissionResult> + Send + '_>> {
        Box::pin(async move {
            self.store_to_backup(&event).await
        })
    }
    
    fn name(&self) -> &'static str {
        "fallback"
    }
    
    fn health_check(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move {
            true
        })
    }
}