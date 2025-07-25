use ftl_sdk::{ftl_tools, ToolResponse};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoRsInput {
    /// The input message to process
    message: String,
}

ftl_tools! {
    /// An MCP tool written in Rust
    fn echo_rs(input: EchoRsInput) -> ToolResponse {
        // TODO: Implement your tool logic here
        ToolResponse::text(format!("Processed: {}", input.message))
    }
}