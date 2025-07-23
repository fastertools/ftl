use clap::Args;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Path to the Spin application
    pub path: Option<PathBuf>,
    
    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,
}

pub async fn execute(args: BuildArgs) -> Result<()> {
    let cmd_args = ftl_commands::build::BuildArgs {
        path: args.path,
        release: args.release,
    };
    
    ftl_commands::build::execute(cmd_args).await
}