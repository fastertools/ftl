use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Path to the Spin application
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Registry to publish to
    #[arg(short, long)]
    pub registry: Option<String>,

    /// Version tag for the published package
    #[arg(short, long)]
    pub tag: Option<String>,
}

pub async fn execute(args: PublishArgs) -> Result<()> {
    let cmd_args = ftl_commands::publish::PublishArgs {
        path: args.path,
        registry: args.registry,
        tag: args.tag,
    };

    ftl_commands::publish::execute(cmd_args).await
}
