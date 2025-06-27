//! # FTL Core
//! 
//! Core MCP server implementation for building AI agent tools.

// Memory allocator for WebAssembly
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: talc::Talck<talc::locking::AssumeUnlockable, talc::ClaimOnOom> = {
    static mut ARENA: [u8; 2 * 1024 * 1024] = [0; 2 * 1024 * 1024];
    talc::Talc::new(unsafe {
        talc::ClaimOnOom::new(talc::Span::from_base_size(
            ARENA.as_mut_ptr(),
            ARENA.len(),
        ))
    })
    .lock()
};

pub mod types;
pub mod tool;
pub mod server;
pub mod spin;

// Re-export commonly used items
pub use server::McpServer;
pub use tool::Tool;
pub use types::{JsonRpcRequest, JsonRpcResponse, ToolResult, ToolError};

// The macro is already exported by spin module

/// Convenient re-exports for tool implementations
pub mod prelude {
    pub use crate::tool::Tool;
    pub use crate::types::{ToolResult, ToolError};
    pub use crate::ftl_mcp_server;
    pub use serde_json;
    pub use serde_json::json;
}