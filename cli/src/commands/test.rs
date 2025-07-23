use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct TestArgs {
    /// Path to the project or tool
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

pub async fn execute(args: TestArgs) -> Result<()> {
    let cmd_args = ftl_commands::test::TestArgs { path: args.path };

    ftl_commands::test::execute(cmd_args).await
}
