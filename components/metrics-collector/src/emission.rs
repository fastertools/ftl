use crate::aggregator::MetricEvent;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// Result type for metric emission operations
#[derive(Debug, Clone, PartialEq)]
pub enum EmissionResult {
    Success,
    Failed(String),
    Fallback(String),
}

impl fmt::Display for EmissionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmissionResult::Success => write!(f, "Success"),
            EmissionResult::Failed(err) => write!(f, "Failed: {}", err),
            EmissionResult::Fallback(reason) => write!(f, "Fallback: {}", reason),
        }
    }
}

/// Configuration for emission behavior
#[derive(Debug, Clone)]
pub struct EmissionConfig {
    pub otel_enabled: bool,
    pub durable_enabled: bool,
    pub fallback_enabled: bool,
}

impl Default for EmissionConfig {
    fn default() -> Self {
        Self {
            otel_enabled: true,
            durable_enabled: true,
            fallback_enabled: true,
        }
    }
}

/// Trait for emitting metric events to external systems
pub trait MetricsEmitter: Send + Sync {
    /// Emit a metric event to the target system
    fn emit_event(&self, event: MetricEvent) -> Pin<Box<dyn Future<Output = EmissionResult> + Send + '_>>;
    
    /// Get the name/type of this emitter for logging
    fn name(&self) -> &'static str;
    
    /// Check if this emitter is healthy/available
    fn health_check(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async { true }) // Default implementation
    }
}

/// Pipeline that manages multiple metric emitters
pub struct EmissionPipeline {
    pub emitters: Vec<Box<dyn MetricsEmitter>>,
    config: EmissionConfig,
}

impl EmissionPipeline {
    pub fn new(config: EmissionConfig) -> Self {
        Self {
            emitters: Vec::new(),
            config,
        }
    }
    
    pub fn add_emitter(&mut self, emitter: Box<dyn MetricsEmitter>) {
        self.emitters.push(emitter);
    }
    
    /// Emit event to all configured emitters
    pub async fn emit_event(&self, event: &MetricEvent) -> Vec<(String, EmissionResult)> {
        let mut results = Vec::new();
        
        for emitter in &self.emitters {
            let result = emitter.emit_event(event.clone()).await;
            results.push((emitter.name().to_string(), result));
        }
        
        results
    }
    
    /// Get health status of all emitters
    pub async fn health_status(&self) -> Vec<(String, bool)> {
        let mut status = Vec::new();
        
        for emitter in &self.emitters {
            let healthy = emitter.health_check().await;
            status.push((emitter.name().to_string(), healthy));
        }
        
        status
    }
}

impl fmt::Debug for EmissionPipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmissionPipeline")
            .field("emitter_count", &self.emitters.len())
            .field("config", &self.config)
            .finish()
    }
}