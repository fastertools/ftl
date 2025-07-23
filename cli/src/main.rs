//! FTL CLI - Build and deploy Model Context Protocol (MCP) tools on WebAssembly
//!
//! FTL (Faster Than Light) is a command-line tool for creating, building, and deploying
//! MCP tools as WebAssembly components.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new FTL project
    Init(commands::init::InitArgs),

    /// Build a Spin application
    Build(commands::build::BuildArgs),

    /// Deploy a Spin application
    Deploy(commands::deploy::DeployArgs),

    /// Start the Spin development server
    Up(commands::up::UpArgs),

    /// Publish a component to the registry
    Publish(commands::publish::PublishArgs),

    /// Authenticate with FTL platform
    Auth(commands::auth::AuthArgs),

    /// Set up FTL dependencies
    Setup(commands::setup::SetupArgs),

    /// Update FTL CLI to the latest version
    Update(commands::update::UpdateArgs),

    /// Add new components to your project
    Add(commands::add::AddArgs),

    /// Log in to FTL platform
    Login(commands::login::LoginArgs),

    /// Log out from FTL platform
    Logout(commands::logout::LogoutArgs),

    /// Run tests for your FTL project
    Test(commands::test::TestArgs),

    /// Manage applications
    App(commands::app::AppArgs),

    /// Manage FTL component registry
    Registry(commands::registry::RegistryArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => commands::init::execute(args).await,
        Commands::Build(args) => commands::build::execute(args).await,
        Commands::Deploy(args) => commands::deploy::execute(args).await,
        Commands::Up(args) => commands::up::execute(args).await,
        Commands::Publish(args) => commands::publish::execute(args).await,
        Commands::Auth(args) => commands::auth::execute(args).await,
        Commands::Setup(args) => commands::setup::execute(args).await,
        Commands::Update(args) => commands::update::execute(args).await,
        Commands::Add(args) => commands::add::execute(args).await,
        Commands::Login(args) => commands::login::execute(args).await,
        Commands::Logout(args) => commands::logout::execute(args).await,
        Commands::Test(args) => commands::test::execute(args).await,
        Commands::App(args) => commands::app::execute(args).await,
        Commands::Registry(args) => commands::registry::execute(args).await,
    }
}
