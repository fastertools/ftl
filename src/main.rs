//! FTL CLI - Build and deploy Model Context Protocol (MCP) tools on WebAssembly
//!
//! FTL (Faster Than Light) is a command-line tool for creating, building, and deploying
//! MCP tools as WebAssembly components. It provides a complete development workflow
//! for building serverless MCP tools that can be deployed to the FTL platform.
//!
//! # Features
//!
//! - Project initialization and scaffolding
//! - Multi-language support (Rust, TypeScript/JavaScript)
//! - Local development server with hot reload
//! - WebAssembly component building and packaging
//! - Deployment to FTL cloud platform
//! - Component registry integration

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod api_client;
mod commands;
mod common;
mod config;
mod deps;
mod language;
mod ui;

use deps::CredentialsProvider;

// Implementations for up command dependencies
struct RealFileWatcher;

#[async_trait::async_trait]
impl commands::up::FileWatcher for RealFileWatcher {
    async fn watch(
        &self,
        path: &Path,
        recursive: bool,
    ) -> Result<Box<dyn commands::up::WatchHandle>> {
        use notify::{RecommendedWatcher, RecursiveMode, Watcher};
        use tokio::sync::mpsc;

        let (tx, rx) = mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.try_send(event);
                }
            },
            notify::Config::default(),
        )?;

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        watcher.watch(path, mode)?;

        Ok(Box::new(RealWatchHandle {
            _watcher: Some(watcher),
            rx,
        }))
    }
}

struct RealWatchHandle {
    _watcher: Option<notify::RecommendedWatcher>,
    rx: tokio::sync::mpsc::Receiver<notify::Event>,
}

#[async_trait::async_trait]
impl commands::up::WatchHandle for RealWatchHandle {
    async fn wait_for_change(&mut self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        while let Some(event) = self.rx.recv().await {
            paths.extend(event.paths);

            // Drain any additional events that arrived
            while let Ok(event) = self.rx.try_recv() {
                paths.extend(event.paths);
            }

            if !paths.is_empty() {
                break;
            }
        }

        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

struct RealSignalHandler;

#[async_trait::async_trait]
impl commands::up::SignalHandler for RealSignalHandler {
    async fn wait_for_interrupt(&self) -> Result<()> {
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to listen for Ctrl+C: {}", e))
    }
}

// Implementations for test command dependencies
struct RealDirectoryReader;

impl commands::test::DirectoryReader for RealDirectoryReader {
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        std::fs::read_dir(path)?
            .map(|entry| Ok(entry?.path()))
            .collect()
    }

    fn is_dir(&self, path: &Path) -> Result<bool> {
        Ok(path.is_dir())
    }
}

struct RealFileChecker;

impl commands::test::FileChecker for RealFileChecker {
    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }
}

struct RealTestCommandExecutor;

impl commands::test::TestCommandExecutor for RealTestCommandExecutor {
    fn execute(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&str>,
    ) -> Result<std::process::Output> {
        use std::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args);
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        cmd.output()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", command, e))
    }
}

// Implementations for publish command dependencies
struct RealProcessExecutor;

impl commands::publish::ProcessExecutor for RealProcessExecutor {
    fn execute(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<commands::publish::ProcessOutput> {
        use std::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args);
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", command, e))?;

        Ok(commands::publish::ProcessOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

struct RealSpinInstallerForPublish;

#[async_trait::async_trait]
impl commands::publish::SpinInstaller for RealSpinInstallerForPublish {
    async fn check_and_install_spin(&self) -> Result<PathBuf> {
        crate::common::spin_installer::check_and_install_spin().await
    }
}

struct RealBuildExecutorForPublish;

#[async_trait::async_trait]
impl commands::publish::BuildExecutor for RealBuildExecutorForPublish {
    async fn execute(&self, path: Option<PathBuf>, release: bool) -> Result<()> {
        // Delegate to build_v2
        let ui = Arc::new(ui::RealUserInterface);
        let deps = Arc::new(commands::build::BuildDependencies {
            file_system: Arc::new(deps::RealFileSystem),
            command_executor: Arc::new(deps::RealCommandExecutor),
            ui: ui.clone(),
            spin_installer: Arc::new(deps::RealSpinInstaller),
        });
        commands::build::execute_with_deps(commands::build::BuildConfig { path, release }, deps)
            .await
    }
}

// Implementations for setup command dependencies
struct RealSpinInstallerForSetup;

impl commands::setup::SpinInstaller for RealSpinInstallerForSetup {
    fn check_and_install(&self) -> Result<PathBuf> {
        // Blocking implementation since trait is sync
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(crate::common::spin_installer::check_and_install_spin())
        })
    }

    fn get_spin_path(&self) -> Result<PathBuf> {
        // Try common locations
        let paths = [
            dirs::home_dir().map(|d| d.join(".cargo/bin/spin")),
            Some(PathBuf::from("/usr/local/bin/spin")),
            Some(PathBuf::from("/usr/bin/spin")),
        ];

        for path_opt in paths.iter().flatten() {
            if path_opt.exists() {
                return Ok(path_opt.clone());
            }
        }

        // Try which
        which::which("spin").map_err(|_| anyhow::anyhow!("Spin not found in PATH"))
    }
}

struct RealSetupCommandExecutor;

impl commands::setup::SetupCommandExecutor for RealSetupCommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<std::process::Output> {
        use std::process::Command;

        Command::new(command)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", command, e))
    }
}

struct RealEnvironmentForSetup;

impl commands::setup::Environment for RealEnvironmentForSetup {
    fn get_cargo_pkg_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

// Implementations for update command dependencies
struct RealHttpClientForUpdate;

#[async_trait::async_trait]
impl commands::update::HttpClient for RealHttpClientForUpdate {
    async fn get(&self, url: &str, user_agent: &str) -> Result<String> {
        let response = reqwest::Client::new()
            .get(url)
            .header("User-Agent", user_agent)
            .send()
            .await?;

        response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))
    }
}

struct RealCommandExecutorForUpdate;

impl commands::update::CommandExecutor for RealCommandExecutorForUpdate {
    fn execute(&self, command: &str, args: &[&str]) -> Result<commands::update::CommandOutput> {
        use std::process::Command;

        let output = Command::new(command)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", command, e))?;

        Ok(commands::update::CommandOutput {
            success: output.status.success(),
            stderr: output.stderr,
        })
    }
}

struct RealEnvironmentForUpdate;

impl commands::update::Environment for RealEnvironmentForUpdate {
    fn get_cargo_pkg_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

// Implementations for login command dependencies
struct RealHttpClient;

#[async_trait::async_trait]
impl commands::login::HttpClient for RealHttpClient {
    async fn post(&self, url: &str, body: &str) -> Result<commands::login::HttpResponse> {
        let response = reqwest::Client::new()
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string())
            .send()
            .await?;

        let status = response.status().as_u16();
        let body = response.text().await?;

        Ok(commands::login::HttpResponse { status, body })
    }
}

struct RealKeyringStorage;

impl commands::login::KeyringStorage for RealKeyringStorage {
    fn store(&self, service: &str, username: &str, password: &str) -> Result<()> {
        let entry = keyring::Entry::new(service, username)?;
        entry.set_password(password)?;
        Ok(())
    }

    fn retrieve(&self, service: &str, username: &str) -> Result<String> {
        let entry = keyring::Entry::new(service, username)?;
        entry
            .get_password()
            .map_err(|e| anyhow::anyhow!("Failed to retrieve credentials: {}", e))
    }

    fn delete(&self, service: &str, username: &str) -> Result<()> {
        let entry = keyring::Entry::new(service, username)?;
        entry.delete_credential()?;
        Ok(())
    }
}

struct RealBrowserLauncher;

impl commands::login::BrowserLauncher for RealBrowserLauncher {
    fn open(&self, url: &str) -> Result<()> {
        webbrowser::open(url).map_err(|e| anyhow::anyhow!("Failed to open browser: {}", e))
    }
}

struct RealClockForLogin;

impl commands::login::Clock for RealClockForLogin {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    fn instant_now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}

// Implementations for logout command dependencies
struct RealCredentialsClearer;

impl commands::logout::CredentialsClearer for RealCredentialsClearer {
    fn clear_stored_credentials(&self) -> Result<()> {
        commands::login::clear_stored_credentials()
    }
}

// Implementations for auth command dependencies
struct RealCredentialsProviderForAuth;

impl commands::auth::CredentialsProvider for RealCredentialsProviderForAuth {
    fn get_stored_credentials(&self) -> Result<deps::StoredCredentials> {
        use crate::commands::login::get_stored_credentials;
        get_stored_credentials()?.ok_or_else(|| anyhow::anyhow!("No matching entry found"))
    }
}

struct RealClockForAuth;

impl commands::auth::Clock for RealClockForAuth {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

#[cfg(test)]
mod test_helpers;

#[derive(Parser)]
#[command(name = "ftl")]
#[command(about = "Build and deploy Model Context Protocol (MCP) tools on WebAssembly")]
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

    /// Add MCP tool to the current project
    Add {
        /// Name of the MCP tool
        name: Option<String>,

        /// MCP tool description
        #[arg(short, long)]
        description: Option<String>,

        /// Language (rust, typescript, javascript, etc.)
        #[arg(short, long)]
        language: Option<String>,

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

    /// Build the MCP tools or project
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Path to project (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Run the project locally
    Up {
        /// Build before running
        #[arg(long)]
        build: bool,

        /// Watch for file changes and rebuild automatically
        #[arg(long)]
        watch: bool,

        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Path to project (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,

        /// Clear the screen before each rebuild (only with --watch)
        #[arg(short, long, requires = "watch")]
        clear: bool,
    },

    /// Run tests
    Test {
        /// Path to project (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Publish MCP tool to registry
    Publish {
        /// Registry URL (defaults to ghcr.io)
        #[arg(short, long)]
        registry: Option<String>,

        /// Tag/version to publish
        #[arg(short, long)]
        tag: Option<String>,

        /// Path to project (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Deploy the project to FTL
    Deploy,

    /// Manage FTL applications
    App {
        #[command(subcommand)]
        command: AppCommand,
    },

    /// Interact with MCP tool registries
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

    /// Authenticate with FTL
    Login {
        /// Don't open browser automatically
        #[arg(long)]
        no_browser: bool,
    },

    /// Log out of FTL
    Logout,

    /// Authentication status and management
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
}

#[derive(Subcommand)]
enum AppCommand {
    /// List all applications
    List {
        /// Output format (table or json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Show application status
    Status {
        /// Application name
        name: String,
        /// Output format (table or json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Delete an application
    Delete {
        /// Application name
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum SetupCommand {
    /// Install or update ftl-mcp templates
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
    /// List available MCP tools
    List {
        /// Registry to list from
        #[arg(short, long)]
        registry: Option<String>,
    },

    /// Search for MCP tools
    Search {
        /// Search query
        query: String,

        /// Registry to search in
        #[arg(short, long)]
        registry: Option<String>,
    },

    /// Show MCP tool information
    Info {
        /// MCP tool name or URL
        component: String,
    },
}

#[derive(Subcommand)]
enum AuthCommand {
    /// Show authentication status
    Status,
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
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
        Command::Init { name, here } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::init::InitDependencies {
                file_system: Arc::new(deps::RealFileSystem),
                command_executor: Arc::new(deps::RealCommandExecutor),
                ui: ui.clone(),
                spin_installer: Arc::new(deps::RealSpinInstaller),
            });

            // Execute with v2
            commands::init::execute_with_deps(commands::init::InitConfig { name, here }, deps).await
        }
        Command::Add {
            name,
            description,
            language,
            git,
            branch,
            dir,
            tar,
        } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::add::AddDependencies {
                file_system: Arc::new(deps::RealFileSystem) as Arc<dyn deps::FileSystem>,
                command_executor: Arc::new(deps::RealCommandExecutor)
                    as Arc<dyn deps::CommandExecutor>,
                ui: ui.clone() as Arc<dyn deps::UserInterface>,
                spin_installer: Arc::new(deps::RealSpinInstaller) as Arc<dyn deps::SpinInstaller>,
            });

            // Execute with v2
            commands::add::execute_with_deps(
                commands::add::AddConfig {
                    name,
                    description,
                    language,
                    git,
                    branch,
                    dir,
                    tar,
                },
                deps,
            )
            .await
        }
        Command::Build { release, path } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::build::BuildDependencies {
                file_system: Arc::new(deps::RealFileSystem),
                command_executor: Arc::new(deps::RealCommandExecutor),
                ui: ui.clone(),
                spin_installer: Arc::new(deps::RealSpinInstaller),
            });

            // Execute with v2
            commands::build::execute_with_deps(commands::build::BuildConfig { path, release }, deps)
                .await
        }
        Command::Up {
            build,
            watch,
            port,
            path,
            clear,
        } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::up::UpDependencies {
                file_system: Arc::new(deps::RealFileSystem),
                command_executor: Arc::new(deps::RealCommandExecutor),
                ui: ui.clone(),
                spin_installer: Arc::new(deps::RealSpinInstaller),
                async_runtime: Arc::new(deps::RealAsyncRuntime),
                process_manager: Arc::new(deps::RealProcessManager),
                file_watcher: Arc::new(RealFileWatcher),
                signal_handler: Arc::new(RealSignalHandler),
            });

            // Execute with v2
            commands::up::execute_with_deps(
                commands::up::UpConfig {
                    path,
                    port,
                    build,
                    watch,
                    clear,
                },
                deps,
            )
            .await
        }
        Command::Test { path } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::test::TestDependencies {
                ui: ui.clone(),
                directory_reader: Arc::new(RealDirectoryReader),
                file_checker: Arc::new(RealFileChecker),
                command_executor: Arc::new(RealTestCommandExecutor),
            });

            // Execute with v2
            commands::test::execute_with_deps(path, &deps)
        }
        Command::Publish {
            registry,
            tag,
            path,
        } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::publish::PublishDependencies {
                ui: ui.clone(),
                file_system: Arc::new(deps::RealFileSystem),
                process_executor: Arc::new(RealProcessExecutor),
                spin_installer: Arc::new(RealSpinInstallerForPublish),
                build_executor: Arc::new(RealBuildExecutorForPublish),
            });

            // Execute with v2
            commands::publish::execute_with_deps(
                commands::publish::PublishConfig {
                    path,
                    registry,
                    tag,
                },
                deps,
            )
            .await
        }
        Command::Deploy => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);

            // Get credentials first to create authenticated API client
            let credentials_provider = deps::RealCredentialsProvider;
            let Ok(credentials) = credentials_provider.get_or_refresh_credentials().await else {
                return Err(anyhow::anyhow!(
                    "Not logged in to FTL. Run 'ftl login' first."
                ));
            };

            // Create API client with authentication
            let api_client_config = api_client::ApiConfig {
                base_url: api_client::get_api_base_url(),
                auth_token: Some(credentials.access_token.clone()),
                timeout: std::time::Duration::from_secs(config::DEFAULT_API_TIMEOUT_SECS),
            };
            let api_client = api_client::create_client(api_client_config)?;

            let deps = Arc::new(commands::deploy::DeployDependencies {
                file_system: Arc::new(deps::RealFileSystem),
                command_executor: Arc::new(deps::RealCommandExecutor),
                ui: ui.clone(),
                credentials_provider: Arc::new(deps::RealCredentialsProvider),
                api_client: Arc::new(deps::RealFtlApiClient::new_with_auth(
                    api_client,
                    credentials.access_token,
                )),
                clock: Arc::new(deps::RealClock),
                async_runtime: Arc::new(deps::RealAsyncRuntime),
                build_executor: Arc::new(deps::RealBuildExecutor),
            });

            // Execute with v2
            commands::deploy::execute_with_deps(deps).await
        }
        Command::App { command } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);

            // Get credentials first to create authenticated API client
            let credentials_provider = deps::RealCredentialsProvider;
            let Ok(credentials) = credentials_provider.get_or_refresh_credentials().await else {
                return Err(anyhow::anyhow!(
                    "Not logged in to FTL. Run 'ftl login' first."
                ));
            };

            // Create API client with authentication
            let api_client_config = api_client::ApiConfig {
                base_url: api_client::get_api_base_url(),
                auth_token: Some(credentials.access_token.clone()),
                timeout: std::time::Duration::from_secs(config::DEFAULT_API_TIMEOUT_SECS),
            };
            let api_client = api_client::create_client(api_client_config)?;
            let api_client = Arc::new(deps::RealFtlApiClient::new_with_auth(
                api_client,
                credentials.access_token,
            ));

            let deps = Arc::new(commands::app::AppDependencies {
                ui: ui.clone(),
                api_client,
            });

            match command {
                AppCommand::List { format } => {
                    let output_format = match format.as_str() {
                        "json" => commands::app::OutputFormat::Json,
                        _ => commands::app::OutputFormat::Table,
                    };
                    commands::app::list_with_deps(output_format, &deps).await
                }
                AppCommand::Status { name, format } => {
                    let output_format = match format.as_str() {
                        "json" => commands::app::OutputFormat::Json,
                        _ => commands::app::OutputFormat::Table,
                    };
                    commands::app::status_with_deps(&name, output_format, &deps).await
                }
                AppCommand::Delete { name, force } => {
                    commands::app::delete_with_deps(&name, force, &deps).await
                }
            }
        }
        Command::Registry { command } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::registry::RegistryDependencies { ui: ui.clone() });

            match command {
                RegistryCommand::List { registry } => {
                    commands::registry::list_with_deps(registry.as_deref(), &deps);
                    Ok(())
                }
                RegistryCommand::Search { query, registry } => {
                    commands::registry::search_with_deps(&query, registry.as_deref(), &deps);
                    Ok(())
                }
                RegistryCommand::Info { component } => {
                    commands::registry::info_with_deps(&component, &deps);
                    Ok(())
                }
            }
        }
        Command::Setup { command } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::setup::SetupDependencies {
                ui: ui.clone(),
                spin_installer: Arc::new(RealSpinInstallerForSetup),
                command_executor: Arc::new(RealSetupCommandExecutor),
                environment: Arc::new(RealEnvironmentForSetup),
            });

            match command {
                SetupCommand::Templates {
                    force,
                    git,
                    branch,
                    dir,
                    tar,
                } => commands::setup::templates_with_deps(
                    force,
                    git.as_deref(),
                    branch.as_deref(),
                    dir.as_ref(),
                    tar.as_deref(),
                    &deps,
                ),
                SetupCommand::Info => {
                    commands::setup::info_with_deps(&deps);
                    Ok(())
                }
            }
        }
        Command::Update { force } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::update::UpdateDependencies {
                ui: ui.clone(),
                http_client: Arc::new(RealHttpClientForUpdate),
                command_executor: Arc::new(RealCommandExecutorForUpdate),
                environment: Arc::new(RealEnvironmentForUpdate),
            });

            // Execute with v2
            commands::update::execute_with_deps(force, deps).await
        }
        Command::Login { no_browser } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::login::LoginDependencies {
                ui: ui.clone(),
                http_client: Arc::new(RealHttpClient),
                keyring: Arc::new(RealKeyringStorage),
                browser_launcher: Arc::new(RealBrowserLauncher),
                async_runtime: Arc::new(deps::RealAsyncRuntime),
                clock: Arc::new(RealClockForLogin),
            });

            // Execute with v2
            commands::login::execute_with_deps(
                commands::login::LoginConfig {
                    no_browser,
                    authkit_domain: None,
                    client_id: None,
                },
                deps,
            )
            .await
        }
        Command::Logout => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::logout::LogoutDependencies {
                ui: ui.clone(),
                credentials_clearer: Arc::new(RealCredentialsClearer),
            });

            // Execute with v2
            commands::logout::execute_with_deps(&deps)
        }
        Command::Auth { command } => {
            // Create dependencies
            let ui = Arc::new(ui::RealUserInterface);
            let deps = Arc::new(commands::auth::AuthDependencies {
                ui: ui.clone(),
                credentials_provider: Arc::new(RealCredentialsProviderForAuth),
                clock: Arc::new(RealClockForAuth),
            });

            match command {
                AuthCommand::Status => {
                    commands::auth::status_with_deps(&deps);
                    Ok(())
                }
            }
        }
    }
}
