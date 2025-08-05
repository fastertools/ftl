use super::types::{RequestMetadata, TimingInfo};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Extensions {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Extensions {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(val));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut())
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok())
            .map(|boxed| *boxed)
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MiddlewareContext {
    pub request_id: String,
    pub tool_name: String,
    pub component_name: String,
    pub metadata: RequestMetadata,
    pub timing: TimingInfo,
    pub extensions: Extensions,
    pub tool_result: Option<ToolResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub response_size: Option<usize>,
}

impl MiddlewareContext {
    pub fn new(tool_name: String, component_name: String) -> Self {
        let request_id = format!("{}-{}", uuid::Uuid::new_v4(), chrono::Utc::now().timestamp_millis());
        
        Self {
            request_id,
            tool_name,
            component_name,
            metadata: RequestMetadata::default(),
            timing: TimingInfo::new(),
            extensions: Extensions::new(),
            tool_result: None,
            error: None,
        }
    }

    pub fn set_tool_success(&mut self, success: bool, response_size: Option<usize>) {
        self.tool_result = Some(ToolResult {
            success,
            response_size,
        });
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        if let Some(result) = &mut self.tool_result {
            result.success = false;
        }
    }

    pub fn is_success(&self) -> bool {
        self.tool_result
            .as_ref()
            .map(|r| r.success)
            .unwrap_or(false)
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn request_size(&self) -> Option<usize> {
        self.extensions.get::<RequestSize>().map(|s| s.0)
    }

    pub fn set_request_size(&mut self, size: usize) {
        self.extensions.insert(RequestSize(size));
    }
}

pub struct RequestSize(pub usize);