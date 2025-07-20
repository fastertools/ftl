//! Test helper utilities and mock implementations

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use mockall::mock;

use crate::api_client::types;
use crate::commands::login::StoredCredentials as Credentials;
use crate::deps::*;
use base64::Engine;

// Type alias for command matcher function
type CommandMatcher = Box<dyn Fn(&str, &[&str]) -> bool + Send + Sync>;

// Mock implementations using mockall
mock! {
    pub FileSystemMock {}

    impl FileSystem for FileSystemMock {
        fn exists(&self, path: &Path) -> bool;
        fn read_to_string(&self, path: &Path) -> Result<String>;
        fn write_string(&self, path: &Path, content: &str) -> Result<()>;
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

// Simple manual mock implementation for CommandExecutor
// This avoids mockall's issues with async traits containing slice references
use std::sync::{Arc, Mutex};

type CommandCheckFn = dyn Fn(&str) -> Result<()> + Send + Sync;
type CommandExecFn = dyn Fn(&str, &[&str]) -> Result<CommandOutput> + Send + Sync;

pub struct MockCommandExecutorMock {
    check_command_exists_fn: Arc<Mutex<Option<Box<CommandCheckFn>>>>,
    execute_fns: Arc<Mutex<Vec<Box<CommandExecFn>>>>,
    execute_call_count: Arc<Mutex<usize>>,
}

impl MockCommandExecutorMock {
    pub fn new() -> Self {
        Self {
            check_command_exists_fn: Arc::new(Mutex::new(None)),
            execute_fns: Arc::new(Mutex::new(Vec::new())),
            execute_call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn expect_check_command_exists(&mut self) -> CheckCommandExistsExpectation {
        CheckCommandExistsExpectation { mock: self }
    }

    pub fn expect_execute(&mut self) -> ExecuteExpectation {
        ExecuteExpectation {
            mock: self,
            matcher: None,
        }
    }
}

pub struct CheckCommandExistsExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
}

impl<'a> CheckCommandExistsExpectation<'a> {
    pub fn with<P>(self, _p: P) -> Self {
        self
    }

    pub fn times(self, _n: usize) -> Self {
        self
    }

    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        *self.mock.check_command_exists_fn.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

pub struct ExecuteExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
    matcher: Option<CommandMatcher>,
}

impl<'a> ExecuteExpectation<'a> {
    pub fn withf<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[&str]) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(f));
        self
    }

    pub fn times(self, _n: usize) -> Self {
        self
    }

    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str, &[&str]) -> Result<CommandOutput> + Send + Sync + 'static,
    {
        self.mock.execute_fns.lock().unwrap().push(Box::new(f));
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
        _command: &str,
        _args: &[&str],
        _stdin: &str,
    ) -> Result<CommandOutput> {
        Ok(CommandOutput {
            success: true,
            stdout: vec![],
            stderr: vec![],
        })
    }
}

/// Helper to create test credentials
#[allow(dead_code)]
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

/// Helper to create test deployment response
#[allow(dead_code)]
pub fn test_deployment_response(deployment_id: &str) -> types::DeploymentResponse {
    // Use a fixed UUID if the provided ID is not a valid UUID
    let uuid = deployment_id
        .parse()
        .unwrap_or_else(|_| "550e8400-e29b-41d4-a716-446655440000".parse().unwrap());
    types::DeploymentResponse {
        deployment_id: uuid,
        status: types::DeploymentResponseStatus::Accepted,
        message: "Deployment started".to_string(),
        status_url: format!("/v1/deployments/{deployment_id}/status"),
    }
}

/// Helper to create test deployment status
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
#[allow(dead_code)]
pub fn test_repository_response(tool_name: &str) -> types::CreateEcrRepositoryResponse {
    types::CreateEcrRepositoryResponse {
        repository_uri: format!("123456789012.dkr.ecr.us-east-1.amazonaws.com/user/{tool_name}"),
        repository_name: format!("user/{tool_name}"),
        already_exists: false,
    }
}
