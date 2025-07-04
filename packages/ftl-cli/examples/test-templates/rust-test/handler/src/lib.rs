use serde_json::{json, Value};

wit_bindgen::generate!({
    world: "mcp-handler",
    path: "./wit",
    exports: {
        "component:mcp/handler": Component
    }
});

use exports::component::mcp::handler::{
    Guest, Tool, ToolResult, ResourceInfo, ResourceContents, Prompt, PromptMessage, Error as McpError
};

struct Component;

impl Guest for Component {
    fn list_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "rust_test".to_string(),
                description: "An MCP tool written in Rust".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input": { 
                            "type": "string", 
                            "description": "Input to process" 
                        }
                    },
                    "required": ["input"]
                }).to_string(),
            },
            // Add more tools here
        ]
    }
    
    fn call_tool(name: String, arguments: String) -> ToolResult {
        let args: Value = match serde_json::from_str(&arguments) {
            Ok(v) => v,
            Err(e) => return ToolResult::Error(McpError {
                code: -32602,
                message: format!("Invalid JSON arguments: {}", e),
                data: None,
            }),
        };
        
        match name.as_str() {
            "rust_test" => {
                let input = args["input"].as_str().unwrap_or("No input provided");
                // TODO: Implement your tool logic here
                ToolResult::Text(format!("Processed: {}", input))
            }
            _ => ToolResult::Error(McpError {
                code: -32601,
                message: format!("Unknown tool: {}", name),
                data: None,
            }),
        }
    }
    
    fn list_resources() -> Vec<ResourceInfo> {
        vec![
            // Add resources here
            // Example:
            // ResourceInfo {
            //     uri: "example://resource".to_string(),
            //     name: "Example Resource".to_string(),
            //     description: Some("An example resource".to_string()),
            //     mime_type: Some("text/plain".to_string()),
            // }
        ]
    }
    
    fn read_resource(uri: String) -> Result<ResourceContents, McpError> {
        // Implement resource reading logic here
        Err(McpError {
            code: -32601,
            message: format!("Resource not found: {}", uri),
            data: None,
        })
    }
    
    fn list_prompts() -> Vec<Prompt> {
        vec![
            // Add prompts here
            // Example:
            // Prompt {
            //     name: "greeting".to_string(),
            //     description: Some("Generate a greeting".to_string()),
            //     arguments: vec![
            //         PromptArgument {
            //             name: "name".to_string(),
            //             description: Some("Name to greet".to_string()),
            //             required: true,
            //         }
            //     ],
            // }
        ]
    }
    
    fn get_prompt(name: String, _arguments: String) -> Result<Vec<PromptMessage>, McpError> {
        // Implement prompt generation logic here
        Err(McpError {
            code: -32601,
            message: format!("Prompt not found: {}", name),
            data: None,
        })
    }
}