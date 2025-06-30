use ftl_sdk_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct EchoTool;

#[derive(Debug, Deserialize)]
struct EchoInput {
    message: String,
}

#[derive(Debug, Serialize)]
struct EchoOutput {
    echo: String,
}

impl Tool for EchoTool {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Echoes back the input message"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The message to echo back"
                }
            },
            "required": ["message"]
        })
    }

    fn call(&self, input: &serde_json::Value) -> Result<ToolResult, ToolError> {
        let input: EchoInput = serde_json::from_value(input.clone())
            .map_err(|e| ToolError::InvalidArguments(e.to_string()))?;
        
        Ok(ToolResult::text(format!("Echo: {}", input.message)))
    }
}

ftl_mcp_server!(EchoTool);