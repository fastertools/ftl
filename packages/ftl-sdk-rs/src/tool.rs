use serde_json::Value;

use crate::types::{ToolError, ToolResult};

/// Trait that all FTL Core tools must implement
///
/// This trait defines a single computational tool optimized for WebAssembly
/// performance. FTL Core follows the "1 tool per server" architecture for
/// maximum efficiency.
pub trait Tool: Clone {
    /// The name of the tool (used in MCP tool calls)
    fn name(&self) -> &'static str;

    /// Human-readable description of what the tool does
    fn description(&self) -> &'static str;

    /// JSON schema for the tool's input parameters
    ///
    /// Return a JSON object describing the expected input structure.
    /// Example:
    /// ```json
    /// {
    ///   "type": "object",
    ///   "properties": {
    ///     "text": {
    ///       "type": "string",
    ///       "description": "The text to process"
    ///     }
    ///   },
    ///   "required": ["text"]
    /// }
    /// ```
    fn input_schema(&self) -> Value;

    /// Execute the tool with the provided arguments
    ///
    /// This is where your tool's core logic goes. The arguments are validated
    /// against your input schema before this method is called.
    fn call(&self, arguments: &Value) -> Result<ToolResult, ToolError>;

    /// Optional: Custom server name (defaults to tool name)
    fn server_name(&self) -> String {
        let name = self.name();
        format!("ftl-{name}")
    }

    /// Optional: Server version (defaults to "0.0.1")
    fn server_version(&self) -> &'static str {
        "0.0.1"
    }

    /// Optional: Additional server capabilities
    fn capabilities(&self) -> Value {
        serde_json::json!({
            "tools": {}
        })
    }
}

/// Information about a tool for MCP tool listing
#[derive(Debug)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl<T: Tool> From<&T> for ToolInfo {
    fn from(tool: &T) -> Self {
        Self {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            input_schema: tool.input_schema(),
        }
    }
}
