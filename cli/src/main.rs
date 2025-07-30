//! FTL CLI - Build and deploy Model Context Protocol (MCP) tools on WebAssembly

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new FTL toolbox
    Init(InitArgs),
    /// Build a toolbox
    Build(BuildArgs),
    /// Start the Spin development server
    Up(UpArgs),
    /// Publish a component to the registry
    Publish(PublishArgs),
    /// Set up FTL dependencies
    Setup(SetupArgs),
    /// Update FTL CLI to the latest version
    Update(UpdateArgs),
    /// Add new components to your toolbox
    Add(AddArgs),
    /// Run tests for your toolbox
    Test(TestArgs),
    /// Manage FTL component registry
    Registry(RegistryArgs),
    /// Manage pre-built tool components
    Tools(ToolsArgs),
    /// Manage remote tools on FTL Boxes
    Box(BoxArgs),
}

// Simple command wrappers - just forward arguments

#[derive(Debug, Args)]
struct InitArgs {
    /// Project name
    name: Option<String>,
    /// Initialize in current directory
    #[arg(long)]
    here: bool,
}

#[derive(Debug, Args)]
struct BuildArgs {
    /// Path to the toolbox
    #[arg(short, long)]
    path: Option<PathBuf>,
    /// Build in release mode
    #[arg(short, long)]
    release: bool,
    /// Export transpiled configuration (e.g., "spin")
    #[arg(long, value_name = "FORMAT")]
    export: Option<String>,
    /// Output path for exported configuration
    #[arg(long, value_name = "PATH", requires = "export")]
    export_out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct UpArgs {
    /// Path to the toolbox
    path: Option<PathBuf>,
    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,
    /// Build before starting
    #[arg(short, long)]
    build: bool,
    /// Watch files and rebuild automatically
    #[arg(short, long)]
    watch: bool,
    /// Clear screen on rebuild (only with --watch)
    #[arg(short, long, requires = "watch")]
    clear: bool,
    /// Directory for component logs (default: .ftl/logs)
    #[arg(long)]
    log_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct PublishArgs {
    /// Path to the component directory
    #[arg(short, long)]
    path: Option<PathBuf>,
    /// Container registry URL
    #[arg(short, long)]
    registry: Option<String>,
    /// Tag for the published image
    #[arg(short, long)]
    tag: Option<String>,
}

#[derive(Debug, Args)]
struct UpdateArgs {
    /// Force update even if already on latest version
    #[arg(short, long)]
    force: bool,
}

#[derive(Debug, Args)]
struct AddArgs {
    /// Name of the tool to add
    name: Option<String>,
    /// Programming language (rust, typescript, javascript)
    #[arg(short, long)]
    language: Option<String>,
    /// Git repository URL for custom templates
    #[arg(long, conflicts_with_all = ["dir", "tar"])]
    git: Option<String>,
    /// Git branch to use (only with --git)
    #[arg(long, requires = "git")]
    branch: Option<String>,
    /// Local directory path for custom templates
    #[arg(long, conflicts_with_all = ["git", "tar"])]
    dir: Option<PathBuf>,
    /// Tar file URL for custom templates
    #[arg(long, conflicts_with_all = ["git", "dir"])]
    tar: Option<String>,
}

#[derive(Debug, Args)]
struct TestArgs {
    /// Path to the project directory
    #[arg(short, long)]
    path: Option<PathBuf>,
}

// Complex commands with subcommands

#[derive(Debug, Args)]
struct BoxArgs {
    #[command(subcommand)]
    command: BoxCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum BoxCommand {
    /// Log in to FTL Boxes
    Login {
        /// Don't open browser automatically
        #[arg(long)]
        no_browser: bool,
        /// `AuthKit` domain (for testing)
        #[arg(long, hide = true)]
        authkit_domain: Option<String>,
        /// OAuth client ID (for testing)
        #[arg(long, hide = true)]
        client_id: Option<String>,
    },
    /// Log out from FTL Boxes
    Logout,
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        command: BoxAuthCommand,
    },
    /// Deploy a box to FTL Boxes
    Deploy {
        /// Variable(s) to be passed to the app
        #[arg(long, value_name = "KEY=VALUE")]
        variable: Vec<String>,
    },
    /// List all boxes
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Get box status
    Status {
        /// Box ID or name
        box_id: String,
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Delete a box
    Delete {
        /// Box ID or name
        box_id: String,
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum BoxAuthCommand {
    /// Show authentication status
    Status,
}

#[derive(Debug, Args)]
struct SetupArgs {
    #[command(subcommand)]
    command: SetupCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum SetupCommand {
    /// Show setup information
    Info,
    /// Install Spin templates
    Templates {
        /// Force reinstall even if already installed
        #[arg(short, long)]
        force: bool,
        /// Git repository URL
        #[arg(long, conflicts_with_all = ["dir", "tar"])]
        git: Option<String>,
        /// Git branch to use
        #[arg(long, requires = "git")]
        branch: Option<String>,
        /// Local directory path
        #[arg(long, conflicts_with_all = ["git", "tar"])]
        dir: Option<PathBuf>,
        /// Tar file URL
        #[arg(long, conflicts_with_all = ["git", "dir"])]
        tar: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Args)]
struct RegistryArgs {
    #[command(subcommand)]
    command: RegistryCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum RegistryCommand {
    /// Search for components in the registry
    Search {
        /// Search query
        query: String,
        /// Registry to search
        #[arg(short, long)]
        registry: Option<String>,
    },
    /// List available components
    List {
        /// Registry to list from
        #[arg(short, long)]
        registry: Option<String>,
    },
    /// Get information about a component
    Info {
        /// Component reference
        component: String,
    },
}

#[derive(Debug, Args)]
struct ToolsArgs {
    #[command(subcommand)]
    command: ToolsCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum ToolsCommand {
    /// List available pre-built tools
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
        /// Filter by keyword in name or description
        #[arg(short, long)]
        filter: Option<String>,
        /// Registry to use (overrides config)
        #[arg(short, long)]
        registry: Option<String>,
        /// Show additional details
        #[arg(short, long)]
        verbose: bool,
        /// List from all enabled registries
        #[arg(short, long)]
        all: bool,
        /// Query registry directly, skip manifest
        #[arg(short, long)]
        direct: bool,
    },
    /// Add pre-built tools to your project
    Add {
        /// Tool names to add (can include registry prefix like docker:tool-name)
        tools: Vec<String>,
        /// Registry to use (overrides config and tool prefix)
        #[arg(short, long)]
        registry: Option<String>,
        /// Version/tag to use (overrides tool:version syntax)
        #[arg(short, long)]
        version: Option<String>,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Update existing tools in your project
    Update {
        /// Tool names to update (can include registry prefix like docker:tool-name)
        tools: Vec<String>,
        /// Registry to use (overrides config and tool prefix)
        #[arg(short, long)]
        registry: Option<String>,
        /// Version/tag to update to (overrides tool:version syntax)
        #[arg(short, long)]
        version: Option<String>,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Remove tools from your project
    Remove {
        /// Tool names to remove
        tools: Vec<String>,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

// Conversion implementations

impl From<InitArgs> for ftl_commands::init::InitArgs {
    fn from(args: InitArgs) -> Self {
        Self {
            name: args.name,
            here: args.here,
        }
    }
}

impl From<BuildArgs> for ftl_commands::build::BuildArgs {
    fn from(args: BuildArgs) -> Self {
        Self {
            path: args.path,
            release: args.release,
            export: args.export,
            export_out: args.export_out,
        }
    }
}

impl From<UpArgs> for ftl_commands::up::UpArgs {
    fn from(args: UpArgs) -> Self {
        Self {
            path: args.path,
            port: args.port,
            build: args.build,
            watch: args.watch,
            clear: args.clear,
            log_dir: args.log_dir,
        }
    }
}

impl From<PublishArgs> for ftl_commands::publish::PublishArgs {
    fn from(args: PublishArgs) -> Self {
        Self {
            path: args.path,
            registry: args.registry,
            tag: args.tag,
        }
    }
}

impl From<UpdateArgs> for ftl_commands::update::UpdateArgs {
    fn from(args: UpdateArgs) -> Self {
        Self { force: args.force }
    }
}

impl From<AddArgs> for ftl_commands::add::AddArgs {
    fn from(args: AddArgs) -> Self {
        Self {
            name: args.name,
            language: args.language,
            git: args.git,
            branch: args.branch,
            dir: args.dir,
            tar: args.tar,
        }
    }
}

impl From<TestArgs> for ftl_commands::test::TestArgs {
    fn from(args: TestArgs) -> Self {
        Self { path: args.path }
    }
}

// Box command conversions
impl From<BoxAuthCommand> for ftl_commands::auth::AuthCommand {
    fn from(cmd: BoxAuthCommand) -> Self {
        match cmd {
            BoxAuthCommand::Status => Self::Status,
        }
    }
}

impl From<OutputFormat> for ftl_commands::r#box::OutputFormat {
    fn from(fmt: OutputFormat) -> Self {
        match fmt {
            OutputFormat::Table => Self::Table,
            OutputFormat::Json => Self::Json,
        }
    }
}

impl From<SetupCommand> for ftl_commands::setup::SetupCommand {
    fn from(cmd: SetupCommand) -> Self {
        match cmd {
            SetupCommand::Info => Self::Info,
            SetupCommand::Templates {
                force,
                git,
                branch,
                dir,
                tar,
            } => Self::Templates {
                force,
                git,
                branch,
                dir,
                tar,
            },
        }
    }
}

impl From<SetupArgs> for ftl_commands::setup::SetupArgs {
    fn from(args: SetupArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

impl From<RegistryCommand> for ftl_commands::registry_command::RegistryCommand {
    fn from(cmd: RegistryCommand) -> Self {
        match cmd {
            RegistryCommand::Search { query, registry } => Self::Search { query, registry },
            RegistryCommand::List { registry } => Self::List { registry },
            RegistryCommand::Info { component } => Self::Info { component },
        }
    }
}

impl From<RegistryArgs> for ftl_commands::registry_command::RegistryArgs {
    fn from(args: RegistryArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

impl From<ToolsCommand> for ftl_commands::tools::ToolsCommand {
    fn from(cmd: ToolsCommand) -> Self {
        match cmd {
            ToolsCommand::List {
                category,
                filter,
                registry,
                verbose,
                all,
                direct,
            } => Self::List {
                category,
                filter,
                registry,
                verbose,
                all,
                direct,
            },
            ToolsCommand::Add {
                tools,
                registry,
                version,
                yes,
            } => Self::Add {
                tools,
                registry,
                version,
                yes,
            },
            ToolsCommand::Update {
                tools,
                registry,
                version,
                yes,
            } => Self::Update {
                tools,
                registry,
                version,
                yes,
            },
            ToolsCommand::Remove { tools, yes } => Self::Remove { tools, yes },
        }
    }
}

impl From<ToolsArgs> for ftl_commands::tools::ToolsArgs {
    fn from(args: ToolsArgs) -> Self {
        Self {
            command: args.command.into(),
        }
    }
}

async fn handle_box_command(args: BoxArgs) -> Result<()> {
    match args.command {
        BoxCommand::Login {
            no_browser,
            authkit_domain,
            client_id,
        } => {
            let login_args = ftl_commands::login::LoginArgs {
                no_browser,
                authkit_domain,
                client_id,
            };
            ftl_commands::login::execute(login_args).await
        }
        BoxCommand::Logout => {
            let logout_args = ftl_commands::logout::LogoutArgs {};
            ftl_commands::logout::execute(logout_args).await
        }
        BoxCommand::Auth { command } => {
            let auth_args = ftl_commands::auth::AuthArgs {
                command: command.into(),
            };
            ftl_commands::auth::execute(auth_args).await
        }
        BoxCommand::Deploy { variable } => {
            let deploy_args = ftl_commands::deploy::DeployArgs {
                variables: variable,
            };
            ftl_commands::deploy::execute(deploy_args).await
        }
        BoxCommand::List { format } => {
            let box_args = ftl_commands::r#box::BoxArgs {
                command: ftl_commands::r#box::BoxCommand::List {
                    format: format.into(),
                },
            };
            ftl_commands::r#box::execute(box_args).await
        }
        BoxCommand::Status { box_id, format } => {
            let box_args = ftl_commands::r#box::BoxArgs {
                command: ftl_commands::r#box::BoxCommand::Status {
                    app_id: box_id,
                    format: format.into(),
                },
            };
            ftl_commands::r#box::execute(box_args).await
        }
        BoxCommand::Delete { box_id, force } => {
            let box_args = ftl_commands::r#box::BoxArgs {
                command: ftl_commands::r#box::BoxCommand::Delete {
                    app_id: box_id,
                    force,
                },
            };
            ftl_commands::r#box::execute(box_args).await
        }
    }
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
        Commands::Init(args) => ftl_commands::init::execute(args.into()).await,
        Commands::Build(args) => ftl_commands::build::execute(args.into()).await,
        Commands::Up(args) => ftl_commands::up::execute(args.into()).await,
        Commands::Publish(args) => ftl_commands::publish::execute(args.into()).await,
        Commands::Setup(args) => ftl_commands::setup::execute(args.into()).await,
        Commands::Update(args) => ftl_commands::update::execute(args.into()).await,
        Commands::Add(args) => ftl_commands::add::execute(args.into()).await,
        Commands::Test(args) => ftl_commands::test::execute(args.into()).await,
        Commands::Registry(args) => ftl_commands::registry_command::execute(args.into()).await,
        Commands::Tools(args) => ftl_commands::tools::execute(args.into()).await,
        Commands::Box(args) => handle_box_command(args).await,
    }
}
