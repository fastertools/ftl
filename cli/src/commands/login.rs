use clap::Args;
use anyhow::Result;

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Don't open browser automatically
    #[arg(long)]
    pub no_browser: bool,
    
    /// AuthKit domain (for testing)
    #[arg(long, hide = true)]
    pub authkit_domain: Option<String>,
    
    /// OAuth client ID (for testing)
    #[arg(long, hide = true)]
    pub client_id: Option<String>,
}

pub async fn execute(args: LoginArgs) -> Result<()> {
    let cmd_args = ftl_commands::login::LoginArgs {
        no_browser: args.no_browser,
        authkit_domain: args.authkit_domain,
        client_id: args.client_id,
    };
    
    ftl_commands::login::execute(cmd_args).await
}