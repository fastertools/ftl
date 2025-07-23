use clap::{Args, Subcommand};
use anyhow::Result;

#[derive(Debug, Args)]
pub struct RegistryArgs {
    #[command(subcommand)]
    pub command: RegistryCommand,
}

#[derive(Debug, Subcommand)]
pub enum RegistryCommand {
    /// List available components
    List {
        /// Registry URL
        #[arg(short, long)]
        registry: Option<String>,
    },
    
    /// Search for components
    Search {
        /// Search query
        query: String,
        
        /// Registry URL
        #[arg(short, long)]
        registry: Option<String>,
    },
    
    /// Get info about a component
    Info {
        /// Component name
        component: String,
    },
}

pub async fn execute(args: RegistryArgs) -> Result<()> {
    let command = match args.command {
        RegistryCommand::List { registry } => {
            ftl_commands::registry::RegistryCommand::List { registry }
        }
        RegistryCommand::Search { query, registry } => {
            ftl_commands::registry::RegistryCommand::Search { query, registry }
        }
        RegistryCommand::Info { component } => {
            ftl_commands::registry::RegistryCommand::Info { component }
        }
    };
    
    let cmd_args = ftl_commands::registry::RegistryArgs { command };
    
    ftl_commands::registry::execute(cmd_args).await
}