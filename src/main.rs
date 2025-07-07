use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;
mod common;
mod language;
mod manifest;

#[derive(Parser)]
#[command(name = "ftl")]
#[command(about = "Build and deploy Model Context Protocol (MCP) servers on WebAssembly")]
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
    /// Initialize a new MCP project
    Init {
        /// Name of the project
        name: Option<String>,

        /// Create in current directory
        #[arg(long)]
        here: bool,
    },

    /// Add MCP server to the current project
    Add {
        /// Name of the MCP server
        name: Option<String>,

        /// MCP server description
        #[arg(short, long)]
        description: Option<String>,

        /// Language (rust, typescript, javascript, etc.)
        #[arg(short, long)]
        language: Option<String>,

        /// HTTP route for the MCP server
        #[arg(short, long)]
        route: Option<String>,

        /// Use a Git repository as the template source
        #[arg(long, conflicts_with = "dir", conflicts_with = "tar")]
        git: Option<String>,

        /// The branch to use from the Git repository
        #[arg(long, requires = "git")]
        branch: Option<String>,

        /// Use a local directory as the template source
        #[arg(long, conflicts_with = "git", conflicts_with = "tar")]
        dir: Option<PathBuf>,

        /// Use a tarball as the template source
        #[arg(long, conflicts_with = "git", conflicts_with = "dir")]
        tar: Option<String>,
    },

    /// Build the MCP server or project
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Path to MCP server (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Run the project locally
    Up {
        /// Build before running
        #[arg(long)]
        build: bool,

        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Path to MCP server (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Build and run the project, rebuilding on file changes
    Watch {
        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Path to MCP server (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Run tests
    Test {
        /// Path to MCP server (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Publish MCP server to registry
    Publish {
        /// Registry URL (defaults to ghcr.io)
        #[arg(short, long)]
        registry: Option<String>,

        /// Tag/version to publish
        #[arg(short, long)]
        tag: Option<String>,

        /// Path to MCP server (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Deploy the project to FTL
    Deploy {
        /// Environment to deploy to
        #[arg(short, long)]
        environment: Option<String>,
    },

    /// Interact with MCP server registries
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },

    /// Setup and configure FTL
    Setup {
        #[command(subcommand)]
        command: SetupCommand,
    },

    /// Update FTL CLI to the latest version
    Update {
        /// Force reinstall even if already latest version
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum SetupCommand {
    /// Install or update wasmcp templates
    Templates {
        /// Force reinstall even if already installed
        #[arg(long)]
        force: bool,

        /// Use a Git repository as the template source
        #[arg(long, conflicts_with = "dir", conflicts_with = "tar")]
        git: Option<String>,

        /// The branch to use from the Git repository
        #[arg(long, requires = "git")]
        branch: Option<String>,

        /// Use a local directory as the template source
        #[arg(long, conflicts_with = "git", conflicts_with = "tar")]
        dir: Option<PathBuf>,

        /// Use a tarball as the template source
        #[arg(long, conflicts_with = "git", conflicts_with = "dir")]
        tar: Option<String>,
    },

    /// Show current configuration
    Info,
}

#[derive(Subcommand)]
enum RegistryCommand {
    /// List available MCP servers
    List {
        /// Registry to list from
        #[arg(short, long)]
        registry: Option<String>,
    },

    /// Search for MCP servers
    Search {
        /// Search query
        query: String,

        /// Registry to search in
        #[arg(short, long)]
        registry: Option<String>,
    },

    /// Show MCP server information
    Info {
        /// MCP server name or URL
        component: String,
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

    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Check for updates (non-blocking, once per day)
    if let Err(e) = common::version_cache::check_and_prompt_for_update().await {
        tracing::debug!("Version check failed: {}", e);
    }

    match cli.command {
        Command::Init { name, here } => commands::init::execute(name, here).await,
        Command::Add {
            name,
            description,
            language,
            route,
            git,
            branch,
            dir,
            tar,
        } => {
            commands::add::execute(commands::add::AddOptions {
                name,
                description,
                language,
                route,
                git,
                branch,
                dir,
                tar,
            })
            .await
        }
        Command::Build { release, path } => commands::build::execute(path, release).await,
        Command::Up { build, port, path } => commands::up::execute(path, port, build).await,
        Command::Watch { port, path } => commands::watch::execute(path, port).await,
        Command::Test { path } => commands::test::execute(path).await,
        Command::Publish {
            registry,
            tag,
            path,
        } => commands::publish::execute(path, registry, tag).await,
        Command::Deploy { environment } => commands::deploy::execute(environment).await,
        Command::Registry { command } => match command {
            RegistryCommand::List { registry } => commands::registry::list(registry).await,
            RegistryCommand::Search { query, registry } => {
                commands::registry::search(query, registry).await
            }
            RegistryCommand::Info { component } => commands::registry::info(component).await,
        },
        Command::Setup { command } => match command {
            SetupCommand::Templates {
                force,
                git,
                branch,
                dir,
                tar,
            } => commands::setup::templates(force, git, branch, dir, tar).await,
            SetupCommand::Info => commands::setup::info().await,
        },
        Command::Update { force } => commands::update::execute(force).await,
    }
}
