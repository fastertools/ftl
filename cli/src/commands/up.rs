use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct UpArgs {
    /// Path to the Spin application
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Port to listen on
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Build before starting
    #[arg(short, long)]
    pub build: bool,

    /// Watch files and rebuild automatically
    #[arg(short, long)]
    pub watch: bool,

    /// Clear screen on rebuild (only with --watch)
    #[arg(short, long, requires = "watch")]
    pub clear: bool,
}

pub async fn execute(args: UpArgs) -> Result<()> {
    let cmd_args = ftl_commands::up::UpArgs {
        path: args.path,
        port: args.port,
        build: args.build,
        watch: args.watch,
        clear: args.clear,
    };

    ftl_commands::up::execute(cmd_args).await
}
