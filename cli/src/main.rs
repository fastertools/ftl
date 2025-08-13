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
    /// Initialize a new FTL project
    Init(InitArgs),
    /// Build a project
    Build(BuildArgs),
    /// Start the Spin development server
    Up(UpArgs),
    /// Set up FTL dependencies
    Setup(SetupArgs),
    /// Update FTL CLI to the latest version
    Update(UpdateArgs),
    /// Add new components to your project
    Add(AddArgs),
    /// Run tests for your project
    Test(TestArgs),
    /// Manage FTL component registry
    Registry(RegistryArgs),
    /// Manage remote tools on FTL Engine
    Eng(EngArgs),
    /// Manage WASM components
    Component(ComponentArgs),
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
    /// Path to the project
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
    /// Path to the project
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
struct EngArgs {
    #[command(subcommand)]
    command: EngCommand,
}

#[derive(Debug, Args)]
struct ComponentArgs {
    #[command(subcommand)]
    command: ComponentCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum ComponentCommand {
    /// Publish a component to a registry
    Publish {
        /// Path to component directory or WASM file
        path: PathBuf,
        /// Registry URL (e.g., "ghcr.io/myorg")
        #[arg(short, long)]
        registry: Option<String>,
        /// Component name (derives from path if not specified)
        #[arg(short, long)]
        name: Option<String>,
        /// Version tag (defaults to "latest")
        #[arg(short, long)]
        tag: Option<String>,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Pull a component from a registry
    Pull {
        /// Component reference (e.g., "mycomp:1.0.0" or "ghcr.io/org/comp:latest")
        component: String,
        /// Output path for the WASM file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Overwrite existing file
        #[arg(short, long)]
        force: bool,
    },
    /// List component versions in a registry
    List {
        /// Repository to list (e.g., "myorg/mycomponent")
        repository: String,
        /// Registry URL override
        #[arg(short, long)]
        registry: Option<String>,
    },
    /// Inspect a component's metadata
    Inspect {
        /// Component reference to inspect
        component: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum EngCommand {
    /// Log in to FTL Engine
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
    /// Log out from FTL Engine
    Logout,
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        command: EngAuthCommand,
    },
    /// Deploy an engine to FTL Engine
    Deploy {
        /// Variable(s) to be passed to the app
        #[arg(long, value_name = "KEY=VALUE")]
        variable: Vec<String>,

        /// Set access control mode (public, private, org, custom)
        /// Overrides `FTL_ACCESS_CONTROL` env var and ftl.toml `project.access_control`
        #[arg(
            long = "access-control",
            value_name = "MODE",
            help = "Access control: public (no auth), private (user only), org (organization), custom (BYO auth)",
            help_heading = "Authentication"
        )]
        access_control: Option<String>,

        /// JWT issuer URL (triggers custom auth mode)
        /// For complex OAuth configuration, use ftl.toml [oauth] section
        /// Overrides `FTL_JWT_ISSUER` env var and ftl.toml oauth.issuer
        #[arg(long, value_name = "URL", help_heading = "Authentication")]
        jwt_issuer: Option<String>,

        /// JWT audience (required when using --jwt-issuer for custom auth)
        /// Overrides `FTL_JWT_AUDIENCE` env var and ftl.toml oauth.audience
        #[arg(long, value_name = "AUDIENCE", help_heading = "Authentication")]
        jwt_audience: Option<String>,

        /// Allowed roles for organization mode (e.g., "admin,developer")
        /// Only users with these roles in the organization can access the app
        #[arg(
            long,
            value_name = "ROLES",
            value_delimiter = ',',
            help_heading = "Authentication"
        )]
        allowed_roles: Option<Vec<String>>,

        /// Run without making any changes (preview what would be deployed)
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// List all engines
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Get engine status
    Status {
        /// Engine ID or name
        engine_id: String,
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Delete an engine
    Delete {
        /// Engine ID or name
        engine_id: String,
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Get logs for an engine
    Logs {
        /// Engine ID or name
        engine_id: String,
        /// Time range for logs (e.g., "30m", "1h", "7d", RFC3339 timestamp, or Unix epoch)
        #[arg(short, long, default_value = "7d")]
        since: String,
        /// Number of log lines to retrieve (1-1000)
        #[arg(short, long, default_value = "100")]
        tail: u32,
        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: LogsOutputFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogsOutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Subcommand)]
enum EngAuthCommand {
    /// Show authentication status
    Status,
    /// Manage authentication tokens
    Token {
        #[command(subcommand)]
        command: EngAuthTokenCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum EngAuthTokenCommand {
    /// Output current user access token (for automation)
    Show,
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
    /// List configured registries or show current configuration
    List,
    /// Set the default registry
    Set {
        /// Registry URL (e.g., "ghcr.io/myorg")
        url: String,
    },
    /// Remove the default registry
    Remove,
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

// Eng command conversions
impl From<EngAuthCommand> for ftl_commands::auth::AuthCommand {
    fn from(cmd: EngAuthCommand) -> Self {
        match cmd {
            EngAuthCommand::Status => Self::Status,
            EngAuthCommand::Token { command } => Self::Token(command.into()),
        }
    }
}

impl From<EngAuthTokenCommand> for ftl_commands::auth::TokenCommand {
    fn from(cmd: EngAuthTokenCommand) -> Self {
        match cmd {
            EngAuthTokenCommand::Show => Self::Show,
        }
    }
}

impl From<OutputFormat> for ftl_commands::r#eng::OutputFormat {
    fn from(fmt: OutputFormat) -> Self {
        match fmt {
            OutputFormat::Table => Self::Table,
            OutputFormat::Json => Self::Json,
        }
    }
}

impl From<LogsOutputFormat> for ftl_commands::r#eng::LogsOutputFormat {
    fn from(fmt: LogsOutputFormat) -> Self {
        match fmt {
            LogsOutputFormat::Text => Self::Text,
            LogsOutputFormat::Json => Self::Json,
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

impl From<RegistryCommand> for ftl_commands::commands::registry::RegistryAction {
    fn from(cmd: RegistryCommand) -> Self {
        match cmd {
            RegistryCommand::List => Self::List,
            RegistryCommand::Set { url } => Self::Set { url },
            RegistryCommand::Remove => Self::Remove,
        }
    }
}

impl From<RegistryArgs> for ftl_commands::commands::registry::RegistryAction {
    fn from(args: RegistryArgs) -> Self {
        args.command.into()
    }
}

impl From<ComponentCommand> for ftl_commands::commands::component::ComponentAction {
    fn from(cmd: ComponentCommand) -> Self {
        match cmd {
            ComponentCommand::Publish {
                path,
                registry,
                name,
                tag,
                yes,
            } => Self::Publish {
                path,
                registry,
                name,
                tag,
                yes,
            },
            ComponentCommand::Pull {
                component,
                output,
                force,
            } => Self::Pull {
                component,
                output,
                force,
            },
            ComponentCommand::List {
                repository,
                registry,
            } => Self::List {
                repository,
                registry,
            },
            ComponentCommand::Inspect { component } => Self::Inspect { component },
        }
    }
}

impl From<ComponentArgs> for ftl_commands::commands::component::ComponentAction {
    fn from(args: ComponentArgs) -> Self {
        args.command.into()
    }
}

async fn handle_eng_command(args: EngArgs) -> Result<()> {
    match args.command {
        EngCommand::Login {
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
        EngCommand::Logout => {
            let logout_args = ftl_commands::logout::LogoutArgs {};
            ftl_commands::logout::execute(logout_args).await
        }
        EngCommand::Auth { command } => {
            let auth_args = ftl_commands::auth::AuthArgs {
                command: command.into(),
            };
            ftl_commands::auth::execute(auth_args).await
        }
        EngCommand::Deploy {
            variable,
            access_control,
            jwt_issuer,
            jwt_audience,
            allowed_roles,
            dry_run,
            yes,
        } => {
            let deploy_args = ftl_commands::deploy::DeployArgs {
                variables: variable,
                access_control,
                jwt_issuer,
                jwt_audience,
                allowed_roles,
                dry_run,
                yes,
            };
            ftl_commands::deploy::execute(deploy_args).await
        }
        EngCommand::List { format } => {
            let eng_args = ftl_commands::r#eng::EngineArgs {
                command: ftl_commands::r#eng::EngineCommand::List {
                    format: format.into(),
                },
            };
            ftl_commands::r#eng::execute(eng_args).await
        }
        EngCommand::Status { engine_id, format } => {
            let eng_args = ftl_commands::r#eng::EngineArgs {
                command: ftl_commands::r#eng::EngineCommand::Status {
                    app_id: engine_id,
                    format: format.into(),
                },
            };
            ftl_commands::r#eng::execute(eng_args).await
        }
        EngCommand::Delete { engine_id, force } => {
            let eng_args = ftl_commands::r#eng::EngineArgs {
                command: ftl_commands::r#eng::EngineCommand::Delete {
                    app_id: engine_id,
                    force,
                },
            };
            ftl_commands::r#eng::execute(eng_args).await
        }
        EngCommand::Logs {
            engine_id,
            since,
            tail,
            format,
        } => {
            let eng_args = ftl_commands::r#eng::EngineArgs {
                command: ftl_commands::r#eng::EngineCommand::Logs {
                    app_id: engine_id,
                    since,
                    tail,
                    format: format.into(),
                },
            };
            ftl_commands::r#eng::execute(eng_args).await
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

    // Execute the command
    match cli.command {
        Commands::Init(args) => ftl_commands::init::execute(args.into()).await,
        Commands::Build(args) => ftl_commands::build::execute(args.into()).await,
        Commands::Up(args) => ftl_commands::up::execute(args.into()).await,
        Commands::Setup(args) => ftl_commands::setup::execute(args.into()).await,
        Commands::Update(args) => ftl_commands::update::execute(args.into()).await,
        Commands::Add(args) => ftl_commands::add::execute(args.into()).await,
        Commands::Test(args) => ftl_commands::test::execute(args.into()).await,
        Commands::Registry(args) => ftl_commands::commands::registry::execute(args.into()),
        Commands::Eng(args) => handle_eng_command(args).await,
        Commands::Component(args) => ftl_commands::commands::component::execute(args.into()).await,
    }
}
