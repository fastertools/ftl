use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct LogoutArgs {
    // No arguments for logout command
}

pub async fn execute(_args: LogoutArgs) -> Result<()> {
    let cmd_args = ftl_commands::logout::LogoutArgs {};
    ftl_commands::logout::execute(cmd_args).await
}
