//! Test helper utilities and mock implementations for ftl-commands
//!
//! This module contains mock implementations that were previously in `ftl_runtime::test_helpers`
//! but are now local to ftl-commands for better encapsulation.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use base64::Engine;
use mockall::mock;

use ftl_common::SpinInstaller;
use ftl_runtime::api_client::types;
use ftl_runtime::deps::*;

// Re-export test utilities from ftl_common
/// Test implementation of the `UserInterface` trait that captures all output and user interactions.
///
/// This is re-exported from `ftl_common` and provides a way to test CLI commands without
/// requiring actual user input or producing console output.
///
/// # Example
///
/// ```rust
/// use ftl_commands::test_helpers::TestUserInterface;
///
/// let ui = TestUserInterface::new();
/// ui.print_info("Test message");
/// assert_eq!(ui.get_output(), vec!["Test message"]);
/// ```
pub use ftl_common::ui::TestUserInterface;

// Mock implementations using mockall

// Mock implementation of the FileSystem trait for testing file operations.
//
// This mock allows you to simulate file system operations without actually touching
// the disk, making tests faster and more reliable.
//
// # Example
//
// ```rust
// use ftl_commands::test_helpers::MockFileSystemMock;
//
// let mut mock_fs = MockFileSystemMock::new();
// mock_fs.expect_exists()
//     .with(eq(Path::new("/test/path")))
//     .times(1)
//     .returning(|_| true);
//
// mock_fs.expect_read_to_string()
//     .with(eq(Path::new("/test/file.txt")))
//     .times(1)
//     .returning(|_| Ok("file content".to_string()));
// ```
mock! {
    pub FileSystemMock {}

    impl FileSystem for FileSystemMock {
        fn exists(&self, path: &Path) -> bool;
        fn read_to_string(&self, path: &Path) -> Result<String>;
        fn write_string(&self, path: &Path, content: &str) -> Result<()>;
    }
}

// Mock implementation of the Clock trait for controlling time in tests.
//
// This mock allows you to control time-related operations in tests, making it possible
// to test timeout behavior, rate limiting, and other time-dependent functionality.
//
// # Example
//
// ```rust
// use ftl_commands::test_helpers::MockClockMock;
// use std::time::{Instant, Duration};
//
// let mut mock_clock = MockClockMock::new();
// let start_time = Instant::now();
//
// mock_clock.expect_now()
//     .times(1)
//     .returning(move || start_time);
//
// mock_clock.expect_duration_from_secs()
//     .with(eq(5))
//     .times(1)
//     .returning(|secs| Duration::from_secs(secs));
// ```
mock! {
    pub ClockMock {}

    impl Clock for ClockMock {
        fn now(&self) -> std::time::Instant;
        fn duration_from_millis(&self, millis: u64) -> std::time::Duration;
        fn duration_from_secs(&self, secs: u64) -> std::time::Duration;
    }
}

// Mock implementation of the CredentialsProvider trait for testing authentication.
//
// This mock allows you to simulate credential retrieval and refresh operations
// without requiring actual authentication services.
//
// # Example
//
// ```rust
// use ftl_commands::test_helpers::{MockCredentialsProviderMock, test_credentials};
//
// let mut mock_creds = MockCredentialsProviderMock::new();
// mock_creds.expect_get_or_refresh_credentials()
//     .times(1)
//     .returning(|| Ok(test_credentials()));
// ```
mock! {
    pub CredentialsProviderMock {}

    #[async_trait]
    impl CredentialsProvider for CredentialsProviderMock {
        async fn get_or_refresh_credentials(&self) -> Result<StoredCredentials>;
    }
}

// Mock implementation of the AsyncRuntime trait for testing async operations.
//
// This mock allows you to control async runtime behavior, particularly useful
// for testing code that uses delays or timeouts.
//
// # Example
//
// ```rust
// use ftl_commands::test_helpers::MockAsyncRuntimeMock;
// use std::time::Duration;
//
// let mut mock_runtime = MockAsyncRuntimeMock::new();
// mock_runtime.expect_sleep()
//     .with(eq(Duration::from_secs(1)))
//     .times(1)
//     .returning(|_| ());
// ```
mock! {
    pub AsyncRuntimeMock {}

    #[async_trait]
    impl AsyncRuntime for AsyncRuntimeMock {
        async fn sleep(&self, duration: std::time::Duration);
    }
}

// Mock implementation of the ApiClientFactory trait for testing API client creation.
//
// This mock allows you to control API client creation in tests.
//
// # Example
//
// ```rust
// use ftl_commands::test_helpers::{MockApiClientFactoryMock, MockFtlApiClientMock};
//
// let mut mock_factory = MockApiClientFactoryMock::new();
// let mock_client = MockFtlApiClientMock::new();
//
// mock_factory.expect_create_api_client()
//     .times(1)
//     .returning(move || Ok(Box::new(mock_client.clone())));
// ```
mock! {
    pub ApiClientFactoryMock {}

    #[async_trait]
    impl ApiClientFactory for ApiClientFactoryMock {
        async fn create_api_client(&self) -> Result<Box<dyn FtlApiClient>>;
    }
}

// Mock implementation of the SpinInstaller trait for testing Spin installation.
//
// This mock allows you to simulate Spin CLI installation without actually downloading
// or installing anything.
//
// # Example
//
// ```rust
// use ftl_commands::test_helpers::MockSpinInstallerMock;
//
// let mut mock_installer = MockSpinInstallerMock::new();
// mock_installer.expect_check_and_install()
//     .times(1)
//     .returning(|| Ok("/usr/local/bin/spin".to_string()));
// ```
mock! {
    pub SpinInstallerMock {}

    #[async_trait]
    impl SpinInstaller for SpinInstallerMock {
        async fn check_and_install(&self) -> Result<String>;
    }
}

// Mock implementation of the ProcessHandle trait for testing process management.
//
// This mock allows you to simulate process lifecycle operations like waiting for
// completion and termination.
//
// # Example
//
// ```rust
// use ftl_commands::test_helpers::MockProcessHandleMock;
// use ftl_runtime::deps::ExitStatus;
//
// let mut mock_handle = MockProcessHandleMock::new();
// mock_handle.expect_wait()
//     .times(1)
//     .returning(|| Ok(ExitStatus { code: Some(0) }));
// ```
mock! {
    pub ProcessHandleMock {}

    #[async_trait]
    impl ProcessHandle for ProcessHandleMock {
        async fn wait(&mut self) -> Result<ExitStatus>;
        async fn terminate(&mut self) -> Result<()>;
        async fn shutdown(&mut self) -> Result<ExitStatus>;
    }
}

// Manual mock for ProcessManager to avoid mockall lifetime issues

// Type alias for the spawn function type to reduce complexity
type SpawnFn =
    Box<dyn Fn(&str, Vec<String>, Option<PathBuf>) -> Result<Box<dyn ProcessHandle>> + Send + Sync>;

/// Mock implementation of the `ProcessManager` trait for testing process spawning.
///
/// This is a manual mock implementation (not using mockall) to avoid lifetime issues
/// with async traits. It allows you to control process spawning behavior in tests.
///
/// # Example
///
/// ```rust
/// use ftl_commands::test_helpers::{MockProcessManagerMock, MockProcessHandleMock};
/// use ftl_runtime::deps::ExitStatus;
///
/// let mut mock_pm = MockProcessManagerMock::new();
/// mock_pm.expect_spawn()
///     .returning(|cmd, args, _working_dir| {
///         assert_eq!(cmd, "spin");
///         assert_eq!(args, vec!["build"]);
///         
///         let mut handle = MockProcessHandleMock::new();
///         handle.expect_wait()
///             .returning(|| Ok(ExitStatus { code: Some(0) }));
///         Ok(Box::new(handle))
///     });
/// ```
pub struct MockProcessManagerMock {
    spawn_fn: Arc<Mutex<Option<SpawnFn>>>,
}

impl Default for MockProcessManagerMock {
    fn default() -> Self {
        Self::new()
    }
}

impl MockProcessManagerMock {
    /// Creates a new instance of the mock process manager.
    pub fn new() -> Self {
        Self {
            spawn_fn: Arc::new(Mutex::new(None)),
        }
    }

    /// Sets up an expectation for the spawn method.
    ///
    /// This method follows the mockall pattern for consistency with other mocks.
    pub fn expect_spawn(&mut self) -> &mut Self {
        self
    }

    /// Configures the mock to return a specific result when spawn is called.
    ///
    /// The provided function will be called with the command, arguments, and working directory
    /// passed to spawn, and should return a mock `ProcessHandle`.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes command, args, and `working_dir` and returns a `ProcessHandle`
    pub fn returning<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&str, Vec<String>, Option<PathBuf>) -> Result<Box<dyn ProcessHandle>>
            + Send
            + Sync
            + 'static,
    {
        *self.spawn_fn.lock().unwrap() = Some(Box::new(f));
        self
    }
}

#[async_trait]
impl ProcessManager for MockProcessManagerMock {
    async fn spawn(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<Box<dyn ProcessHandle>> {
        let args_owned: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
        let working_dir_owned = working_dir.map(std::path::Path::to_path_buf);

        if let Some(ref f) = *self.spawn_fn.lock().unwrap() {
            f(command, args_owned, working_dir_owned)
        } else {
            panic!("MockProcessManagerMock::spawn: no expectation set")
        }
    }
}

// Mock implementation of the FtlApiClient trait for testing API interactions.
//
// This mock allows you to simulate FTL API responses without making actual network calls.
// Use the helper functions like `test_ecr_credentials()` and `test_deployment_response()`
// to create realistic test data.
//
// # Example
//
// ```rust
// use ftl_commands::test_helpers::{MockFtlApiClientMock, test_ecr_credentials, test_deployment_response};
//
// let mut mock_client = MockFtlApiClientMock::new();
//
// // Mock ECR credentials retrieval
// mock_client.expect_get_ecr_credentials()
//     .times(1)
//     .returning(|| Ok(test_ecr_credentials()));
//
// // Mock deployment creation
// mock_client.expect_deploy_app()
//     .times(1)
//     .returning(|_req| Ok(test_deployment_response("test-deployment-id")));
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
type UpdateComponentsFn = Box<
    dyn Fn(&str, &types::UpdateComponentsRequest) -> Result<types::UpdateComponentsResponse>
        + Send
        + Sync,
>;
type ListComponentsFn = Box<dyn Fn(&str) -> Result<types::ListComponentsResponse> + Send + Sync>;
type CreateEcrTokenFn = Box<dyn Fn(&str) -> Result<types::CreateEcrTokenResponse> + Send + Sync>;
type GetAppLogsFn = Box<
    dyn Fn(&str, Option<&str>, Option<&str>) -> Result<types::GetAppLogsResponse> + Send + Sync,
>;

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
    update_components: Arc<Mutex<Option<UpdateComponentsFn>>>,
    list_app_components: Arc<Mutex<Option<ListComponentsFn>>>,
    create_ecr_token: Arc<Mutex<Option<CreateEcrTokenFn>>>,
    get_app_logs: Arc<Mutex<Option<GetAppLogsFn>>>,
}

impl Clone for MockFtlApiClientMock {
    fn clone(&self) -> Self {
        Self {
            create_app: Arc::clone(&self.create_app),
            list_apps: Arc::clone(&self.list_apps),
            get_app: Arc::clone(&self.get_app),
            delete_app: Arc::clone(&self.delete_app),
            create_deployment: Arc::clone(&self.create_deployment),
            update_components: Arc::clone(&self.update_components),
            list_app_components: Arc::clone(&self.list_app_components),
            create_ecr_token: Arc::clone(&self.create_ecr_token),
            get_app_logs: Arc::clone(&self.get_app_logs),
        }
    }
}

impl Default for MockFtlApiClientMock {
    fn default() -> Self {
        Self::new()
    }
}

impl MockFtlApiClientMock {
    /// Creates a new instance of the mock API client
    pub fn new() -> Self {
        Self {
            create_app: Arc::new(Mutex::new(None)),
            list_apps: Arc::new(Mutex::new(None)),
            get_app: Arc::new(Mutex::new(None)),
            delete_app: Arc::new(Mutex::new(None)),
            create_deployment: Arc::new(Mutex::new(None)),
            update_components: Arc::new(Mutex::new(None)),
            list_app_components: Arc::new(Mutex::new(None)),
            create_ecr_token: Arc::new(Mutex::new(None)),
            get_app_logs: Arc::new(Mutex::new(None)),
        }
    }

    /// Set up expectation for `create_app` method
    pub fn expect_create_app(&mut self) -> CreateAppExpectation<'_> {
        CreateAppExpectation { mock: self }
    }

    /// Set up expectation for `list_apps` method
    pub fn expect_list_apps(&mut self) -> ListAppsExpectation<'_> {
        ListAppsExpectation { mock: self }
    }

    /// Set up expectation for `get_app` method
    pub fn expect_get_app(&mut self) -> GetAppExpectation<'_> {
        GetAppExpectation { mock: self }
    }

    /// Set up expectation for `delete_app` method
    pub fn expect_delete_app(&mut self) -> DeleteAppExpectation<'_> {
        DeleteAppExpectation { mock: self }
    }

    /// Set up expectation for `create_deployment` method
    pub fn expect_create_deployment(&mut self) -> CreateDeploymentExpectation<'_> {
        CreateDeploymentExpectation { mock: self }
    }

    /// Set up expectation for `update_components` method
    pub fn expect_update_components(&mut self) -> UpdateComponentsExpectation<'_> {
        UpdateComponentsExpectation { mock: self }
    }

    /// Set up expectation for `list_app_components` method
    pub fn expect_list_app_components(&mut self) -> ListAppComponentsExpectation<'_> {
        ListAppComponentsExpectation { mock: self }
    }

    /// Set up expectation for `create_ecr_token` method
    pub fn expect_create_ecr_token(&mut self) -> CreateEcrTokenExpectation<'_> {
        CreateEcrTokenExpectation { mock: self }
    }

    /// Set up expectation for `get_app_logs` method
    pub fn expect_get_app_logs(&mut self) -> GetAppLogsExpectation<'_> {
        GetAppLogsExpectation { mock: self }
    }
}

// Expectation builders
/// Expectation builder for `create_app` method
pub struct CreateAppExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> CreateAppExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&types::CreateAppRequest) -> Result<types::CreateAppResponse> + Send + Sync + 'static,
    {
        *self.mock.create_app.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Expectation builder for `list_apps` method
pub struct ListAppsExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> ListAppsExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
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

/// Expectation builder for `get_app` method
pub struct GetAppExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> GetAppExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str) -> Result<types::App> + Send + Sync + 'static,
    {
        *self.mock.get_app.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Expectation builder for `delete_app` method
pub struct DeleteAppExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> DeleteAppExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str) -> Result<types::DeleteAppResponse> + Send + Sync + 'static,
    {
        *self.mock.delete_app.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Expectation builder for `create_deployment` method
pub struct CreateDeploymentExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> CreateDeploymentExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
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

/// Expectation builder for `update_components` method
pub struct UpdateComponentsExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> UpdateComponentsExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str, &types::UpdateComponentsRequest) -> Result<types::UpdateComponentsResponse>
            + Send
            + Sync
            + 'static,
    {
        *self.mock.update_components.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Expectation builder for `list_app_components` method
pub struct ListAppComponentsExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> ListAppComponentsExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str) -> Result<types::ListComponentsResponse> + Send + Sync + 'static,
    {
        *self.mock.list_app_components.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

/// Expectation builder for `create_ecr_token` method
pub struct CreateEcrTokenExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> CreateEcrTokenExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str) -> Result<types::CreateEcrTokenResponse> + Send + Sync + 'static,
    {
        *self.mock.create_ecr_token.lock().unwrap() = Some(Box::new(f));
        self.mock
    }

    /// Set the function to call when this expectation is matched (legacy - ignores `app_id`)
    pub fn returning_const<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn() -> Result<types::CreateEcrTokenResponse> + Send + Sync + 'static,
    {
        let wrapper = move |_app_id: &str| f();
        *self.mock.create_ecr_token.lock().unwrap() = Some(Box::new(wrapper));
        self.mock
    }
}

/// Expectation builder for `get_app_logs` method
pub struct GetAppLogsExpectation<'a> {
    mock: &'a mut MockFtlApiClientMock,
}

impl<'a> GetAppLogsExpectation<'a> {
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Set the function to call when this expectation is matched
    pub fn returning<F>(self, f: F) -> &'a mut MockFtlApiClientMock
    where
        F: Fn(&str, Option<&str>, Option<&str>) -> Result<types::GetAppLogsResponse>
            + Send
            + Sync
            + 'static,
    {
        *self.mock.get_app_logs.lock().unwrap() = Some(Box::new(f));
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

    async fn update_components(
        &self,
        app_id: &str,
        request: &types::UpdateComponentsRequest,
    ) -> Result<types::UpdateComponentsResponse> {
        if let Some(ref f) = *self.update_components.lock().unwrap() {
            f(app_id, request)
        } else {
            Err(anyhow!("update_components not mocked"))
        }
    }

    async fn list_app_components(&self, app_id: &str) -> Result<types::ListComponentsResponse> {
        if let Some(ref f) = *self.list_app_components.lock().unwrap() {
            f(app_id)
        } else {
            Err(anyhow!("list_app_components not mocked"))
        }
    }

    async fn create_ecr_token(&self, app_id: &str) -> Result<types::CreateEcrTokenResponse> {
        if let Some(ref f) = *self.create_ecr_token.lock().unwrap() {
            f(app_id)
        } else {
            Err(anyhow!("create_ecr_token not mocked"))
        }
    }

    async fn get_app_logs(
        &self,
        app_id: &str,
        since: Option<&str>,
        tail: Option<&str>,
    ) -> Result<types::GetAppLogsResponse> {
        if let Some(ref f) = *self.get_app_logs.lock().unwrap() {
            f(app_id, since, tail)
        } else {
            Err(anyhow!("get_app_logs not mocked"))
        }
    }

    async fn get_user_orgs(&self) -> Result<types::GetUserOrgsResponse> {
        // Return empty organizations list by default for tests
        Ok(types::GetUserOrgsResponse {
            organizations: vec![],
        })
    }
}

// Simple manual mock implementation for CommandExecutor
// This avoids mockall's issues with async traits containing slice references
type CommandCheckFn = dyn Fn(&str) -> Result<()> + Send + Sync;
type CommandExecFn = dyn Fn(&str, &[&str]) -> Result<CommandOutput> + Send + Sync;
type CommandExecWithStdinFn = dyn Fn(&str, &[&str], &str) -> Result<CommandOutput> + Send + Sync;

/// Mock implementation of the `CommandExecutor` trait for testing command execution.
///
/// This is a manual mock implementation that allows you to simulate command execution
/// without actually running external processes. It supports multiple expectations
/// and can simulate both successful and failed command executions.
///
/// # Example
///
/// ```rust
/// use ftl_commands::test_helpers::MockCommandExecutorMock;
/// use ftl_runtime::deps::CommandOutput;
///
/// let mut mock_exec = MockCommandExecutorMock::new();
///
/// // Check if command exists
/// mock_exec.expect_check_command_exists()
///     .with(eq("docker"))
///     .returning(|_| Ok(()));
///
/// // Mock command execution
/// mock_exec.expect_execute()
///     .withf(|cmd, args| cmd == "docker" && args == &["build", "."])
///     .returning(|_, _| Ok(CommandOutput {
///         success: true,
///         stdout: b"Successfully built image\n".to_vec(),
///         stderr: vec![],
///     }));
///
/// // Mock command with stdin
/// mock_exec.expect_execute_with_stdin()
///     .withf(|cmd, args, stdin| cmd == "docker" && stdin.contains("FROM rust"))
///     .returning(|_, _, _| Ok(CommandOutput {
///         success: true,
///         stdout: b"Image created\n".to_vec(),
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
    /// Returns a builder that allows you to configure the expectation with
    /// matchers and return values.
    pub fn expect_check_command_exists(&mut self) -> CheckCommandExistsExpectation<'_> {
        CheckCommandExistsExpectation { mock: self }
    }

    /// Sets up an expectation for the execute method.
    ///
    /// Returns a builder that allows you to configure the expectation with
    /// matchers and return values. Multiple expectations are called in order.
    pub fn expect_execute(&mut self) -> ExecuteExpectation<'_> {
        ExecuteExpectation {
            mock: self,
            matcher: None,
        }
    }

    /// Sets up an expectation for the `execute_with_stdin` method.
    ///
    /// Returns a builder that allows you to configure the expectation with
    /// matchers and return values. Multiple expectations are called in order.
    pub fn expect_execute_with_stdin(&mut self) -> ExecuteWithStdinExpectation<'_> {
        ExecuteWithStdinExpectation {
            mock: self,
            matcher: None,
        }
    }
}

/// Builder for configuring `check_command_exists` expectations.
///
/// This struct is returned by `MockCommandExecutorMock::expect_check_command_exists()`
/// and provides methods to configure the expectation.
pub struct CheckCommandExistsExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
}

impl<'a> CheckCommandExistsExpectation<'a> {
    /// Adds a matcher predicate for this expectation (currently unused but kept for API compatibility).
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn with<P>(self, _p: P) -> Self {
        self
    }

    /// Specifies how many times this expectation should be called (currently unused but kept for API compatibility).
    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Configures the return value for this expectation.
    ///
    /// The provided function will be called with the command name and should return
    /// `Ok(())` if the command exists, or an error if it doesn't.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a command name and returns a Result
    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        *self.mock.check_command_exists_fn.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

// Type alias for command matcher function
type CommandMatcher = Box<dyn Fn(&str, &[&str]) -> bool + Send + Sync>;
type CommandWithStdinMatcher = Box<dyn Fn(&str, &[&str], &str) -> bool + Send + Sync>;

/// Builder for configuring execute expectations.
///
/// This struct is returned by `MockCommandExecutorMock::expect_execute()`
/// and provides methods to configure the expectation with matchers and return values.
pub struct ExecuteExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
    matcher: Option<CommandMatcher>,
}

impl<'a> ExecuteExpectation<'a> {
    /// Adds a matcher function to verify the command and arguments.
    ///
    /// The matcher function receives the command and arguments and should return
    /// true if they match the expected values.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes command and args and returns true if they match
    #[must_use]
    pub fn withf<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[&str]) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(f));
        self
    }

    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Configures the return value for this expectation.
    ///
    /// The provided function will be called with the command and arguments
    /// and should return a `CommandOutput` with the desired stdout, stderr, and success status.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes command and args and returns a `CommandOutput`
    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str, &[&str]) -> Result<CommandOutput> + Send + Sync + 'static,
    {
        self.mock.execute_fns.lock().unwrap().push(Box::new(f));
        self.mock
    }
}

/// Builder for configuring `execute_with_stdin` expectations.
///
/// This struct is returned by `MockCommandExecutorMock::expect_execute_with_stdin()`
/// and provides methods to configure the expectation with matchers and return values.
pub struct ExecuteWithStdinExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
    matcher: Option<CommandWithStdinMatcher>,
}

impl<'a> ExecuteWithStdinExpectation<'a> {
    /// Adds a matcher function to verify the command, arguments, and stdin input.
    ///
    /// The matcher function receives the command, arguments, and stdin content
    /// and should return true if they match the expected values.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes command, args, and stdin and returns true if they match
    #[must_use]
    pub fn withf<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[&str], &str) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(f));
        self
    }

    /// Specifies how many times this expectation should be called (currently unused)
    #[must_use]
    pub fn times(self, _n: usize) -> Self {
        self
    }

    /// Configures the return value for this expectation.
    ///
    /// The provided function will be called with the command, arguments, and stdin content
    /// and should return a `CommandOutput` with the desired stdout, stderr, and success status.
    ///
    /// # Arguments
    ///
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

/// Creates a set of test credentials for use in tests.
///
/// This function returns a valid `StoredCredentials` struct with test values
/// that can be used when testing authentication-related functionality.
///
/// # Example
///
/// ```rust
/// use ftl_commands::test_helpers::test_credentials;
///
/// let creds = test_credentials();
/// assert_eq!(creds.access_token, "test-token");
/// assert_eq!(creds.authkit_domain, "test.authkit.app");
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

/// Creates a test ECR credentials response for use in tests.
///
/// This function returns a valid `GetEcrCredentialsResponse` with test values
/// that simulate AWS ECR credentials. The authorization token is properly
/// base64-encoded in the format expected by Docker.
///
/// # Example
///
/// ```rust
/// use ftl_commands::test_helpers::test_ecr_credentials;
///
/// let ecr_creds = test_ecr_credentials();
/// assert!(ecr_creds.registry_uri.contains("dkr.ecr.us-east-1.amazonaws.com"));
/// assert!(ecr_creds.authorization_token.len() > 0);
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
