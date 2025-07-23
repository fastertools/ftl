use clap::Args;
use anyhow::Result;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Name of the new project
    pub name: Option<String>,
    
    /// Initialize in current directory instead of creating new one
    #[arg(long)]
    pub here: bool,
}

pub async fn execute(args: InitArgs) -> Result<()> {
    let cmd_args = ftl_commands::init::InitArgs {
        name: args.name,
        here: args.here,
    };
    
    ftl_commands::init::execute(cmd_args).await
}