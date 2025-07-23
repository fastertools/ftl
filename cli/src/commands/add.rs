use std::path::PathBuf;
use clap::Args;
use anyhow::Result;

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Name of the tool to add
    #[arg(value_name = "NAME")]
    pub name: Option<String>,
    
    /// Description of the tool
    #[arg(short, long)]
    pub description: Option<String>,
    
    /// Programming language
    #[arg(short, long)]
    pub language: Option<String>,
    
    /// Git repository URL for custom template
    #[arg(long, conflicts_with_all = &["dir", "tar"])]
    pub git: Option<String>,
    
    /// Git branch for custom template
    #[arg(long, requires = "git")]
    pub branch: Option<String>,
    
    /// Local directory path for custom template
    #[arg(long, conflicts_with_all = &["git", "tar"])]
    pub dir: Option<PathBuf>,
    
    /// Tarball path for custom template
    #[arg(long, conflicts_with_all = &["git", "dir"])]
    pub tar: Option<String>,
}

pub async fn execute(args: AddArgs) -> Result<()> {
    let cmd_args = ftl_commands::add::AddArgs {
        name: args.name,
        description: args.description,
        language: args.language,
        git: args.git,
        branch: args.branch,
        dir: args.dir,
        tar: args.tar,
    };
    
    ftl_commands::add::execute(cmd_args).await
}