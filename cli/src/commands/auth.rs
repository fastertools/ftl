use clap::{Args, Subcommand};
use anyhow::Result;

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Show authentication status
    Status,
}

pub async fn execute(args: AuthArgs) -> Result<()> {
    let command = match args.command {
        AuthCommand::Status => ftl_commands::auth::AuthCommand::Status,
    };
    
    let cmd_args = ftl_commands::auth::AuthArgs {
        command,
    };
    
    ftl_commands::auth::execute(cmd_args).await
}