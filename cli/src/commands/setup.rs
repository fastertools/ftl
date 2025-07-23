use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct SetupArgs {
    #[command(subcommand)]
    pub command: SetupCommand,
}

#[derive(Debug, Subcommand)]
pub enum SetupCommand {
    /// Install and manage FTL templates
    Templates {
        /// Force reinstall even if templates exist
        #[arg(short, long)]
        force: bool,

        /// Install from a Git repository
        #[arg(long, conflicts_with_all = &["dir", "tar"])]
        git: Option<String>,

        /// Git branch to use
        #[arg(long, requires = "git")]
        branch: Option<String>,

        /// Install from a local directory
        #[arg(long, conflicts_with_all = &["git", "tar"])]
        dir: Option<PathBuf>,

        /// Install from a tarball
        #[arg(long, conflicts_with_all = &["git", "dir"])]
        tar: Option<String>,
    },

    /// Show FTL configuration info
    Info,
}

pub async fn execute(args: SetupArgs) -> Result<()> {
    let command = match args.command {
        SetupCommand::Templates {
            force,
            git,
            branch,
            dir,
            tar,
        } => ftl_commands::setup::SetupCommand::Templates {
            force,
            git,
            branch,
            dir,
            tar,
        },
        SetupCommand::Info => ftl_commands::setup::SetupCommand::Info,
    };

    let cmd_args = ftl_commands::setup::SetupArgs { command };

    ftl_commands::setup::execute(cmd_args).await
}
