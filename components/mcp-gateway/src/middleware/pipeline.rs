use super::context::MiddlewareContext;
use super::types::{MiddlewareError, Middleware};
use super::invocation_tracker::InvocationTracker;

pub struct MiddlewarePipeline {
    invocation_tracker: Option<InvocationTracker>,
}

impl MiddlewarePipeline {
    pub fn new() -> Self {
        Self {
            invocation_tracker: None,
        }
    }

    pub fn add_invocation_tracker(&mut self, tracker: InvocationTracker) {
        self.invocation_tracker = Some(tracker);
    }

    pub async fn pre_process(&self, ctx: &mut MiddlewareContext) -> Result<(), MiddlewareError> {
        if let Some(tracker) = &self.invocation_tracker {
            if let Err(err) = tracker.pre_process(ctx).await {
                eprintln!("Middleware pre_process error: {} (fatal: {})", err.message, err.is_fatal);
                if err.is_fatal {
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    pub async fn post_process(&self, ctx: &mut MiddlewareContext) -> Result<(), MiddlewareError> {
        if let Some(tracker) = &self.invocation_tracker {
            if let Err(err) = tracker.post_process(ctx).await {
                eprintln!("Middleware post_process error: {} (fatal: {})", err.message, err.is_fatal);
                if err.is_fatal {
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), MiddlewareError> {
        if let Some(tracker) = &self.invocation_tracker {
            if let Err(err) = tracker.shutdown().await {
                eprintln!("Middleware shutdown error: {}", err.message);
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        if self.invocation_tracker.is_some() { 1 } else { 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.invocation_tracker.is_none()
    }
}

impl Default for MiddlewarePipeline {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MiddlewareBuilder {
    pipeline: MiddlewarePipeline,
}

impl MiddlewareBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: MiddlewarePipeline::new(),
        }
    }

    pub fn with_invocation_tracker(mut self, tracker: InvocationTracker) -> Self {
        self.pipeline.add_invocation_tracker(tracker);
        self
    }

    pub fn build(self) -> MiddlewarePipeline {
        self.pipeline
    }
}

impl Default for MiddlewareBuilder {
    fn default() -> Self {
        Self::new()
    }
}