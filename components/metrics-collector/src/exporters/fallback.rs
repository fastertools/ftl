use crate::aggregator::MetricEvent;
use crate::emission::{EmissionResult, MetricsEmitter};
use serde_json;
use std::collections::VecDeque;
use std::sync::Mutex;

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing fast
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker for managing fallback behavior
#[derive(Debug)]
pub struct CircuitBreaker {
    state: Mutex<CircuitState>,
    failure_count: Mutex<u32>,
    last_failure_time: Mutex<Option<std::time::Instant>>,
    failure_threshold: u32,
    reset_timeout_seconds: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout_seconds: u64) -> Self {
        Self {
            state: Mutex::new(CircuitState::Closed),
            failure_count: Mutex::new(0),
            last_failure_time: Mutex::new(None),
            failure_threshold,
            reset_timeout_seconds,
        }
    }
    
    pub fn should_allow_request(&self) -> bool {
        let state = self.state.lock().unwrap().clone();
        
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if enough time has passed to try half-open
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed().as_secs() >= self.reset_timeout_seconds {
                        *self.state.lock().unwrap() = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CircuitState::HalfOpen => true,
        }
    }
    
    pub fn record_success(&self) {
        *self.failure_count.lock().unwrap() = 0;
        *self.state.lock().unwrap() = CircuitState::Closed;
    }
    
    pub fn record_failure(&self) {
        let mut failure_count = self.failure_count.lock().unwrap();
        *failure_count += 1;
        
        *self.last_failure_time.lock().unwrap() = Some(std::time::Instant::now());
        
        if *failure_count >= self.failure_threshold {
            *self.state.lock().unwrap() = CircuitState::Open;
        }
    }
    
    pub fn get_state(&self) -> CircuitState {
        self.state.lock().unwrap().clone()
    }
}

/// Fallback emitter configuration
#[derive(Debug, Clone)]
pub struct FallbackEmitterConfig {
    pub max_buffer_size: usize,
    pub circuit_failure_threshold: u32,
    pub circuit_reset_timeout_seconds: u64,
    pub enable_local_storage: bool,
}

impl Default for FallbackEmitterConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 10000,
            circuit_failure_threshold: 5,
            circuit_reset_timeout_seconds: 60,
            enable_local_storage: true,
        }
    }
}

/// Fallback emitter that handles primary emitter failures
pub struct FallbackEmitter {
    config: FallbackEmitterConfig,
    circuit_breaker: CircuitBreaker,
    buffer: Mutex<VecDeque<MetricEvent>>,
}

impl FallbackEmitter {
    pub fn new(config: FallbackEmitterConfig) -> Self {
        let circuit_breaker = CircuitBreaker::new(
            config.circuit_failure_threshold,
            config.circuit_reset_timeout_seconds,
        );
        
        Self {
            config,
            circuit_breaker,
            buffer: Mutex::new(VecDeque::new()),
        }
    }
    
    /// Store event in local buffer for retry or analysis
    fn buffer_event(&self, event: &MetricEvent) -> Result<(), String> {
        let mut buffer = self.buffer.lock().unwrap();
        
        // Check buffer size limit
        if buffer.len() >= self.config.max_buffer_size {
            // Remove oldest event to make room
            buffer.pop_front();
        }
        
        buffer.push_back(event.clone());
        Ok(())
    }
    
    /// Get buffered events count for monitoring
    pub fn get_buffer_size(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }
    
    /// Clear the buffer (for testing or maintenance)
    pub fn clear_buffer(&self) {
        self.buffer.lock().unwrap().clear();
    }
    
    /// Get circuit breaker status for monitoring
    pub fn get_circuit_status(&self) -> (CircuitState, u32) {
        let state = self.circuit_breaker.get_state();
        let failure_count = *self.circuit_breaker.failure_count.lock().unwrap();
        (state, failure_count)
    }
    
    /// Simulate local storage for fallback scenarios
    async fn store_locally(&self, event: &MetricEvent) -> Result<(), String> {
        if !self.config.enable_local_storage {
            return Err("Local storage disabled".to_string());
        }
        
        // STUBBED: In real implementation, this would write to local file/database
        // For now, we log the event with fallback marker
        let json_event = serde_json::to_string_pretty(event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;
        
        println!("FALLBACK: Storing event locally");
        println!("FALLBACK: Circuit state: {:?}", self.circuit_breaker.get_state());
        println!("FALLBACK: Buffer size: {}", self.get_buffer_size());
        println!("FALLBACK: Event: {}", json_event);
        
        Ok(())
    }
}

impl MetricsEmitter for FallbackEmitter {
    fn emit_event(&self, event: MetricEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmissionResult> + Send + '_>> {
        Box::pin(self.emit_event_impl(event))
    }
    
    fn name(&self) -> &'static str {
        "fallback"
    }
    
    fn health_check(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        let enable_local_storage = self.config.enable_local_storage;
        let (circuit_state, _) = self.get_circuit_status();
        
        Box::pin(async move {
            match circuit_state {
                CircuitState::Open => false,
                _ => enable_local_storage,
            }
        })
    }
}

impl FallbackEmitter {
    async fn emit_event_impl(&self, event: MetricEvent) -> EmissionResult {
        // Check circuit breaker
        if !self.circuit_breaker.should_allow_request() {
            // Circuit is open, go straight to fallback
            self.buffer_event(&event).ok();
            
            match self.store_locally(&event).await {
                Ok(()) => EmissionResult::Fallback("Circuit breaker open - stored locally".to_string()),
                Err(err) => EmissionResult::Failed(format!("Fallback storage failed: {}", err)),
            }
        } else {
            // Circuit allows request, but fallback emitter always falls back
            // This emitter represents the "fallback path" itself
            self.buffer_event(&event).ok();
            
            match self.store_locally(&event).await {
                Ok(()) => {
                    self.circuit_breaker.record_success();
                    EmissionResult::Fallback("Fallback emitter - stored locally".to_string())
                },
                Err(err) => {
                    self.circuit_breaker.record_failure();
                    EmissionResult::Failed(format!("Fallback storage failed: {}", err))
                },
            }
        }
    }
}