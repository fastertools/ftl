use ftl_sdk::{tools, text};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct ExampleToolInput {
    /// The input message to process
    message: String,
}

tools! {
    /// An example tool that processes messages
    fn example_tool(input: ExampleToolInput) -> ToolResponse {
        // TODO: Implement your tool logic here
        text!("Processed: {}", input.message)
    }
    
    // Add more tools here as needed:
    // fn another_tool(input: AnotherInput) -> ToolResponse {
    //     text!("Another tool response")
    // }
}