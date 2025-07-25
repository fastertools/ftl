use ftl_sdk::{tools, text};
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct EchoInput {
    /// The message to echo back
    message: String,
}

#[derive(Deserialize, JsonSchema)]
struct ReverseInput {
    /// The text to reverse
    text: String,
}

#[derive(Deserialize, JsonSchema)]
struct UppercaseInput {
    /// The text to convert to uppercase
    text: String,
}

#[derive(Deserialize, JsonSchema)]
struct WordCountInput {
    /// The text to count words in
    text: String,
}

tools! {
    /// Echo back the input message
    fn echo(input: EchoInput) -> ToolResponse {
        text!("Echo: {}", input.message)
    }
    
    /// Reverse the input text
    fn reverse(input: ReverseInput) -> ToolResponse {
        let reversed: String = input.text.chars().rev().collect();
        text!("{}", reversed)
    }
    
    /// Convert text to uppercase
    fn uppercase(input: UppercaseInput) -> ToolResponse {
        text!("{}", input.text.to_uppercase())
    }
    
    /// Count the number of words in the input text
    fn word_count(input: WordCountInput) -> ToolResponse {
        let count = input.text.split_whitespace().count();
        text!("Word count: {}", count)
    }
}