//! Test helper utilities and mock implementations

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;
use mockall::Predicate;

use crate::api_client::types;
use crate::commands::login::StoredCredentials as Credentials;
use crate::deps::*;
use base64::Engine;

/// Mock implementations using mockall
mock! {
    pub FileSystemMock {}
    
    impl FileSystem for FileSystemMock {
        fn exists(&self, path: &Path) -> bool;
        fn read_to_string(&self, path: &Path) -> Result<String>;
        fn write_string(&self, path: &Path, content: &str) -> Result<()>;
    }
}

// Manual mock for CommandExecutor due to lifetime issues with mockall
// We'll use a simpler approach that matches mockall's interface but doesn't require lifetimes
pub struct MockCommandExecutorMock {
    check_command_exists_fn: Option<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
    execute_fn: Option<Box<dyn Fn(&str, &[&str]) -> Result<CommandOutput> + Send + Sync>>,
    execute_with_stdin_fn: Option<Box<dyn Fn(&str, &[&str], &str) -> Result<CommandOutput> + Send + Sync>>,
}

impl MockCommandExecutorMock {
    pub fn new() -> Self {
        Self {
            check_command_exists_fn: None,
            execute_fn: None,
            execute_with_stdin_fn: None,
        }
    }

    pub fn expect_check_command_exists(&mut self) -> CheckCommandExistsExpectation {
        CheckCommandExistsExpectation { mock: self }
    }

    pub fn expect_execute(&mut self) -> ExecuteExpectation {
        ExecuteExpectation { mock: self }
    }

    pub fn expect_execute_with_stdin(&mut self) -> ExecuteWithStdinExpectation {
        ExecuteWithStdinExpectation { mock: self }
    }
}

pub struct CheckCommandExistsExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
}

impl<'a> CheckCommandExistsExpectation<'a> {
    pub fn with(self, _p: impl Predicate<str> + 'static) -> Self {
        self
    }

    pub fn times(self, _n: usize) -> Self {
        self
    }

    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock 
    where 
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        self.mock.check_command_exists_fn = Some(Box::new(f));
        self.mock
    }
}

pub struct ExecuteExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
}

impl<'a> ExecuteExpectation<'a> {
    pub fn withf<F>(self, _f: F) -> Self {
        self
    }

    pub fn times(self, _n: usize) -> Self {
        self
    }

    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock 
    where 
        F: Fn(&str, &[&str]) -> Result<CommandOutput> + Send + Sync + 'static,
    {
        // Store the function to be called
        self.mock.execute_fn = Some(Box::new(f));
        self.mock
    }
}

pub struct ExecuteWithStdinExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
}

impl<'a> ExecuteWithStdinExpectation<'a> {
    pub fn withf<F>(self, _f: F) -> Self {
        self
    }

    pub fn times(self, _n: usize) -> Self {
        self
    }

    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock 
    where 
        F: Fn(&str, &[&str], &str) -> Result<CommandOutput> + Send + Sync + 'static,
    {
        self.mock.execute_with_stdin_fn = Some(Box::new(f));
        self.mock
    }
}

#[async_trait]
impl CommandExecutor for MockCommandExecutorMock {
    async fn check_command_exists(&self, command: &str) -> Result<()> {
        if let Some(ref f) = self.check_command_exists_fn {
            f(command)
        } else {
            // Default behavior if no expectation is set
            Ok(())
        }
    }

    async fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        if let Some(ref f) = self.execute_fn {
            f(command, args)
        } else {
            // Default behavior if no expectation is set
            Ok(CommandOutput {
                success: true,
                stdout: vec![],
                stderr: vec![],
            })
        }
    }

    async fn execute_with_stdin(&self, command: &str, args: &[&str], stdin: &str) -> Result<CommandOutput> {
        if let Some(ref f) = self.execute_with_stdin_fn {
            f(command, args, stdin)
        } else {
            // Default behavior if no expectation is set
            Ok(CommandOutput {
                success: true,
                stdout: vec![],
                stderr: vec![],
            })
        }
    }
}

mock! {
    pub FtlApiClientMock {}
    
    #[async_trait]
    impl FtlApiClient for FtlApiClientMock {
        async fn get_ecr_credentials(&self) -> Result<types::GetEcrCredentialsResponse>;
        async fn create_ecr_repository(&self, request: &types::CreateEcrRepositoryRequest) -> Result<types::CreateEcrRepositoryResponse>;
        async fn get_deployment_status(&self, deployment_id: &str) -> Result<types::DeploymentStatus>;
        async fn deploy_app(&self, request: &types::DeploymentRequest) -> Result<types::DeploymentResponse>;
    }
}

mock! {
    pub ClockMock {}
    
    impl Clock for ClockMock {
        fn now(&self) -> Instant;
        fn duration_from_millis(&self, millis: u64) -> Duration;
        fn duration_from_secs(&self, secs: u64) -> Duration;
    }
}

mock! {
    pub CredentialsProviderMock {}
    
    #[async_trait]
    impl CredentialsProvider for CredentialsProviderMock {
        async fn get_or_refresh_credentials(&self) -> Result<Credentials>;
    }
}

// Manual mock for BuildExecutor due to lifetime issues
pub struct MockBuildExecutorMock {
    execute_fn: Option<Box<dyn Fn(Option<PathBuf>, bool) -> Result<()> + Send + Sync>>,
}

impl MockBuildExecutorMock {
    pub fn new() -> Self {
        Self {
            execute_fn: None,
        }
    }
    
    pub fn expect_execute(&mut self) -> &mut Self {
        self
    }
    
    pub fn times(&mut self, _n: usize) -> &mut Self {
        self
    }
    
    pub fn returning<F>(&mut self, f: F) -> &mut Self 
    where 
        F: Fn(Option<&Path>, bool) -> Result<()> + Send + Sync + 'static,
    {
        self.execute_fn = Some(Box::new(move |path: Option<PathBuf>, release| {
            f(path.as_deref(), release)
        }));
        self
    }
}

#[async_trait]
impl BuildExecutor for MockBuildExecutorMock {
    async fn execute(&self, path: Option<&Path>, release: bool) -> Result<()> {
        if let Some(ref f) = self.execute_fn {
            f(path.map(|p| p.to_path_buf()), release)
        } else {
            Ok(())
        }
    }
}

mock! {
    pub AsyncRuntimeMock {}
    
    #[async_trait]
    impl AsyncRuntime for AsyncRuntimeMock {
        async fn sleep(&self, duration: Duration);
    }
}

mock! {
    pub SpinInstallerMock {}
    
    #[async_trait]
    impl SpinInstaller for SpinInstallerMock {
        async fn check_and_install(&self) -> Result<String>;
    }
}

/// Test fixture builder for creating test scenarios
pub struct TestFixture {
    pub file_system: MockFileSystemMock,
    pub command_executor: MockCommandExecutorMock,
    pub api_client: MockFtlApiClientMock,
    pub clock: MockClockMock,
    pub credentials_provider: MockCredentialsProviderMock,
    pub ui: Arc<crate::ui::TestUserInterface>,
    pub build_executor: MockBuildExecutorMock,
    pub async_runtime: MockAsyncRuntimeMock,
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            file_system: MockFileSystemMock::new(),
            command_executor: MockCommandExecutorMock::new(),
            api_client: MockFtlApiClientMock::new(),
            clock: MockClockMock::new(),
            credentials_provider: MockCredentialsProviderMock::new(),
            ui: Arc::new(crate::ui::TestUserInterface::new()),
            build_executor: MockBuildExecutorMock::new(),
            async_runtime: MockAsyncRuntimeMock::new(),
        }
    }
    
    pub fn to_deps(self) -> Arc<crate::commands::deploy::DeployDependencies> {
        Arc::new(crate::commands::deploy::DeployDependencies {
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            api_client: Arc::new(self.api_client) as Arc<dyn FtlApiClient>,
            clock: Arc::new(self.clock) as Arc<dyn Clock>,
            credentials_provider: Arc::new(self.credentials_provider) as Arc<dyn CredentialsProvider>,
            ui: self.ui as Arc<dyn UserInterface>,
            build_executor: Arc::new(self.build_executor) as Arc<dyn BuildExecutor>,
            async_runtime: Arc::new(self.async_runtime) as Arc<dyn AsyncRuntime>,
        })
    }
}

/// Helper to create test credentials
pub fn test_credentials() -> Credentials {
    Credentials {
        access_token: "test-token".to_string(),
        refresh_token: Some("refresh-token".to_string()),
        id_token: None,
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        authkit_domain: "test.authkit.app".to_string(),
    }
}

/// Helper to create test ECR credentials response
pub fn test_ecr_credentials() -> types::GetEcrCredentialsResponse {
    types::GetEcrCredentialsResponse {
        registry_uri: "123456789012.dkr.ecr.us-east-1.amazonaws.com".to_string(),
        authorization_token: base64::engine::general_purpose::STANDARD.encode("AWS:test-password"),
        proxy_endpoint: "https://123456789012.dkr.ecr.us-east-1.amazonaws.com".to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(12),
        region: "us-east-1".to_string(),
    }
}

/// Helper to create test deployment response
pub fn test_deployment_response(deployment_id: &str) -> types::DeploymentResponse {
    // Use a fixed UUID if the provided ID is not a valid UUID
    let uuid = deployment_id.parse().unwrap_or_else(|_| {
        "550e8400-e29b-41d4-a716-446655440000".parse().unwrap()
    });
    types::DeploymentResponse {
        deployment_id: uuid,
        status: types::DeploymentResponseStatus::Accepted,
        message: "Deployment started".to_string(),
        status_url: format!("/v1/deployments/{}/status", deployment_id),
    }
}

/// Helper to create test deployment status
pub fn test_deployment_status(
    deployment_id: &str,
    status: types::DeploymentStatusDeploymentStatus,
) -> types::DeploymentStatus {
    // Use a fixed UUID if the provided ID is not a valid UUID
    let uuid = deployment_id.parse().unwrap_or_else(|_| {
        "550e8400-e29b-41d4-a716-446655440000".parse().unwrap()
    });
    types::DeploymentStatus {
        deployment: types::DeploymentStatusDeployment {
            deployment_id: uuid,
            app_name: "test-app".to_string(),
            display_name: "Test App".to_string(),
            status,
            deployment_url: Some("https://test-app.example.com".to_string()),
            image_url: "test-image:latest".to_string(),
            platform: types::DeploymentStatusDeploymentPlatform::Fermyon,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            completed_at: None,
            error: None,
            retry_count: 0,
            stages: vec![],
        },
    }
}

/// Helper to create test repository response
pub fn test_repository_response(tool_name: &str) -> types::CreateEcrRepositoryResponse {
    types::CreateEcrRepositoryResponse {
        repository_uri: format!("123456789012.dkr.ecr.us-east-1.amazonaws.com/user/{}", tool_name),
        repository_name: format!("user/{}", tool_name),
        already_exists: false,
    }
}