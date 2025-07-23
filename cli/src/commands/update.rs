use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Force update even if already on latest version
    #[arg(short, long)]
    pub force: bool,
}

pub async fn execute(args: UpdateArgs) -> Result<()> {
    let cmd_args = ftl_commands::update::UpdateArgs { force: args.force };

    ftl_commands::update::execute(cmd_args).await
}
