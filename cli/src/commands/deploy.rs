use clap::Args;
use anyhow::Result;

#[derive(Debug, Args)]
pub struct DeployArgs {
    // Deploy takes no arguments
}

pub async fn execute(_args: DeployArgs) -> Result<()> {
    let cmd_args = ftl_commands::deploy::DeployArgs {};
    ftl_commands::deploy::execute(cmd_args).await
}