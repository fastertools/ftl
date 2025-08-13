//! Generate JSON schema for ftl.toml configuration

use crate::config::ftl_schema;
use anyhow::Result;

/// Generate and output the JSON schema for ftl.toml configuration
pub async fn execute(output: Option<String>) -> Result<()> {
    let schema_str = ftl_schema::generate_ftl_schema_string()?;
    
    if let Some(output_path) = output {
        std::fs::write(&output_path, &schema_str)?;
        println!("JSON schema written to: {}", output_path);
    } else {
        println!("{}", schema_str);
    }
    
    Ok(())
}