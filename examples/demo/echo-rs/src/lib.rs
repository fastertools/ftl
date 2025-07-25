use ftl_sdk::{tools, text, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoRsInput {
    /// The input message to process
    message: String,
}

tools! {
    /// An MCP tool written in Rust
    fn echo_rs(input: EchoRsInput) -> ToolResponse {
        // TODO: Implement your tool logic here
        text!("Processed: {}", input.message)
    }
}