//! # FTL Core
//!
//! Core MCP server implementation for building AI agent tools.

pub mod gateway;
pub mod mcp;
pub mod server;
pub mod spin;
pub mod tool;
pub mod types;

#[cfg(test)]
mod gateway_test;

// Re-export commonly used items
pub use gateway::{GatewayConfig, GatewayHandler, McpGateway, ToolEndpoint};
pub use server::McpServer;
pub use tool::Tool;
pub use types::{JsonRpcRequest, JsonRpcResponse, ToolError, ToolResult};

/// Convenient re-exports for tool implementations
pub mod prelude {
    pub use serde_json::{self, json};

    pub use crate::{
        ftl_mcp_server,
        tool::Tool,
        types::{ToolError, ToolResult},
    };
}
