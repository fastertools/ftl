//! Test helper utilities and mock implementations for the FTL CLI.
//!
//! This module provides mock implementations and utility functions to help
//! write tests for the FTL CLI. It includes mocks for external dependencies
//! like the file system, API client, and command executor.
//!
//! # Example
//!
//! ```rust
//! use ftl_runtime::test_helpers::*;
//! use ftl_runtime::deps::*;
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
//!         .returning(|_, _, _| Ok(types::ListAppsResponse { apps: vec![], next_token: None }));
//!
//!     // Use the mocks in your test
//!     // ...
//! }
//! ```

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
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

// Mock implementation of the `FileSystem` trait for testing.
//
// This mock allows you to control file system operations in tests without
// actually touching the disk.
//
// # Example
//
// ```rust
// use ftl_runtime::test_helpers::MockFileSystemMock;
// use std::path::Path;
//
// let mut fs_mock = MockFileSystemMock::new();
//
// // Mock that a file exists
// fs_mock
//     .expect_exists()
//     .with(mockall::predicate::eq(Path::new("/test/file.txt")))
//     .returning(|_| true);
//
// // Mock reading a file
// fs_mock
//     .expect_read_to_string()
//     .with(mockall::predicate::eq(Path::new("/test/file.txt")))
//     .returning(|_| Ok("file contents".to_string()));
// ```
mock! {
    pub FileSystemMock {}

    impl FileSystem for FileSystemMock {
        fn exists(&self, path: &Path) -> bool;
        fn read_to_string(&self, path: &Path) -> Result<String>;
        fn write_string(&self, path: &Path, content: &str) -> Result<()>;
    }
}

// Mock implementation of the `FtlApiClient` trait for testing.
//
// This mock allows you to simulate API responses without making actual network calls.
// Use the helper functions like `test_ecr_credentials()` and `test_deployment_response()`
// to create realistic test data.
//
// # Example
//
// ```rust
// use ftl_runtime::test_helpers::{MockFtlApiClientMock, test_deployment_response};
// use ftl_runtime::api_client::types;
//
// let mut api_mock = MockFtlApiClientMock::new();
//
// // Mock listing apps
// api_mock
//     .expect_list_apps()
//     .returning(|| Ok(types::ListAppsResponse {
//         apps: vec![types::App {
//             name: "test-app".to_string(),
//             display_name: "Test App".to_string(),
//             // ... other fields
//         }]
//     }));
//
// // Mock deploying an app
// api_mock
//     .expect_deploy_app()
//     .returning(|_| Ok(test_deployment_response("test-deployment-id")));
// ```
// Type aliases to simplify complex function signatures
type CreateAppFn =
    Box<dyn Fn(&types::CreateAppRequest) -> Result<types::CreateAppResponse> + Send + Sync>;
type ListAppsFn = Box<
    dyn Fn(
            Option<std::num::NonZeroU64>,
            Option<&str>,
            Option<&str>,
        ) -> Result<types::ListAppsResponse>
        + Send
        + Sync,
>;
type GetAppFn = Box<dyn Fn(&str) -> Result<types::App> + Send + Sync>;
type DeleteAppFn = Box<dyn Fn(&str) -> Result<types::DeleteAppResponse> + Send + Sync>;
type CreateDeploymentFn = Box<
    dyn Fn(&str, &types::CreateDeploymentRequest) -> Result<types::CreateDeploymentResponse>
        + Send
        + Sync,
>;
type CreateEcrRepositoryFn = Box<
    dyn Fn(&types::CreateEcrRepositoryRequest) -> Result<types::CreateEcrRepositoryResponse>
        + Send
        + Sync,
>;
type CreateEcrTokenFn = Box<dyn Fn() -> Result<types::CreateEcrTokenResponse> + Send + Sync>;

/// Manual mock implementation for `FtlApiClient` due to mockall issues with async traits and references.
///
/// This mock allows setting up expectations for each method of the `FtlApiClient` trait.
/// Each method can have a custom implementation provided via the `expect_*` methods.
pub struct MockFtlApiClientMock {
    create_app: Arc<Mutex<Option<CreateAppFn>>>,
    list_apps: Arc<Mutex<Option<ListAppsFn>>>,
    get_app: Arc<Mutex<Option<GetAppFn>>>,
    delete_app: Arc<Mutex<Option<DeleteAppFn>>>,
    create_deployment: Arc<Mutex<Option<CreateDeploymentFn>>>,
    create_ecr_repository: Arc<Mutex<Option<CreateEcrRepositoryFn>>>,
    create_ecr_token: Arc<Mutex<Option<CreateEcrTokenFn>>>,
}

impl Default for MockFtlApiClientMock {
    fn default() -> Self {
        Self::new()
    }
}

impl MockFtlApiClientMock {
    /// Creates a new instance of the mock API client with no expectations set
    pub fn new() -> Self {
        Self {
            create_app: Arc::new(Mutex::new(None)),
            list_apps: Arc::new(Mutex::new(None)),
            get_app: Arc::new(Mutex::new(None)),
            delete_app: Arc::new(Mutex::new(None)),
            create_deployment: Arc::new(Mutex::new(None)),
            create_ecr_repository: Arc::new(Mutex::new(None)),
            create_ecr_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Sets up an expectation for the `create_app` method
    pub fn expect_create_app(&mut self) -> CreateAppExpectation {
        CreateAppExpectation { mock: self }
    }

    /// Sets up an expectation for the `list_apps` method
    pub fn expect_list_apps(&mut self) -> ListAppsExpectation {
        ListAppsExpectation { mock: self }
    }

    /// Sets up an expectation for the `get_app` method
    pub fn expect_get_app(&mut self) -> GetAppExpectation {
        GetAppExpectation { mock: self }
    }

    /// Sets up an expectation for the `delete_app` method
    pub fn expect_delete_app(&mut self) -> DeleteAppExpectation {
        DeleteAppExpectation { mock: self }
    }

    /// Sets up an expectation for the `create_deployment` method
    pub fn expect_create_deployment(&mut self) -> CreateDeploymentExpectation {
        CreateDeploymentExpectation { mock: self }
    }

    /// Sets up an expectation for the `create_ecr_repository` method
    pub fn expect_create_ecr_repository(&mut self) -> CreateEcrRepositoryExpectation {
        CreateEcrRepositoryExpectation { mock: self }
    }

    /// Sets up an expectation for the `create_ecr_token` method
    pub fn expect_create_ecr_token(&mut self) -> CreateEcrTokenExpectation {
        CreateEcrTokenExpectation { mock: self }
    }
}

// Expectation builders
/// Builder for setting up expectations on the `create_app` method
pub struct CreateAppExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> CreateAppExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&types::CreateAppRequest) -> Result<types::CreateAppResponse> + Send + Sync + 'static,
    {
        *self.mock.create_app.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Builder for setting up expectations on the `list_apps` method
pub struct ListAppsExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> ListAppsExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(
                Option<std::num::NonZeroU64>,
                Option<&str>,
                Option<&str>,
            ) -> Result<types::ListAppsResponse>
            + Send
            + Sync
            + 'static,
    {
        *self.mock.list_apps.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Builder for setting up expectations on the `get_app` method
pub struct GetAppExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> GetAppExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str) -> Result<types::App> + Send + Sync + 'static,
    {
        *self.mock.get_app.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Builder for setting up expectations on the `delete_app` method
pub struct DeleteAppExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> DeleteAppExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str) -> Result<types::DeleteAppResponse> + Send + Sync + 'static,
    {
        *self.mock.delete_app.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Builder for setting up expectations on the `create_deployment` method
pub struct CreateDeploymentExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> CreateDeploymentExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str, &types::CreateDeploymentRequest) -> Result<types::CreateDeploymentResponse>
            + Send
            + Sync
            + 'static,
    {
        *self.mock.create_deployment.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Builder for setting up expectations on the `create_ecr_repository` method
pub struct CreateEcrRepositoryExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> CreateEcrRepositoryExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&types::CreateEcrRepositoryRequest) -> Result<types::CreateEcrRepositoryResponse>
            + Send
            + Sync
            + 'static,
    {
        *self.mock.create_ecr_repository.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Builder for setting up expectations on the `create_ecr_token` method
pub struct CreateEcrTokenExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> CreateEcrTokenExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn() -> Result<types::CreateEcrTokenResponse> + Send + Sync + 'static,
    {
        *self.mock.create_ecr_token.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

#[async_trait]
impl FtlApiClient for MockFtlApiClientMock {
    async fn create_app(
        &self,
        request: &types::CreateAppRequest,
    ) -> Result<types::CreateAppResponse> {
        if let Some(ref f) = *self.create_app.lock().unwrap() {
            f(request)
        } else {
            Err(anyhow!("create_app not mocked"))
        }
    }

    async fn list_apps(
        &self,
        limit: Option<std::num::NonZeroU64>,
        next_token: Option<&str>,
        name: Option<&str>,
    ) -> Result<types::ListAppsResponse> {
        if let Some(ref f) = *self.list_apps.lock().unwrap() {
            f(limit, next_token, name)
        } else {
            Err(anyhow!("list_apps not mocked"))
        }
    }

    async fn get_app(&self, app_id: &str) -> Result<types::App> {
        if let Some(ref f) = *self.get_app.lock().unwrap() {
            f(app_id)
        } else {
            Err(anyhow!("get_app not mocked"))
        }
    }

    async fn delete_app(&self, app_id: &str) -> Result<types::DeleteAppResponse> {
        if let Some(ref f) = *self.delete_app.lock().unwrap() {
            f(app_id)
        } else {
            Err(anyhow!("delete_app not mocked"))
        }
    }

    async fn create_deployment(
        &self,
        app_id: &str,
        request: &types::CreateDeploymentRequest,
    ) -> Result<types::CreateDeploymentResponse> {
        if let Some(ref f) = *self.create_deployment.lock().unwrap() {
            f(app_id, request)
        } else {
            Err(anyhow!("create_deployment not mocked"))
        }
    }

    async fn create_ecr_repository(
        &self,
        request: &types::CreateEcrRepositoryRequest,
    ) -> Result<types::CreateEcrRepositoryResponse> {
        if let Some(ref f) = *self.create_ecr_repository.lock().unwrap() {
            f(request)
        } else {
            Err(anyhow!("create_ecr_repository not mocked"))
        }
    }

    async fn create_ecr_token(&self) -> Result<types::CreateEcrTokenResponse> {
        if let Some(ref f) = *self.create_ecr_token.lock().unwrap() {
            f()
        } else {
            Err(anyhow!("create_ecr_token not mocked"))
        }
    }
}

// Mock implementation of the `Clock` trait for testing.
//
// This mock allows you to control time-related operations in tests,
// making them deterministic and faster.
//
// # Example
//
// ```rust
// use ftl_runtime::test_helpers::MockClockMock;
// use std::time::{Duration, Instant};
//
// let mut clock_mock = MockClockMock::new();
//
// // Mock the current time
// let now = Instant::now();
// clock_mock
//     .expect_now()
//     .returning(move || now);
//
// // Mock duration creation
// clock_mock
//     .expect_duration_from_secs()
//     .with(mockall::predicate::eq(5))
//     .returning(|secs| Duration::from_secs(secs));
// ```
mock! {
    pub ClockMock {}

    impl Clock for ClockMock {
        fn now(&self) -> Instant;
        fn duration_from_millis(&self, millis: u64) -> Duration;
        fn duration_from_secs(&self, secs: u64) -> Duration;
    }
}

// Mock implementation of the `CredentialsProvider` trait for testing.
//
// This mock allows you to simulate credential retrieval without actually
// interacting with authentication services.
//
// # Example
//
// ```rust
// use ftl_runtime::test_helpers::{MockCredentialsProviderMock, test_credentials};
//
// let mut creds_mock = MockCredentialsProviderMock::new();
//
// // Return test credentials
// creds_mock
//     .expect_get_or_refresh_credentials()
//     .returning(|| Ok(test_credentials()));
// ```
mock! {
    pub CredentialsProviderMock {}

    #[async_trait]
    impl CredentialsProvider for CredentialsProviderMock {
        async fn get_or_refresh_credentials(&self) -> Result<StoredCredentials>;
    }
}

// Mock implementation of the `AsyncRuntime` trait for testing.
//
// This mock allows you to control async operations like sleep in tests,
// making them run instantly instead of waiting.
//
// # Example
//
// ```rust
// use ftl_runtime::test_helpers::MockAsyncRuntimeMock;
// use std::time::Duration;
//
// let mut runtime_mock = MockAsyncRuntimeMock::new();
//
// // Make sleep return immediately instead of waiting
// runtime_mock
//     .expect_sleep()
//     .with(mockall::predicate::eq(Duration::from_secs(5)))
//     .returning(|_| ());
// ```
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
/// use ftl_runtime::test_helpers::MockCommandExecutorMock;
/// use ftl_runtime::deps::CommandOutput;
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

impl Default for MockCommandExecutorMock {
    fn default() -> Self {
        Self::new()
    }
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
    #[must_use]
    pub fn with<P>(self, _p: P) -> Self {
        self
    }

    /// Specifies how many times this expectation should be called.
    /// Currently not implemented but kept for API compatibility.
    #[must_use]
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
    #[must_use]
    pub fn withf<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[&str]) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(f));
        self
    }

    /// Specifies how many times this expectation should be called.
    /// Currently not implemented but kept for API compatibility.
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched.
    ///
    /// # Parameters
    /// * `f` - A function that takes command and args and returns a `CommandOutput`
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
    #[must_use]
    pub fn withf<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[&str], &str) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(f));
        self
    }

    /// Specifies how many times this expectation should be called.
    /// Currently not implemented but kept for API compatibility.
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Sets the function to be called when this expectation is matched.
    ///
    /// # Parameters
    /// * `f` - A function that takes command, args, and stdin and returns a `CommandOutput`
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
/// - `AuthKit` domain: "test.authkit.app"
///
/// # Example
///
/// ```rust
/// use ftl_runtime::test_helpers::test_credentials;
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
/// use ftl_runtime::test_helpers::test_ecr_credentials;
///
/// let ecr_creds = test_ecr_credentials();
/// assert!(ecr_creds.registry_uri.contains("123456789012.dkr.ecr"));
/// ```
#[allow(dead_code)]
pub fn test_ecr_credentials() -> types::CreateEcrTokenResponse {
    types::CreateEcrTokenResponse {
        registry_uri: "123456789012.dkr.ecr.us-east-1.amazonaws.com".to_string(),
        authorization_token: base64::engine::general_purpose::STANDARD.encode("AWS:test-password"),
        proxy_endpoint: "https://123456789012.dkr.ecr.us-east-1.amazonaws.com".to_string(),
        expires_at: (chrono::Utc::now() + chrono::Duration::hours(12)).to_rfc3339(),
        region: "us-east-1".to_string(),
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
/// - `already_exists`: false
///
/// # Example
///
/// ```rust
/// use ftl_runtime::test_helpers::test_repository_response;
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
