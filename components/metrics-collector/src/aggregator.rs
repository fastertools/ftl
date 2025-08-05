use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MetricEvent {
    pub timestamp: u64,
    pub tool_name: String,
    pub component_name: String,
    pub duration_ms: f64,
    pub success: bool,
    pub request_size: Option<usize>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct ToolMetrics {
    pub invocation_count: AtomicU64,
    pub success_count: AtomicU64,
    pub failure_count: AtomicU64,
    pub total_duration_ms: AtomicU64,
    pub total_request_size: AtomicU64,
    pub min_duration_ms: AtomicU64,
    pub max_duration_ms: AtomicU64,
}

impl ToolMetrics {
    pub fn new() -> Self {
        Self {
            min_duration_ms: AtomicU64::new(u64::MAX),
            ..Default::default()
        }
    }

    pub fn record_invocation(&self, success: bool, duration_ms: u64, request_size: Option<usize>) {
        self.invocation_count.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.success_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
        }
        
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        if let Some(size) = request_size {
            self.total_request_size.fetch_add(size as u64, Ordering::Relaxed);
        }
        
        // Update min/max duration
        let _ = self.min_duration_ms.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
            if duration_ms < current {
                Some(duration_ms)
            } else {
                None
            }
        });
        
        let _ = self.max_duration_ms.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
            if duration_ms > current {
                Some(duration_ms)
            } else {
                None
            }
        });
    }

    pub fn to_json(&self) -> serde_json::Value {
        let invocation_count = self.invocation_count.load(Ordering::Relaxed);
        let avg_duration = if invocation_count > 0 {
            self.total_duration_ms.load(Ordering::Relaxed) as f64 / invocation_count as f64
        } else {
            0.0
        };

        serde_json::json!({
            "invocation_count": invocation_count,
            "success_count": self.success_count.load(Ordering::Relaxed),
            "failure_count": self.failure_count.load(Ordering::Relaxed),
            "total_duration_ms": self.total_duration_ms.load(Ordering::Relaxed),
            "avg_duration_ms": avg_duration,
            "min_duration_ms": if invocation_count > 0 { self.min_duration_ms.load(Ordering::Relaxed) } else { 0 },
            "max_duration_ms": self.max_duration_ms.load(Ordering::Relaxed),
            "total_request_size": self.total_request_size.load(Ordering::Relaxed),
        })
    }
}

#[derive(Clone)]
pub struct MetricsAggregator {
    tool_metrics: Arc<std::sync::Mutex<HashMap<String, Arc<ToolMetrics>>>>,
    global_metrics: Arc<GlobalMetrics>,
    max_metrics: usize,
}

pub struct GlobalMetrics {
    pub total_invocations: AtomicU64,
    pub active_invocations: AtomicU64,
    pub peak_concurrency: AtomicU64,
}

impl MetricsAggregator {
    pub fn new(max_metrics: usize) -> Self {
        Self {
            tool_metrics: Arc::new(std::sync::Mutex::new(HashMap::new())),
            global_metrics: Arc::new(GlobalMetrics {
                total_invocations: AtomicU64::new(0),
                active_invocations: AtomicU64::new(0),
                peak_concurrency: AtomicU64::new(0),
            }),
            max_metrics,
        }
    }

    pub async fn record_event(&self, event: MetricEvent) {
        
        // Increment global counters
        self.global_metrics.total_invocations.fetch_add(1, Ordering::Relaxed);
        
        // Get or create tool metrics
        let tool_metrics = {
            let mut metrics_map = self.tool_metrics.lock().unwrap();
            
            // Enforce max metrics limit with simple eviction
            if metrics_map.len() > self.max_metrics {
                if let Some(first_key) = metrics_map.keys().next().cloned() {
                    metrics_map.remove(&first_key);
                }
            }
            
            metrics_map.entry(event.tool_name.clone())
                .or_insert_with(|| Arc::new(ToolMetrics::new()))
                .clone()
        };
        
        // Record the invocation (outside the lock)
        tool_metrics.record_invocation(
            event.success,
            event.duration_ms as u64,
            event.request_size,
        );
        
    }

    pub async fn get_all_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();
        
        // Add global metrics
        metrics.insert("_global".to_string(), serde_json::json!({
            "total_invocations": self.global_metrics.total_invocations.load(Ordering::Relaxed),
            "active_invocations": self.global_metrics.active_invocations.load(Ordering::Relaxed),
            "peak_concurrency": self.global_metrics.peak_concurrency.load(Ordering::Relaxed),
        }));
        
        // Add tool metrics
        let metrics_map = self.tool_metrics.lock().unwrap();
        for (key, value) in metrics_map.iter() {
            metrics.insert(key.clone(), value.to_json());
        }
        
        metrics
    }

    pub async fn get_tool_metrics(&self, tool_name: &str) -> Option<serde_json::Value> {
        let metrics_map = self.tool_metrics.lock().unwrap();
        metrics_map.get(tool_name).map(|metrics| metrics.to_json())
    }
}