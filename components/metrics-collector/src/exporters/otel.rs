use crate::aggregator::MetricEvent;
use crate::emission::{EmissionResult, MetricsEmitter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenTelemetry metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtelMetricType {
    Counter,
    Gauge,
    Histogram,
}

/// OpenTelemetry metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelDataPoint {
    pub timestamp: u64,
    pub value: f64,
    pub attributes: HashMap<String, String>,
}

/// OpenTelemetry metric structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelMetric {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub metric_type: OtelMetricType,
    pub data_points: Vec<OtelDataPoint>,
}

/// OpenTelemetry resource attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelResource {
    pub service_name: String,
    pub service_version: String,
    pub service_instance_id: String,
    pub attributes: HashMap<String, String>,
}

/// Complete OTEL metrics payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelMetricsPayload {
    pub resource: OtelResource,
    pub metrics: Vec<OtelMetric>,
}

/// Configuration for OTEL emitter
#[derive(Debug, Clone)]
pub struct OtelEmitterConfig {
    pub endpoint: String,
    pub service_name: String,
    pub service_version: String,
    pub batch_size: usize,
}

impl Default for OtelEmitterConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4318/v1/metrics".to_string(), // OTLP HTTP endpoint
            service_name: "ftl-metrics-collector".to_string(),
            service_version: "0.1.0".to_string(),
            batch_size: 100,
        }
    }
}

/// OTEL metrics emitter (stubbed implementation)
pub struct OtelEmitter {
    config: OtelEmitterConfig,
    batch: Vec<OtelMetric>,
}

impl OtelEmitter {
    pub fn new(config: OtelEmitterConfig) -> Self {
        Self {
            config,
            batch: Vec::new(),
        }
    }
    
    /// Convert MetricEvent to OTEL metrics
    fn convert_to_otel_metrics(&self, event: &MetricEvent) -> Vec<OtelMetric> {
        let mut metrics = Vec::new();
        let timestamp = event.timestamp;
        
        // Base attributes from the event
        let mut base_attributes = HashMap::new();
        base_attributes.insert("tool_name".to_string(), event.tool_name.clone());
        base_attributes.insert("component_name".to_string(), event.component_name.clone());
        
        // Add metadata as attributes
        for (key, value) in &event.metadata {
            base_attributes.insert(key.clone(), value.clone());
        }
        
        // Tool invocation counter
        metrics.push(OtelMetric {
            name: "ftl_tool_invocations_total".to_string(),
            description: "Total number of tool invocations".to_string(),
            unit: "1".to_string(),
            metric_type: OtelMetricType::Counter,
            data_points: vec![OtelDataPoint {
                timestamp,
                value: 1.0,
                attributes: base_attributes.clone(),
            }],
        });
        
        // Success/failure counter
        let success_value = if event.success { 1.0 } else { 0.0 };
        let failure_value = if event.success { 0.0 } else { 1.0 };
        
        metrics.push(OtelMetric {
            name: "ftl_tool_success_total".to_string(),
            description: "Total number of successful tool invocations".to_string(),
            unit: "1".to_string(),
            metric_type: OtelMetricType::Counter,
            data_points: vec![OtelDataPoint {
                timestamp,
                value: success_value,
                attributes: base_attributes.clone(),
            }],
        });
        
        metrics.push(OtelMetric {
            name: "ftl_tool_failures_total".to_string(),
            description: "Total number of failed tool invocations".to_string(),
            unit: "1".to_string(),
            metric_type: OtelMetricType::Counter,
            data_points: vec![OtelDataPoint {
                timestamp,
                value: failure_value,
                attributes: base_attributes.clone(),
            }],
        });
        
        // Duration histogram
        metrics.push(OtelMetric {
            name: "ftl_tool_duration_ms".to_string(),
            description: "Tool invocation duration in milliseconds".to_string(),
            unit: "ms".to_string(),
            metric_type: OtelMetricType::Histogram,
            data_points: vec![OtelDataPoint {
                timestamp,
                value: event.duration_ms,
                attributes: base_attributes.clone(),
            }],
        });
        
        // Request size gauge (if available)
        if let Some(size) = event.request_size {
            metrics.push(OtelMetric {
                name: "ftl_tool_request_size_bytes".to_string(),
                description: "Size of tool request in bytes".to_string(),
                unit: "bytes".to_string(),
                metric_type: OtelMetricType::Gauge,
                data_points: vec![OtelDataPoint {
                    timestamp,
                    value: size as f64,
                    attributes: base_attributes,
                }],
            });
        }
        
        metrics
    }
    
    /// Create OTEL resource information
    fn create_resource(&self) -> OtelResource {
        let mut attributes = HashMap::new();
        attributes.insert("deployment.environment".to_string(), "development".to_string());
        
        OtelResource {
            service_name: self.config.service_name.clone(),
            service_version: self.config.service_version.clone(),
            service_instance_id: uuid::Uuid::new_v4().to_string(),
            attributes,
        }
    }
    
    /// Stub: Send metrics to OTEL endpoint
    async fn send_metrics(&self, payload: &OtelMetricsPayload) -> Result<(), String> {
        // STUBBED: In real implementation, this would send HTTP request to OTEL endpoint
        // For now, we just log the structured payload
        
        let json_payload = serde_json::to_string_pretty(payload)
            .map_err(|e| format!("Failed to serialize OTEL payload: {}", e))?;
            
        println!("OTEL: Sending metrics to {}", self.config.endpoint);
        println!("OTEL: {}", json_payload);
        
        // Simulate success
        Ok(())
    }
}

impl MetricsEmitter for OtelEmitter {
    fn emit_event(&self, event: MetricEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmissionResult> + Send + '_>> {
        Box::pin(self.emit_event_impl(event))
    }
    
    fn name(&self) -> &'static str {
        "otel"
    }
    
    fn health_check(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async { true }) // STUBBED: In real implementation, this would ping the OTEL endpoint
    }
}

impl OtelEmitter {
    async fn emit_event_impl(&self, event: MetricEvent) -> EmissionResult {
        // Convert event to OTEL metrics
        let otel_metrics = self.convert_to_otel_metrics(&event);
        
        // Create payload
        let payload = OtelMetricsPayload {
            resource: self.create_resource(),
            metrics: otel_metrics,
        };
        
        // Send to OTEL endpoint (stubbed)
        match self.send_metrics(&payload).await {
            Ok(()) => EmissionResult::Success,
            Err(err) => EmissionResult::Failed(format!("OTEL emission failed: {}", err)),
        }
    }
}