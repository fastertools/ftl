use ftl_sdk::{ftl_tools, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct ExampleToolInput {
    /// The input message to process
    message: String,
}

ftl_tools! {
    /// An example tool that processes messages
    fn example_tool(input: ExampleToolInput) -> ToolResponse {
        // TODO: Implement your tool logic here
        ToolResponse::text(format!("Processed: {}", input.message))
    }
    
    // Add more tools here as needed:
    // fn another_tool(input: AnotherInput) -> ToolResponse {
    //     ToolResponse::text("Another tool response")
    // }
}