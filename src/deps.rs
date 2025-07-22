//! Dependency injection traits for testability
//!
//! This module provides trait abstractions for all external dependencies,
//! allowing for easy mocking and testing.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;

use crate::api_client::{Client as ApiClient, types};
use crate::commands::login::StoredCredentials as Credentials;

/// File system operations
pub trait FileSystem: Send + Sync {
    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;

    /// Read a file to string
    fn read_to_string(&self, path: &Path) -> Result<String>;

    /// Write string to file
    fn write_string(&self, path: &Path, content: &str) -> Result<()>;
}

/// Command execution operations
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Check if a command exists in PATH
    async fn check_command_exists(&self, command: &str) -> Result<()>;

    /// Execute a command with arguments
    async fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput>;

    /// Execute a command with stdin input
    async fn execute_with_stdin(
        &self,
        command: &str,
        args: &[&str],
        stdin: &str,
    ) -> Result<CommandOutput>;
}

/// Output from command execution
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// FTL API client operations
#[async_trait]
pub trait FtlApiClient: Send + Sync {
    /// Get ECR credentials
    async fn get_ecr_credentials(&self) -> Result<types::GetEcrCredentialsResponse>;

    /// Create ECR repository
    async fn create_ecr_repository(
        &self,
        request: &types::CreateEcrRepositoryRequest,
    ) -> Result<types::CreateEcrRepositoryResponse>;

    /// Get deployment status
    async fn get_deployment_status(&self, deployment_id: &str) -> Result<types::DeploymentStatus>;

    /// Deploy application
    async fn deploy_app(
        &self,
        request: &types::DeploymentRequest,
    ) -> Result<types::DeploymentResponse>;
}

/// Time/clock operations
pub trait Clock: Send + Sync {
    /// Get current instant
    fn now(&self) -> Instant;

    /// Create duration from milliseconds
    fn duration_from_millis(&self, millis: u64) -> Duration;

    /// Create duration from seconds
    fn duration_from_secs(&self, secs: u64) -> Duration;
}

/// Credentials provider operations
#[async_trait]
pub trait CredentialsProvider: Send + Sync {
    /// Get or refresh credentials
    async fn get_or_refresh_credentials(&self) -> Result<Credentials>;
}

/// User interface operations
pub trait UserInterface: Send + Sync {
    /// Create a spinner progress indicator
    fn create_spinner(&self) -> Box<dyn ProgressIndicator>;

    /// Create a multi-progress manager
    fn create_multi_progress(&self) -> Box<dyn MultiProgressManager>;

    /// Print a message
    fn print(&self, message: &str);

    /// Print a styled message
    fn print_styled(&self, message: &str, style: MessageStyle);

    /// Check if running in interactive mode
    fn is_interactive(&self) -> bool;

    /// Prompt for text input
    fn prompt_input(&self, prompt: &str, default: Option<&str>) -> Result<String>;

    /// Prompt for selection
    fn prompt_select(&self, prompt: &str, items: &[&str], default: usize) -> Result<usize>;

    /// Clear the screen
    fn clear_screen(&self);
}

/// Progress indicator trait
pub trait ProgressIndicator: Send + Sync {
    /// Set the message
    fn set_message(&self, message: &str);

    /// Finish and clear the progress
    fn finish_and_clear(&self);

    /// Enable steady tick
    fn enable_steady_tick(&self, duration: Duration);

    /// Finish with a message
    fn finish_with_message(&self, message: String);

    /// Set prefix
    fn set_prefix(&self, prefix: String);
}

/// Multi-progress manager trait
pub trait MultiProgressManager: Send + Sync {
    /// Add a progress bar
    fn add_spinner(&self) -> Box<dyn ProgressIndicator>;
}

/// Message styling options
#[derive(Debug, Clone, Copy)]
pub enum MessageStyle {
    Bold,
    Cyan,
    Green,
    Red,
    Yellow,
    Warning,
    Error,
    Success,
}

/// Build executor operations
#[async_trait]
pub trait BuildExecutor: Send + Sync {
    /// Execute build
    async fn execute(&self, path: Option<&Path>, release: bool) -> Result<()>;
}

/// Async runtime operations
#[async_trait]
pub trait AsyncRuntime: Send + Sync {
    /// Sleep for a duration
    async fn sleep(&self, duration: Duration);
}

/// Spin installer operations
#[async_trait]
pub trait SpinInstaller: Send + Sync {
    /// Check and install spin if needed
    async fn check_and_install(&self) -> Result<String>;
}

// Production implementations

/// Production file system implementation
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", path.display(), e))
    }

    fn write_string(&self, path: &Path, content: &str) -> Result<()> {
        std::fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file {}: {}", path.display(), e))
    }
}

/// Production command executor implementation
pub struct RealCommandExecutor;

#[async_trait]
impl CommandExecutor for RealCommandExecutor {
    async fn check_command_exists(&self, command: &str) -> Result<()> {
        which::which(command)
            .map(|_| ())
            .map_err(|_| anyhow::anyhow!("{} not found in PATH", command))
    }

    async fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        let output = tokio::process::Command::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", command, e))?;

        Ok(CommandOutput {
            success: output.status.success(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    async fn execute_with_stdin(
        &self,
        command: &str,
        args: &[&str],
        stdin: &str,
    ) -> Result<CommandOutput> {
        use tokio::io::AsyncWriteExt;
        use tokio::process::Command as TokioCommand;

        let mut child = TokioCommand::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn {}: {}", command, e))?;

        if let Some(mut stdin_handle) = child.stdin.take() {
            stdin_handle.write_all(stdin.as_bytes()).await?;
            stdin_handle.shutdown().await?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to wait for {}: {}", command, e))?;

        Ok(CommandOutput {
            success: output.status.success(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

/// Production API client wrapper
pub struct RealFtlApiClient {
    client: ApiClient,
    auth_token: Option<String>,
}

impl RealFtlApiClient {
    #[allow(dead_code)]
    pub const fn new(client: ApiClient) -> Self {
        Self {
            client,
            auth_token: None,
        }
    }

    pub const fn new_with_auth(client: ApiClient, auth_token: String) -> Self {
        Self {
            client,
            auth_token: Some(auth_token),
        }
    }
}

#[async_trait]
impl FtlApiClient for RealFtlApiClient {
    async fn get_ecr_credentials(&self) -> Result<types::GetEcrCredentialsResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .get_ecr_credentials()
            .authorization(format!("Bearer {auth}"))
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn create_ecr_repository(
        &self,
        request: &types::CreateEcrRepositoryRequest,
    ) -> Result<types::CreateEcrRepositoryResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .create_ecr_repository()
            .authorization(format!("Bearer {auth}"))
            .body(request)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to create ECR repository: {}", e))
    }

    async fn get_deployment_status(&self, deployment_id: &str) -> Result<types::DeploymentStatus> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .get_deployment_status()
            .authorization(format!("Bearer {auth}"))
            .deployment_id(deployment_id)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to get deployment status: {}", e))
    }

    async fn deploy_app(
        &self,
        request: &types::DeploymentRequest,
    ) -> Result<types::DeploymentResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .deploy_app()
            .authorization(format!("Bearer {auth}"))
            .body(request)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to deploy app: {}", e))
    }
}

/// Production clock implementation
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn duration_from_millis(&self, millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    fn duration_from_secs(&self, secs: u64) -> Duration {
        Duration::from_secs(secs)
    }
}

/// Production credentials provider
pub struct RealCredentialsProvider;

#[async_trait]
impl CredentialsProvider for RealCredentialsProvider {
    async fn get_or_refresh_credentials(&self) -> Result<Credentials> {
        crate::commands::login::get_or_refresh_credentials().await
    }
}

/// Production build executor
pub struct RealBuildExecutor;

#[async_trait]
impl BuildExecutor for RealBuildExecutor {
    async fn execute(&self, path: Option<&Path>, release: bool) -> Result<()> {
        // Create dependencies and call execute_with_deps
        let ui = Arc::new(crate::ui::RealUserInterface);
        let deps = Arc::new(crate::commands::build::BuildDependencies {
            file_system: Arc::new(RealFileSystem),
            command_executor: Arc::new(RealCommandExecutor),
            ui: ui.clone(),
            spin_installer: Arc::new(RealSpinInstaller),
        });

        crate::commands::build::execute_with_deps(
            crate::commands::build::BuildConfig {
                path: path.map(std::path::Path::to_path_buf),
                release,
            },
            deps,
        )
        .await
    }
}

/// Production async runtime
pub struct RealAsyncRuntime;

#[async_trait]
impl AsyncRuntime for RealAsyncRuntime {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

/// Production spin installer
pub struct RealSpinInstaller;

#[async_trait]
impl SpinInstaller for RealSpinInstaller {
    async fn check_and_install(&self) -> Result<String> {
        let path = crate::common::spin_installer::check_and_install_spin().await?;
        Ok(path.to_string_lossy().to_string())
    }
}

/// Process management traits
#[async_trait]
pub trait ProcessManager: Send + Sync {
    /// Spawn a new process
    async fn spawn(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<Box<dyn ProcessHandle>>;
}

/// Process handle trait
#[async_trait]
pub trait ProcessHandle: Send + Sync {
    /// Wait for the process to exit
    async fn wait(&mut self) -> Result<ExitStatus>;

    /// Terminate the process
    async fn terminate(&mut self) -> Result<()>;
}

/// Exit status
pub struct ExitStatus {
    code: Option<i32>,
}

impl ExitStatus {
    pub const fn new(code: Option<i32>) -> Self {
        Self { code }
    }

    pub fn success(&self) -> bool {
        self.code == Some(0)
    }

    pub const fn code(&self) -> Option<i32> {
        self.code
    }
}

/// Real process manager implementation
pub struct RealProcessManager;

#[async_trait]
impl ProcessManager for RealProcessManager {
    async fn spawn(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<Box<dyn ProcessHandle>> {
        use std::process::{Command, Stdio};

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn process: {}", e))?;

        Ok(Box::new(RealProcessHandle { child: Some(child) }))
    }
}

/// Real process handle implementation
pub struct RealProcessHandle {
    child: Option<std::process::Child>,
}

#[async_trait]
impl ProcessHandle for RealProcessHandle {
    async fn wait(&mut self) -> Result<ExitStatus> {
        if let Some(mut child) = self.child.take() {
            let status = child
                .wait()
                .map_err(|e| anyhow::anyhow!("Failed to wait for process: {}", e))?;
            Ok(ExitStatus::new(status.code()))
        } else {
            anyhow::bail!("Process already consumed")
        }
    }

    async fn terminate(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child
                .kill()
                .map_err(|e| anyhow::anyhow!("Failed to terminate process: {}", e))?;
        }
        Ok(())
    }
}
