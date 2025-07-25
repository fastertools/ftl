use ftl_sdk::{ftl_tools, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct {{project-name | pascal_case}}Input {
    /// The input message to process
    message: String,
}

ftl_tools! {
    /// {{tool-description}}
    fn {{project-name | snake_case}}(input: {{project-name | pascal_case}}Input) -> ToolResponse {
        // TODO: Implement your tool logic here
        ToolResponse::text(format!("Processed: {}", input.message))
    }
    
    // Add more tools here as needed:
    // fn another_tool(input: AnotherInput) -> ToolResponse {
    //     ToolResponse::text("Another tool response")
    // }
}