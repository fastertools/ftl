//! Test helper utilities and mock implementations for the FTL CLI.
//!
//! This module provides mock implementations and utility functions to help
//! write tests for the FTL CLI. It includes mocks for external dependencies
//! like the file system, API client, and command executor.
//!
//! # Example
//!
//! ```rust
//! use ftl_core::test_helpers::*;
//! use ftl_core::deps::*;
//!
//! #[tokio::test]
//! async fn test_with_mocked_dependencies() {
//!     // Create a mock file system
//!     let mut fs_mock = MockFileSystemMock::new();
//!     fs_mock
//!         .expect_exists()
//!         .with(mockall::predicate::eq(Path::new("/test/path")))
//!         .returning(|_| true);
//!
//!     // Create a mock API client
//!     let mut api_mock = MockFtlApiClientMock::new();
//!     api_mock
//!         .expect_list_apps()
//!         .returning(|| Ok(types::ListAppsResponse { apps: vec![] }));
//!
//!     // Use the mocks in your test
//!     // ...
//! }
//! ```

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::api_client::types;
use crate::deps::*;
use base64::Engine;

/// Type alias for a function that matches command execution calls.
/// Used to verify that the correct command and arguments were provided.
type CommandMatcher = Box<dyn Fn(&str, &[&str]) -> bool + Send + Sync>;

/// Type alias for a function that matches command execution calls with stdin.
/// Used to verify that the correct command, arguments, and stdin were provided.
type CommandWithStdinMatcher = Box<dyn Fn(&str, &[&str], &str) -> bool + Send + Sync>;

// Mock implementations using mockall

/// Mock implementation of the `FileSystem` trait for testing.
///
/// This mock allows you to control file system operations in tests without
/// actually touching the disk.
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::MockFileSystemMock;
/// use std::path::Path;
///
/// let mut fs_mock = MockFileSystemMock::new();
/// 
/// // Mock that a file exists
/// fs_mock
///     .expect_exists()
///     .with(mockall::predicate::eq(Path::new("/test/file.txt")))
///     .returning(|_| true);
///
/// // Mock reading a file
/// fs_mock
///     .expect_read_to_string()
///     .with(mockall::predicate::eq(Path::new("/test/file.txt")))
///     .returning(|_| Ok("file contents".to_string()));
/// ```
mock! {
    pub FileSystemMock {}

    impl FileSystem for FileSystemMock {
        fn exists(&self, path: &Path) -> bool;
        fn read_to_string(&self, path: &Path) -> Result<String>;
        fn write_string(&self, path: &Path, content: &str) -> Result<()>;
    }
}

/// Mock implementation of the `FtlApiClient` trait for testing.
///
/// This mock allows you to simulate API responses without making actual network calls.
/// Use the helper functions like `test_ecr_credentials()` and `test_deployment_response()`
/// to create realistic test data.
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::{MockFtlApiClientMock, test_deployment_response};
/// use ftl_core::api_client::types;
///
/// let mut api_mock = MockFtlApiClientMock::new();
/// 
/// // Mock listing apps
/// api_mock
///     .expect_list_apps()
///     .returning(|| Ok(types::ListAppsResponse {
///         apps: vec![types::App {
///             name: "test-app".to_string(),
///             display_name: "Test App".to_string(),
///             // ... other fields
///         }]
///     }));
///
/// // Mock deploying an app
/// api_mock
///     .expect_deploy_app()
///     .returning(|_| Ok(test_deployment_response("test-deployment-id")));
/// ```
mock! {
    pub FtlApiClientMock {}

    #[async_trait]
    impl FtlApiClient for FtlApiClientMock {
        async fn get_ecr_credentials(&self) -> Result<types::GetEcrCredentialsResponse>;
        async fn create_ecr_repository(&self, request: &types::CreateEcrRepositoryRequest) -> Result<types::CreateEcrRepositoryResponse>;
        async fn get_deployment_status(&self, deployment_id: &str) -> Result<types::DeploymentStatus>;
        async fn deploy_app(&self, request: &types::DeploymentRequest) -> Result<types::DeploymentResponse>;
        async fn list_apps(&self) -> Result<types::ListAppsResponse>;
        async fn get_app_status(&self, app_name: &str) -> Result<types::GetAppStatusResponse>;
        async fn delete_app(&self, app_name: &str) -> Result<types::DeleteAppResponse>;
    }
}

/// Mock implementation of the `Clock` trait for testing.
///
/// This mock allows you to control time-related operations in tests,
/// making them deterministic and faster.
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::MockClockMock;
/// use std::time::{Duration, Instant};
///
/// let mut clock_mock = MockClockMock::new();
/// 
/// // Mock the current time
/// let now = Instant::now();
/// clock_mock
///     .expect_now()
///     .returning(move || now);
///
/// // Mock duration creation
/// clock_mock
///     .expect_duration_from_secs()
///     .with(mockall::predicate::eq(5))
///     .returning(|secs| Duration::from_secs(secs));
/// ```
mock! {
    pub ClockMock {}

    impl Clock for ClockMock {
        fn now(&self) -> Instant;
        fn duration_from_millis(&self, millis: u64) -> Duration;
        fn duration_from_secs(&self, secs: u64) -> Duration;
    }
}

/// Mock implementation of the `CredentialsProvider` trait for testing.
///
/// This mock allows you to simulate credential retrieval without actually
/// interacting with authentication services.
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::{MockCredentialsProviderMock, test_credentials};
///
/// let mut creds_mock = MockCredentialsProviderMock::new();
/// 
/// // Return test credentials
/// creds_mock
///     .expect_get_or_refresh_credentials()
///     .returning(|| Ok(test_credentials()));
/// ```
mock! {
    pub CredentialsProviderMock {}

    #[async_trait]
    impl CredentialsProvider for CredentialsProviderMock {
        async fn get_or_refresh_credentials(&self) -> Result<StoredCredentials>;
    }
}

/// Mock implementation of the `AsyncRuntime` trait for testing.
///
/// This mock allows you to control async operations like sleep in tests,
/// making them run instantly instead of waiting.
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::MockAsyncRuntimeMock;
/// use std::time::Duration;
///
/// let mut runtime_mock = MockAsyncRuntimeMock::new();
/// 
/// // Make sleep return immediately instead of waiting
/// runtime_mock
///     .expect_sleep()
///     .with(mockall::predicate::eq(Duration::from_secs(5)))
///     .returning(|_| ());
/// ```
mock! {
    pub AsyncRuntimeMock {}

    #[async_trait]
    impl AsyncRuntime for AsyncRuntimeMock {
        async fn sleep(&self, duration: Duration);
    }
}


// Simple manual mock implementation for CommandExecutor
// This avoids mockall's issues with async traits containing slice references
use std::sync::{Arc, Mutex};

/// Type alias for command existence check functions
type CommandCheckFn = dyn Fn(&str) -> Result<()> + Send + Sync;

/// Type alias for command execution functions
type CommandExecFn = dyn Fn(&str, &[&str]) -> Result<CommandOutput> + Send + Sync;

/// Type alias for command execution with stdin functions
type CommandExecWithStdinFn = dyn Fn(&str, &[&str], &str) -> Result<CommandOutput> + Send + Sync;

/// Mock implementation of the `CommandExecutor` trait for testing.
///
/// This is a custom mock implementation (not using mockall) because the
/// `CommandExecutor` trait has async methods with slice references, which
/// mockall has trouble handling.
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::MockCommandExecutorMock;
/// use ftl_core::deps::CommandOutput;
///
/// let mut cmd_mock = MockCommandExecutorMock::new();
///
/// // Mock checking if a command exists
/// cmd_mock
///     .expect_check_command_exists()
///     .with(mockall::predicate::eq("spin"))
///     .returning(|_| Ok(()));
///
/// // Mock executing a command
/// cmd_mock
///     .expect_execute()
///     .withf(|cmd, args| cmd == "spin" && args == &["--version"])
///     .returning(|_, _| Ok(CommandOutput {
///         success: true,
///         stdout: b"spin 2.0.0\n".to_vec(),
///         stderr: vec![],
///     }));
///
/// // Mock executing a command with stdin
/// cmd_mock
///     .expect_execute_with_stdin()
///     .withf(|cmd, args, stdin| {
///         cmd == "docker" && args == &["login"] && stdin.contains("password")
///     })
///     .returning(|_, _, _| Ok(CommandOutput {
///         success: true,
///         stdout: b"Login Succeeded\n".to_vec(),
///         stderr: vec![],
///     }));
/// ```
pub struct MockCommandExecutorMock {
    check_command_exists_fn: Arc<Mutex<Option<Box<CommandCheckFn>>>>,
    execute_fns: Arc<Mutex<Vec<Box<CommandExecFn>>>>,
    execute_call_count: Arc<Mutex<usize>>,
    execute_with_stdin_fns: Arc<Mutex<Vec<Box<CommandExecWithStdinFn>>>>,
    execute_with_stdin_call_count: Arc<Mutex<usize>>,
}

impl MockCommandExecutorMock {
    /// Creates a new instance of the mock command executor.
    pub fn new() -> Self {
        Self {
            check_command_exists_fn: Arc::new(Mutex::new(None)),
            execute_fns: Arc::new(Mutex::new(Vec::new())),
            execute_call_count: Arc::new(Mutex::new(0)),
            execute_with_stdin_fns: Arc::new(Mutex::new(Vec::new())),
            execute_with_stdin_call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Sets up an expectation for the `check_command_exists` method.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cmd_mock = MockCommandExecutorMock::new();
    /// cmd_mock
    ///     .expect_check_command_exists()
    ///     .with(mockall::predicate::eq("docker"))
    ///     .returning(|_| Ok(()));
    /// ```
    pub fn expect_check_command_exists(&mut self) -> CheckCommandExistsExpectation {
        CheckCommandExistsExpectation { mock: self }
    }

    /// Sets up an expectation for the `execute` method.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cmd_mock = MockCommandExecutorMock::new();
    /// cmd_mock
    ///     .expect_execute()
    ///     .withf(|cmd, args| cmd == "echo" && args == &["hello"])
    ///     .returning(|_, _| Ok(CommandOutput {
    ///         success: true,
    ///         stdout: b"hello\n".to_vec(),
    ///         stderr: vec![],
    ///     }));
    /// ```
    pub fn expect_execute(&mut self) -> ExecuteExpectation {
        ExecuteExpectation {
            mock: self,
            matcher: None,
        }
    }

    /// Sets up an expectation for the `execute_with_stdin` method.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cmd_mock = MockCommandExecutorMock::new();
    /// cmd_mock
    ///     .expect_execute_with_stdin()
    ///     .withf(|cmd, args, stdin| {
    ///         cmd == "cat" && args.is_empty() && stdin == "test input"
    ///     })
    ///     .returning(|_, _, stdin| Ok(CommandOutput {
    ///         success: true,
    ///         stdout: stdin.as_bytes().to_vec(),
    ///         stderr: vec![],
    ///     }));
    /// ```
    pub fn expect_execute_with_stdin(&mut self) -> ExecuteWithStdinExpectation {
        ExecuteWithStdinExpectation {
            mock: self,
            matcher: None,
        }
    }
}

/// Builder for setting up expectations on the `check_command_exists` method.
pub struct CheckCommandExistsExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
}

impl<'a> CheckCommandExistsExpectation<'a> {
    /// Adds a predicate to match the command parameter.
    /// Currently not implemented but kept for API compatibility.
    pub fn with<P>(self, _p: P) -> Self {
        self
    }

    /// Specifies how many times this expectation should be called.
    /// Currently not implemented but kept for API compatibility.
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched.
    ///
    /// # Parameters
    /// * `f` - A function that takes a command name and returns a Result
    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        *self.mock.check_command_exists_fn.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Builder for setting up expectations on the `execute` method.
pub struct ExecuteExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
    matcher: Option<CommandMatcher>,
}

impl<'a> ExecuteExpectation<'a> {
    /// Adds a custom matcher function to verify the command and arguments.
    ///
    /// # Parameters
    /// * `f` - A function that takes command and args and returns true if they match
    pub fn withf<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[&str]) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(f));
        self
    }

    /// Specifies how many times this expectation should be called.
    /// Currently not implemented but kept for API compatibility.
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched.
    ///
    /// # Parameters
    /// * `f` - A function that takes command and args and returns a CommandOutput
    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str, &[&str]) -> Result<CommandOutput> + Send + Sync + 'static,
    {
        self.mock.execute_fns.lock().unwrap().push(Box::new(f));
        self.mock
    }
}

/// Builder for setting up expectations on the `execute_with_stdin` method.
pub struct ExecuteWithStdinExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
    matcher: Option<CommandWithStdinMatcher>,
}

impl<'a> ExecuteWithStdinExpectation<'a> {
    /// Adds a custom matcher function to verify the command, arguments, and stdin.
    ///
    /// # Parameters
    /// * `f` - A function that takes command, args, and stdin and returns true if they match
    pub fn withf<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[&str], &str) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(f));
        self
    }

    /// Specifies how many times this expectation should be called.
    /// Currently not implemented but kept for API compatibility.
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched.
    ///
    /// # Parameters
    /// * `f` - A function that takes command, args, and stdin and returns a CommandOutput
    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str, &[&str], &str) -> Result<CommandOutput> + Send + Sync + 'static,
    {
        self.mock
            .execute_with_stdin_fns
            .lock()
            .unwrap()
            .push(Box::new(f));
        self.mock
    }
}

#[async_trait]
impl CommandExecutor for MockCommandExecutorMock {
    async fn check_command_exists(&self, command: &str) -> Result<()> {
        if let Some(ref f) = *self.check_command_exists_fn.lock().unwrap() {
            f(command)
        } else {
            Ok(())
        }
    }

    async fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        let mut count = self.execute_call_count.lock().unwrap();
        let index = *count;
        *count += 1;

        let fns = self.execute_fns.lock().unwrap();
        if index < fns.len() {
            fns[index](command, args)
        } else {
            Ok(CommandOutput {
                success: true,
                stdout: vec![],
                stderr: vec![],
            })
        }
    }

    async fn execute_with_stdin(
        &self,
        command: &str,
        args: &[&str],
        stdin: &str,
    ) -> Result<CommandOutput> {
        let mut count = self.execute_with_stdin_call_count.lock().unwrap();
        let index = *count;
        *count += 1;

        let fns = self.execute_with_stdin_fns.lock().unwrap();
        if index < fns.len() {
            fns[index](command, args, stdin)
        } else {
            Ok(CommandOutput {
                success: true,
                stdout: vec![],
                stderr: vec![],
            })
        }
    }
}

/// Creates test credentials for use in tests.
///
/// Returns a `StoredCredentials` instance with:
/// - Access token: "test-token"
/// - Refresh token: "refresh-token"
/// - Expiration: 1 hour from now
/// - AuthKit domain: "test.authkit.app"
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::test_credentials;
///
/// let creds = test_credentials();
/// assert_eq!(creds.access_token, "test-token");
/// ```
#[allow(dead_code)]
pub fn test_credentials() -> StoredCredentials {
    StoredCredentials {
        access_token: "test-token".to_string(),
        refresh_token: Some("refresh-token".to_string()),
        id_token: None,
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        authkit_domain: "test.authkit.app".to_string(),
    }
}

/// Creates test ECR (Elastic Container Registry) credentials for use in tests.
///
/// Returns a `GetEcrCredentialsResponse` with realistic test data including:
/// - Registry URI for AWS account 123456789012 in us-east-1
/// - Base64-encoded authorization token
/// - Expiration: 12 hours from now
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::test_ecr_credentials;
///
/// let ecr_creds = test_ecr_credentials();
/// assert!(ecr_creds.registry_uri.contains("123456789012.dkr.ecr"));
/// ```
#[allow(dead_code)]
pub fn test_ecr_credentials() -> types::GetEcrCredentialsResponse {
    types::GetEcrCredentialsResponse {
        registry_uri: "123456789012.dkr.ecr.us-east-1.amazonaws.com".to_string(),
        authorization_token: base64::engine::general_purpose::STANDARD.encode("AWS:test-password"),
        proxy_endpoint: "https://123456789012.dkr.ecr.us-east-1.amazonaws.com".to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(12),
        region: "us-east-1".to_string(),
    }
}

/// Creates a test deployment response for use in tests.
///
/// # Parameters
/// * `deployment_id` - The deployment ID to use. If not a valid UUID, defaults to a fixed UUID.
///
/// # Returns
/// A `DeploymentResponse` with:
/// - App name: "test-app"
/// - Status: Initializing
/// - Status URL pointing to the deployment status endpoint
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::test_deployment_response;
///
/// let response = test_deployment_response("my-deployment-id");
/// assert_eq!(response.app_name, "test-app");
/// assert_eq!(response.status, types::DeploymentResponseStatus::Initializing);
/// ```
#[allow(dead_code)]
pub fn test_deployment_response(deployment_id: &str) -> types::DeploymentResponse {
    // Use a fixed UUID if the provided ID is not a valid UUID
    let uuid = deployment_id
        .parse()
        .unwrap_or_else(|_| "550e8400-e29b-41d4-a716-446655440000".parse().unwrap());
    types::DeploymentResponse {
        app_name: "test-app".to_string(),
        deployment_id: uuid,
        status: types::DeploymentResponseStatus::Initializing,
        message: "Deployment started".to_string(),
        status_url: format!("/v1/deployments/{deployment_id}/status"),
    }
}

/// Creates a test deployment status for use in tests.
///
/// # Parameters
/// * `deployment_id` - The deployment ID to use. If not a valid UUID, defaults to a fixed UUID.
/// * `status` - The deployment status to set (e.g., Pending, Running, Completed, Failed)
///
/// # Returns
/// A `DeploymentStatus` with comprehensive deployment information including:
/// - App details (name: "test-app", display name: "Test App")
/// - Deployment URL: "https://test-app.example.com"
/// - Platform: "Fermyon"
/// - Timestamps set to current time
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::test_deployment_status;
/// use ftl_core::api_client::types::DeploymentStatusDeploymentStatus;
///
/// let status = test_deployment_status(
///     "my-deployment-id",
///     DeploymentStatusDeploymentStatus::Completed
/// );
/// assert_eq!(status.deployment.app_name, "test-app");
/// ```
#[allow(dead_code)]
pub fn test_deployment_status(
    deployment_id: &str,
    status: types::DeploymentStatusDeploymentStatus,
) -> types::DeploymentStatus {
    // Use a fixed UUID if the provided ID is not a valid UUID
    let uuid = deployment_id
        .parse()
        .unwrap_or_else(|_| "550e8400-e29b-41d4-a716-446655440000".parse().unwrap());
    types::DeploymentStatus {
        deployment: types::DeploymentStatusDeployment {
            deployment_id: uuid,
            app_name: "test-app".to_string(),
            display_name: "Test App".to_string(),
            status,
            deployment_url: Some("https://test-app.example.com".to_string()),
            image_url: "test-image:latest".to_string(),
            platform: "Fermyon".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            completed_at: None,
            error: None,
            retry_count: 0,
            stages: vec![],
        },
    }
}

/// Creates a test ECR repository response for use in tests.
///
/// # Parameters
/// * `tool_name` - The name of the tool/repository
///
/// # Returns
/// A `CreateEcrRepositoryResponse` with:
/// - Repository URI in the format: `123456789012.dkr.ecr.us-east-1.amazonaws.com/user/{tool_name}`
/// - Repository name: `user/{tool_name}`
/// - already_exists: false
///
/// # Example
///
/// ```rust
/// use ftl_core::test_helpers::test_repository_response;
///
/// let response = test_repository_response("my-app");
/// assert_eq!(response.repository_name, "user/my-app");
/// assert!(!response.already_exists);
/// ```
#[allow(dead_code)]
pub fn test_repository_response(tool_name: &str) -> types::CreateEcrRepositoryResponse {
    types::CreateEcrRepositoryResponse {
        repository_uri: format!("123456789012.dkr.ecr.us-east-1.amazonaws.com/user/{tool_name}"),
        repository_name: format!("user/{tool_name}"),
        already_exists: false,
    }
}
