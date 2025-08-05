use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct MiddlewareError {
    pub message: String,
    pub is_fatal: bool,
}

impl MiddlewareError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_fatal: false,
        }
    }

    pub fn fatal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_fatal: true,
        }
    }
}

impl fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MiddlewareError: {}", self.message)
    }
}

impl std::error::Error for MiddlewareError {}

pub trait Middleware: 'static {
    async fn pre_process(
        &self,
        ctx: &mut crate::middleware::context::MiddlewareContext,
    ) -> Result<(), MiddlewareError>;

    async fn post_process(
        &self,
        ctx: &mut crate::middleware::context::MiddlewareContext,
    ) -> Result<(), MiddlewareError>;

    async fn shutdown(&self) -> Result<(), MiddlewareError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub client_id: Option<String>,
    pub user_agent: Option<String>,
    pub source_ip: Option<String>,
    pub additional: HashMap<String, String>,
}

impl Default for RequestMetadata {
    fn default() -> Self {
        Self {
            client_id: None,
            user_agent: None,
            source_ip: None,
            additional: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimingInfo {
    pub request_received_at: std::time::Instant,
    pub pre_process_start: Option<std::time::Instant>,
    pub pre_process_end: Option<std::time::Instant>,
    pub tool_execution_start: Option<std::time::Instant>,
    pub tool_execution_end: Option<std::time::Instant>,
    pub post_process_start: Option<std::time::Instant>,
    pub post_process_end: Option<std::time::Instant>,
}

impl TimingInfo {
    pub fn new() -> Self {
        Self {
            request_received_at: std::time::Instant::now(),
            pre_process_start: None,
            pre_process_end: None,
            tool_execution_start: None,
            tool_execution_end: None,
            post_process_start: None,
            post_process_end: None,
        }
    }

    pub fn total_duration(&self) -> std::time::Duration {
        self.request_received_at.elapsed()
    }

    pub fn tool_duration(&self) -> Option<std::time::Duration> {
        match (self.tool_execution_start, self.tool_execution_end) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }

    pub fn pre_process_duration(&self) -> Option<std::time::Duration> {
        match (self.pre_process_start, self.pre_process_end) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }

    pub fn post_process_duration(&self) -> Option<std::time::Duration> {
        match (self.post_process_start, self.post_process_end) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }
}

impl Default for TimingInfo {
    fn default() -> Self {
        Self::new()
    }
}