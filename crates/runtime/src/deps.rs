//! Dependency injection traits for testability
//!
//! This module provides trait abstractions for all external dependencies,
//! allowing for easy mocking and testing.

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api_client::{Client as ApiClient, types};

/// Unified stored credentials structure used across the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    /// JWT access token for API authentication
    pub access_token: String,
    /// Optional refresh token for obtaining new access tokens
    pub refresh_token: Option<String>,
    /// Optional ID token containing user information
    pub id_token: Option<String>,
    /// Expiration time of the access token
    pub expires_at: Option<DateTime<Utc>>,
    /// `AuthKit` domain used for authentication
    pub authkit_domain: String,
}

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
    /// Whether the command exited successfully
    pub success: bool,
    /// Standard output from the command
    pub stdout: Vec<u8>,
    /// Standard error from the command
    pub stderr: Vec<u8>,
}

/// FTL API client operations
#[async_trait]
pub trait FtlApiClient: Send + Sync {
    /// Create application
    async fn create_app(
        &self,
        request: &types::CreateAppRequest,
    ) -> Result<types::CreateAppResponse>;

    /// List applications
    async fn list_apps(
        &self,
        limit: Option<std::num::NonZeroU64>,
        next_token: Option<&str>,
        name: Option<&str>,
    ) -> Result<types::ListAppsResponse>;

    /// Get application
    async fn get_app(&self, app_id: &str) -> Result<types::App>;

    /// Delete application
    async fn delete_app(&self, app_id: &str) -> Result<types::DeleteAppResponse>;

    /// Create deployment
    async fn create_deployment(
        &self,
        app_id: &str,
        request: &types::CreateDeploymentRequest,
    ) -> Result<types::CreateDeploymentResponse>;

    /// List components for an app
    async fn list_app_components(&self, app_id: &str) -> Result<types::ListComponentsResponse>;

    /// Update components for an app (creates/updates/removes components and their repositories)
    async fn update_components(
        &self,
        app_id: &str,
        request: &types::UpdateComponentsRequest,
    ) -> Result<types::UpdateComponentsResponse>;

    /// Create ECR token
    async fn create_ecr_token(&self) -> Result<types::CreateEcrTokenResponse>;

    /// Update authentication configuration for an app
    async fn update_auth_config(
        &self,
        app_id: &str,
        request: &types::UpdateAuthConfigRequest,
    ) -> Result<types::AuthConfigResponse>;
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
    async fn get_or_refresh_credentials(&self) -> Result<StoredCredentials>;
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
    /// Bold text style
    Bold,
    /// Cyan colored text
    Cyan,
    /// Green colored text
    Green,
    /// Red colored text
    Red,
    /// Yellow colored text
    Yellow,
    /// Warning style (typically yellow)
    Warning,
    /// Error style (typically red)
    Error,
    /// Success style (typically green)
    Success,
}

/// Async runtime operations
#[async_trait]
pub trait AsyncRuntime: Send + Sync {
    /// Sleep for a duration
    async fn sleep(&self, duration: Duration);
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
    /// Create a new API client with authentication token
    pub const fn new_with_auth(client: ApiClient, auth_token: String) -> Self {
        Self {
            client,
            auth_token: Some(auth_token),
        }
    }
}

impl Default for RealFtlApiClient {
    fn default() -> Self {
        use crate::config::DEFAULT_API_BASE_URL;

        // Create with no auth token - will need to be set later via credentials provider
        Self {
            client: ApiClient::new(DEFAULT_API_BASE_URL),
            auth_token: None,
        }
    }
}

#[async_trait]
impl FtlApiClient for RealFtlApiClient {
    async fn create_app(
        &self,
        request: &types::CreateAppRequest,
    ) -> Result<types::CreateAppResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .create_app()
            .authorization(format!("Bearer {auth}"))
            .body(request)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to create app: {}", e))
    }

    async fn list_apps(
        &self,
        limit: Option<std::num::NonZeroU64>,
        next_token: Option<&str>,
        name: Option<&str>,
    ) -> Result<types::ListAppsResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        let mut request = self
            .client
            .list_apps()
            .authorization(format!("Bearer {auth}"));

        if let Some(limit) = limit {
            request = request.limit(limit);
        }

        if let Some(token) = next_token {
            request = request.next_token(token);
        }

        if let Some(name) = name {
            request = request.name(name);
        }

        request
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to list apps: {}", e))
    }

    async fn get_app(&self, app_id: &str) -> Result<types::App> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .get_app()
            .authorization(format!("Bearer {auth}"))
            .app_id(app_id)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to get app: {}", e))
    }

    async fn delete_app(&self, app_id: &str) -> Result<types::DeleteAppResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .delete_app()
            .authorization(format!("Bearer {auth}"))
            .app_id(app_id)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to delete app: {}", e))
    }

    async fn create_deployment(
        &self,
        app_id: &str,
        request: &types::CreateDeploymentRequest,
    ) -> Result<types::CreateDeploymentResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .create_deployment()
            .authorization(format!("Bearer {auth}"))
            .app_id(app_id)
            .body(request)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to create deployment: {}", e))
    }

    async fn update_components(
        &self,
        app_id: &str,
        request: &types::UpdateComponentsRequest,
    ) -> Result<types::UpdateComponentsResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .update_components()
            .authorization(format!("Bearer {auth}"))
            .app_id(app_id)
            .body(request)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to update components: {}", e))
    }

    async fn list_app_components(&self, app_id: &str) -> Result<types::ListComponentsResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .list_app_components()
            .authorization(format!("Bearer {auth}"))
            .app_id(app_id)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to list components: {}", e))
    }

    async fn create_ecr_token(&self) -> Result<types::CreateEcrTokenResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .create_ecr_token()
            .authorization(format!("Bearer {auth}"))
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to create ECR token: {}", e))
    }

    async fn update_auth_config(
        &self,
        app_id: &str,
        request: &types::UpdateAuthConfigRequest,
    ) -> Result<types::AuthConfigResponse> {
        let auth = self
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication token available"))?;

        self.client
            .update_auth_config()
            .app_id(app_id)
            .authorization(format!("Bearer {auth}"))
            .body(request)
            .send()
            .await
            .map(progenitor_client::ResponseValue::into_inner)
            .map_err(|e| anyhow::anyhow!("Failed to update auth config: {}", e))
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
    async fn get_or_refresh_credentials(&self) -> Result<StoredCredentials> {
        use keyring::Entry;

        // Try to get stored credentials
        let entry = Entry::new("ftl-cli", "default")
            .map_err(|e| anyhow::anyhow!("Failed to access keyring: {}", e))?;

        let json = entry
            .get_password()
            .map_err(|e| anyhow::anyhow!("Failed to retrieve credentials: {}", e))?;

        let credentials: StoredCredentials = serde_json::from_str(&json)
            .map_err(|e| anyhow::anyhow!("Failed to parse stored credentials: {}", e))?;

        // Check if token is expired or about to expire (within 30 seconds)
        if let Some(expires_at) = credentials.expires_at {
            let now = Utc::now();
            let buffer = chrono::Duration::seconds(30);

            if expires_at < now + buffer {
                // Token is expired or about to expire, try to refresh
                if let Some(refresh_token) = credentials.refresh_token.clone() {
                    match self
                        .refresh_access_token(&credentials.authkit_domain, &refresh_token)
                        .await
                    {
                        Ok(new_credentials) => {
                            // Store updated credentials
                            let updated_json = serde_json::to_string(&new_credentials)?;
                            entry.set_password(&updated_json).map_err(|e| {
                                anyhow::anyhow!("Failed to store updated credentials: {}", e)
                            })?;
                            return Ok(new_credentials);
                        }
                        Err(e) => {
                            // Log refresh error but continue with existing token if not fully expired
                            if expires_at < now {
                                return Err(anyhow::anyhow!(
                                    "Token expired and refresh failed: {}",
                                    e
                                ));
                            }
                            // Otherwise continue with existing token
                        }
                    }
                }
            }
        }

        Ok(credentials)
    }
}

impl RealCredentialsProvider {
    async fn refresh_access_token(
        &self,
        authkit_domain: &str,
        refresh_token: &str,
    ) -> Result<StoredCredentials> {
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            id_token: Option<String>,
            expires_in: Option<u64>,
        }

        let client = reqwest::Client::new();
        let token_url = format!("https://{authkit_domain}/oauth2/token");
        let client_id = "client_01K06E1DRP26N8A3T9CGMB1YSP"; // FTL OAuth client ID

        let response = client
            .post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", client_id),
            ])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send refresh request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to refresh token: {} - {}",
                status,
                body
            ));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse token response: {}", e))?;

        let expires_at = token_response.expires_in.and_then(|seconds| {
            i64::try_from(seconds)
                .ok()
                .map(|secs| Utc::now() + chrono::Duration::seconds(secs))
        });

        Ok(StoredCredentials {
            access_token: token_response.access_token,
            refresh_token: token_response
                .refresh_token
                .or_else(|| Some(refresh_token.to_string())),
            id_token: token_response.id_token,
            expires_at,
            authkit_domain: authkit_domain.to_string(),
        })
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

    /// Send termination signal to the process (does not wait for exit)
    async fn terminate(&mut self) -> Result<()>;

    /// Terminate the process and wait for it to exit
    async fn shutdown(&mut self) -> Result<ExitStatus>;
}

/// Exit status
pub struct ExitStatus {
    /// Exit code of the process
    code: Option<i32>,
}

impl ExitStatus {
    /// Create a new exit status with an optional exit code
    pub const fn new(code: Option<i32>) -> Self {
        Self { code }
    }

    /// Check if the exit status indicates success (code 0)
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }

    /// Get the exit code if available
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

        // Don't create a new process group on spawn - let it share our process group
        // so it receives Ctrl+C signals. We'll use process groups only for termination.

        let child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn process: {}", e))?;

        Ok(Box::new(RealProcessHandle { child: Some(child) }))
    }
}

/// Real process handle implementation
pub struct RealProcessHandle {
    /// The child process
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
        if let Some(child) = self.child.as_mut() {
            // On Unix systems, try to terminate gracefully
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                let pid = i32::try_from(child.id()).unwrap_or(i32::MAX);
                let process_pid = Pid::from_raw(pid);

                // Try SIGTERM on the process first
                let _ = signal::kill(process_pid, Signal::SIGTERM);

                // Give it a moment to terminate gracefully
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                // Check if the main process is still running
                if !matches!(child.try_wait(), Ok(Some(_))) {
                    // Try to kill the process group in case it created children
                    let process_group = Pid::from_raw(-pid);
                    let _ = signal::kill(process_group, Signal::SIGTERM);

                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                    // Force kill if still running
                    if !matches!(child.try_wait(), Ok(Some(_))) {
                        let _ = signal::kill(process_pid, Signal::SIGKILL);
                        let _ = signal::kill(process_group, Signal::SIGKILL);
                        let _ = child.kill();
                    }
                }
            }

            #[cfg(not(unix))]
            {
                let _ = child.kill();
            }
        }
        Ok(())
    }

    /// Terminate the process and wait for it to exit
    async fn shutdown(&mut self) -> Result<ExitStatus> {
        self.terminate().await?;

        // Now wait for the process to fully exit
        if let Some(mut child) = self.child.take() {
            let status = child
                .wait()
                .map_err(|e| anyhow::anyhow!("Failed to wait for process: {}", e))?;
            Ok(ExitStatus::new(status.code()))
        } else {
            // Process was already consumed or never started
            Ok(ExitStatus::new(Some(0)))
        }
    }
}
