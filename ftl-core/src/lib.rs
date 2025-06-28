//! # FTL Core
//!
//! Core MCP server implementation for building AI agent tools.

pub mod server;
pub mod spin;
pub mod tool;
pub mod types;

// Re-export commonly used items
pub use server::McpServer;
pub use tool::Tool;
pub use types::{JsonRpcRequest, JsonRpcResponse, ToolError, ToolResult};

// The macro is already exported by spin module

/// Convenient re-exports for tool implementations
pub mod prelude {
    pub use serde_json::{self, json};

    pub use crate::{
        ftl_mcp_server,
        tool::Tool,
        types::{ToolError, ToolResult},
    };
}
