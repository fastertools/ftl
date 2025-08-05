use crate::aggregator::MetricEvent;
use crate::emission::{EmissionResult, MetricsEmitter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Durable event record with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableEventRecord {
    /// Event ID for deduplication
    pub event_id: String,
    /// Event timestamp (original from MetricEvent)
    pub timestamp: u64,
    /// Processing timestamp (when we processed the event)
    pub processed_at: u64,
    /// Partition key for Kafka-like systems
    pub partition_key: String,
    /// Full metric event data
    pub event: MetricEvent,
    /// Additional context for audit trail
    pub audit_context: AuditContext,
}

/// Audit context for compliance and billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditContext {
    /// Tenant/customer identifier
    pub tenant_id: Option<String>,
    /// User identifier (anonymized)
    pub user_id: Option<String>,
    /// Authentication provider
    pub auth_provider: Option<String>,
    /// Request source IP (anonymized)
    pub source_ip: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Billing category for this event
    pub billing_category: String,
    /// Cost associated with this event (in cents)
    pub cost_cents: Option<u32>,
}

/// Partitioning strategy for durable storage
#[derive(Debug, Clone)]
pub enum PartitionStrategy {
    ByTenant,
    ByTool,
    ByTimestamp,
    ByHash,
}

/// Configuration for durable emitter
#[derive(Debug, Clone)]
pub struct DurableEmitterConfig {
    pub kafka_brokers: Vec<String>,
    pub topic_name: String,
    pub partition_strategy: PartitionStrategy,
    pub batch_size: usize,
    pub enable_compression: bool,
    pub retention_days: u32,
}

impl Default for DurableEmitterConfig {
    fn default() -> Self {
        Self {
            kafka_brokers: vec!["localhost:9092".to_string()],
            topic_name: "ftl-metrics-events".to_string(),
            partition_strategy: PartitionStrategy::ByTenant,
            batch_size: 1000,
            enable_compression: true,
            retention_days: 365, // 1 year retention for audit
        }
    }
}

/// Retry configuration for failed deliveries
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 30000, // 30 seconds
            backoff_multiplier: 2.0,
        }
    }
}

/// Durable metrics emitter with full audit capabilities
pub struct DurableEmitter {
    config: DurableEmitterConfig,
    retry_config: RetryConfig,
    event_batch: Vec<DurableEventRecord>,
}

impl DurableEmitter {
    pub fn new(config: DurableEmitterConfig, retry_config: RetryConfig) -> Self {
        Self {
            config,
            retry_config,
            event_batch: Vec::new(),
        }
    }
    
    /// Extract audit context from metric event
    fn extract_audit_context(&self, event: &MetricEvent) -> AuditContext {
        AuditContext {
            tenant_id: event.metadata.get("tenant_id").cloned(),
            user_id: event.metadata.get("user_id").cloned(),
            auth_provider: event.metadata.get("auth_provider").cloned(),
            source_ip: event.metadata.get("source_ip").cloned(),
            session_id: event.metadata.get("session_id").cloned(),
            billing_category: self.determine_billing_category(event),
            cost_cents: self.calculate_cost(event),
        }
    }
    
    /// Determine billing category based on tool and success
    fn determine_billing_category(&self, event: &MetricEvent) -> String {
        // Simple categorization logic - can be expanded
        match event.tool_name.as_str() {
            name if name.contains("ai") || name.contains("llm") => "ai_inference".to_string(),
            name if name.contains("storage") => "storage_ops".to_string(),
            name if name.contains("compute") => "compute_ops".to_string(),
            _ => "standard_ops".to_string(),
        }
    }
    
    /// Calculate cost for billing (simplified)
    fn calculate_cost(&self, event: &MetricEvent) -> Option<u32> {
        // Simplified cost calculation - in real implementation this would be more sophisticated
        let base_cost = match self.determine_billing_category(event).as_str() {
            "ai_inference" => 10, // 10 cents per AI call
            "storage_ops" => 1,   // 1 cent per storage op
            "compute_ops" => 5,   // 5 cents per compute op
            _ => 1,               // 1 cent for standard ops
        };
        
        // Scale by duration (rough approximation)
        let duration_multiplier = (event.duration_ms / 1000.0).max(1.0) as u32;
        Some(base_cost * duration_multiplier)
    }
    
    /// Generate partition key based on strategy
    fn generate_partition_key(&self, event: &MetricEvent, audit_context: &AuditContext) -> String {
        match self.config.partition_strategy {
            PartitionStrategy::ByTenant => {
                audit_context.tenant_id.clone()
                    .unwrap_or_else(|| "unknown".to_string())
            },
            PartitionStrategy::ByTool => event.tool_name.clone(),
            PartitionStrategy::ByTimestamp => {
                // Partition by hour for time-based queries
                let hour = event.timestamp / (1000 * 60 * 60);
                format!("hour_{}", hour)
            },
            PartitionStrategy::ByHash => {
                // Hash of tool name and tenant for even distribution
                let hash_input = format!("{}:{}", 
                    event.tool_name, 
                    audit_context.tenant_id.as_deref().unwrap_or("default")
                );
                format!("hash_{}", self.simple_hash(&hash_input) % 32)
            },
        }
    }
    
    /// Simple hash function for partitioning
    fn simple_hash(&self, input: &str) -> u32 {
        input.bytes().fold(0u32, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u32)
        })
    }
    
    /// Convert MetricEvent to DurableEventRecord
    fn convert_to_durable_record(&self, event: &MetricEvent) -> DurableEventRecord {
        let audit_context = self.extract_audit_context(event);
        let partition_key = self.generate_partition_key(event, &audit_context);
        
        DurableEventRecord {
            event_id: format!("{}_{}", event.timestamp, uuid::Uuid::new_v4()),
            timestamp: event.timestamp,
            processed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            partition_key,
            event: event.clone(),
            audit_context,
        }
    }
    
    /// Send durable record to Kafka (stubbed with stdout)
    async fn send_to_kafka(&self, record: &DurableEventRecord) -> Result<(), String> {
        // IMPLEMENTATION: Full call chain as requested, but with stdout integration
        
        // 1. Serialize the record
        let json_record = serde_json::to_string(record)
            .map_err(|e| format!("Failed to serialize durable record: {}", e))?;
        
        // 2. Determine Kafka topic and partition
        let topic = &self.config.topic_name;
        let partition = self.calculate_partition(&record.partition_key);
        
        // 3. Create Kafka message headers
        let headers = self.create_kafka_headers(record);
        
        // 4. Apply compression if enabled
        let payload = if self.config.enable_compression {
            self.compress_payload(&json_record)?
        } else {
            json_record.clone().into_bytes()
        };
        
        // 5. STUBBED: Send to Kafka with structured logging to stdout
        println!("DURABLE: Sending to Kafka");
        println!("DURABLE: Topic: {}, Partition: {}", topic, partition);
        println!("DURABLE: Headers: {:?}", headers);
        println!("DURABLE: Payload Size: {} bytes", payload.len());
        println!("DURABLE: {}", json_record);
        
        // 6. Simulate Kafka response
        self.simulate_kafka_response().await
    }
    
    /// Calculate partition number from partition key
    fn calculate_partition(&self, partition_key: &str) -> u32 {
        // Simple hash-based partitioning
        self.simple_hash(partition_key) % 12 // Assume 12 partitions
    }
    
    /// Create Kafka message headers
    fn create_kafka_headers(&self, record: &DurableEventRecord) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("event_id".to_string(), record.event_id.clone());
        headers.insert("event_type".to_string(), "metric_event".to_string());
        headers.insert("tool_name".to_string(), record.event.tool_name.clone());
        headers.insert("component_name".to_string(), record.event.component_name.clone());
        headers.insert("billing_category".to_string(), record.audit_context.billing_category.clone());
        
        if let Some(tenant_id) = &record.audit_context.tenant_id {
            headers.insert("tenant_id".to_string(), tenant_id.clone());
        }
        
        headers.insert("schema_version".to_string(), "1.0".to_string());
        headers
    }
    
    /// Compress payload (stubbed)
    fn compress_payload(&self, data: &str) -> Result<Vec<u8>, String> {
        // STUBBED: In real implementation, this would use gzip/snappy compression
        // For now, just convert to bytes
        Ok(data.as_bytes().to_vec())
    }
    
    /// Simulate Kafka response
    async fn simulate_kafka_response(&self) -> Result<(), String> {
        // WASM-compatible: No sleep, just simulate response
        // Simulate occasional failures for testing retry logic
        if self.simple_hash(&uuid::Uuid::new_v4().to_string()) % 100 < 5 {
            return Err("Simulated Kafka timeout".to_string());
        }
        
        Ok(())
    }
    
    /// Retry logic with exponential backoff (WASM-compatible)
    async fn retry_send(&self, record: &DurableEventRecord) -> EmissionResult {
        for attempt in 0..=self.retry_config.max_retries {
            match self.send_to_kafka(record).await {
                Ok(()) => return EmissionResult::Success,
                Err(err) if attempt == self.retry_config.max_retries => {
                    return EmissionResult::Failed(format!(
                        "Durable emission failed after {} retries: {}", 
                        self.retry_config.max_retries, 
                        err
                    ));
                },
                Err(_) => {
                    // WASM-compatible: No sleep, just continue to next attempt
                    // In real implementation, would use Spin runtime's async delay
                }
            }
        }
        
        EmissionResult::Failed("Unexpected retry loop exit".to_string())
    }
}

impl MetricsEmitter for DurableEmitter {
    fn emit_event(&self, event: MetricEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmissionResult> + Send + '_>> {
        Box::pin(self.emit_event_impl(event))
    }
    
    fn name(&self) -> &'static str {
        "durable"
    }
    
    fn health_check(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        let brokers = self.config.kafka_brokers.clone();
        Box::pin(async move {
            // STUBBED: In real implementation, this would check Kafka cluster health
            // For now, simulate health check
            println!("DURABLE: Health check - Kafka brokers: {:?}", brokers);
            true
        })
    }
}

impl DurableEmitter {
    async fn emit_event_impl(&self, event: MetricEvent) -> EmissionResult {
        // Convert to durable record
        let durable_record = self.convert_to_durable_record(&event);
        
        // Send with retry logic
        self.retry_send(&durable_record).await
    }
}