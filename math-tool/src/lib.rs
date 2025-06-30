use ftl_sdk_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MathTool;

#[derive(Debug, Deserialize)]
#[serde(tag = "operation")]
enum MathOperation {
    #[serde(rename = "add")]
    Add { a: f64, b: f64 },
    #[serde(rename = "multiply")]
    Multiply { a: f64, b: f64 },
    #[serde(rename = "divide")]
    Divide { a: f64, b: f64 },
}

impl Tool for MathTool {
    fn name(&self) -> &'static str {
        "math"
    }

    fn description(&self) -> &'static str {
        "Performs basic mathematical operations"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "operation": { "const": "add" },
                        "a": { "type": "number" },
                        "b": { "type": "number" }
                    },
                    "required": ["operation", "a", "b"]
                },
                {
                    "type": "object",
                    "properties": {
                        "operation": { "const": "multiply" },
                        "a": { "type": "number" },
                        "b": { "type": "number" }
                    },
                    "required": ["operation", "a", "b"]
                },
                {
                    "type": "object",
                    "properties": {
                        "operation": { "const": "divide" },
                        "a": { "type": "number" },
                        "b": { "type": "number" }
                    },
                    "required": ["operation", "a", "b"]
                }
            ]
        })
    }

    fn call(&self, input: &serde_json::Value) -> Result<ToolResult, ToolError> {
        let operation: MathOperation = serde_json::from_value(input.clone())
            .map_err(|e| ToolError::InvalidArguments(e.to_string()))?;
        
        let result = match operation {
            MathOperation::Add { a, b } => a + b,
            MathOperation::Multiply { a, b } => a * b,
            MathOperation::Divide { a, b } => {
                if b == 0.0 {
                    return Err(ToolError::InvalidArguments("Cannot divide by zero".to_string()));
                }
                a / b
            }
        };
        
        Ok(ToolResult::text(format!("Result: {}", result)))
    }
}

ftl_mcp_server!(MathTool);