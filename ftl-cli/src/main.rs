use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

mod commands;
mod common;
mod manifest;
mod spin_generator;
mod templates;


#[derive(Parser)]
#[command(name = "ftl")]
#[command(about = "FTL - WebAssembly MCP tools")]
#[command(version)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    
    /// Increase logging verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new tool from template
    New {
        /// Name of the tool
        name: String,
        /// Description of the tool
        #[arg(short, long)]
        description: Option<String>,
    },
    
    /// Build a tool
    Build {
        /// Name of the tool to build (defaults to current directory)
        name: Option<String>,
        /// Build profile to use
        #[arg(short, long)]
        profile: Option<String>,
        /// Start serving after build completes
        #[arg(short, long)]
        serve: bool,
    },
    
    /// Serve a tool locally
    Serve {
        /// Name of the tool to serve (defaults to current directory)
        name: Option<String>,
        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Build before serving
        #[arg(short, long)]
        build: bool,
    },
    
    /// Run tests for a tool
    Test {
        /// Name of the tool to test (defaults to current directory)
        name: Option<String>,
    },
    
    /// Deploy a tool
    Deploy {
        /// Name of the tool to deploy (defaults to current directory)
        name: Option<String>,
    },
    
    /// Export a tool as a standalone WASM component
    Export {
        /// Name of the tool to export (defaults to current directory)
        name: Option<String>,
        /// Output path for the component WASM file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Build profile to use
        #[arg(short, long)]
        profile: Option<String>,
    },
    
    /// Watch a tool for changes and rebuild
    Watch {
        /// Name of the tool to watch (defaults to current directory)
        name: Option<String>,
    },
    
    /// Validate tool configuration
    Validate {
        /// Name of the tool to validate (defaults to current directory)
        name: Option<String>,
    },
    
    /// Show binary size information
    Size {
        /// Name of the tool (defaults to current directory)
        name: Option<String>,
        /// Show detailed analysis including sections and imports
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// List deployed tools and toolkits
    List,
    
    /// Login to FTL Edge
    Login,
    
    /// Get status of a deployed tool or toolkit
    Status {
        /// Name of the tool or toolkit (defaults to current directory)
        name: Option<String>,
    },
    
    /// Delete a deployed tool or toolkit
    Delete {
        /// Name of the tool or toolkit (defaults to current directory)
        name: Option<String>,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    
    /// View logs from a deployed tool or toolkit
    Logs {
        /// Name of the tool or toolkit (defaults to current directory)
        name: Option<String>,
        /// Follow log output (not yet supported by spin)
        #[arg(short, long, hide = true)]
        follow: bool,
        /// Number of lines to show from the end
        #[arg(short, long)]
        tail: Option<usize>,
    },
    
    /// Link current tool to an existing deployment
    Link {
        /// Name of the deployed tool to link to
        name: String,
        /// Path to the tool directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Unlink current tool from its deployment
    Unlink {
        /// Path to the tool directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Manage toolkits (collections of tools)
    Toolkit {
        #[command(subcommand)]
        command: ToolkitCommand,
    },
}

#[derive(Subcommand)]
enum ToolkitCommand {
    /// Build a toolkit from multiple tools
    Build {
        /// Name of the toolkit
        #[arg(long)]
        name: String,
        /// Tools to include in the toolkit
        tools: Vec<String>,
    },
    
    /// Serve a toolkit locally
    Serve {
        /// Name of the toolkit
        name: String,
        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    
    /// Deploy a toolkit to FTL Edge
    Deploy {
        /// Name of the toolkit
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let filter = match cli.verbose {
        0 => EnvFilter::new("error"),
        1 => EnvFilter::new("warn"),
        2 => EnvFilter::new("info"),
        3 => EnvFilter::new("debug"),
        _ => EnvFilter::new("trace"),
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    match cli.command {
        Command::New { name, description } => {
            commands::new::execute(name, description).await
        }
        Command::Build { name, profile, serve } => {
            if serve {
                commands::build::execute_and_serve(name, profile).await
            } else {
                commands::build::execute(name, profile).await
            }
        }
        Command::Serve { name, port, build } => {
            commands::serve::execute(name.unwrap_or_else(|| ".".to_string()), port, build).await
        }
        Command::Test { name } => {
            commands::test::execute(name).await
        }
        Command::Deploy { name } => {
            commands::deploy::execute(name.unwrap_or_else(|| ".".to_string())).await
        }
        Command::Export { name, output, profile } => {
            commands::export::execute(name, output, profile).await
        }
        Command::Watch { name } => {
            commands::watch::execute(name.unwrap_or_else(|| ".".to_string())).await
        }
        Command::Validate { name } => {
            commands::validate::execute(name.unwrap_or_else(|| ".".to_string())).await
        }
        Command::Size { name, verbose } => {
            commands::size::execute(name.unwrap_or_else(|| ".".to_string()), verbose).await
        }
        Command::List => {
            commands::list::execute().await
        }
        Command::Login => {
            commands::login::execute().await
        }
        Command::Status { name } => {
            commands::status::execute(name).await
        }
        Command::Delete { name, yes } => {
            commands::delete::execute(name, yes).await
        }
        Command::Logs { name, follow, tail } => {
            commands::logs::execute(name, follow, tail).await
        }
        Command::Link { name, path } => {
            commands::link::execute(name, path).await
        }
        Command::Unlink { path } => {
            commands::unlink::execute(path).await
        }
        Command::Toolkit { command } => {
            match command {
                ToolkitCommand::Build { name, tools } => {
                    commands::toolkit::build(name, tools).await
                }
                ToolkitCommand::Serve { name, port } => {
                    commands::toolkit::serve(name, port).await
                }
                ToolkitCommand::Deploy { name } => {
                    commands::toolkit::deploy(name).await
                }
            }
        }
    }
}

