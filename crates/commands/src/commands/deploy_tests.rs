//! Comprehensive unit tests for `deploy_v2` module

use crate::commands::deploy::*;
use crate::test_helpers::*;
use anyhow::{Result, anyhow};
use ftl_common::ui::TestUserInterface;
use ftl_runtime::api_client::types;
use ftl_runtime::deps::*;
use mockall::predicate::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct TestFixture {
    file_system: MockFileSystemMock,
    command_executor: MockCommandExecutorMock,
    api_client: MockFtlApiClientMock,
    clock: MockClockMock,
    credentials_provider: MockCredentialsProviderMock,
    ui: Arc<TestUserInterface>,
    build_executor: Arc<MockBuildExecutor>,
    async_runtime: MockAsyncRuntimeMock,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            file_system: MockFileSystemMock::new(),
            command_executor: MockCommandExecutorMock::new(),
            api_client: MockFtlApiClientMock::new(),
            clock: MockClockMock::new(),
            credentials_provider: MockCredentialsProviderMock::new(),
            ui: Arc::new(TestUserInterface::new()),
            build_executor: Arc::new(MockBuildExecutor::new()),
            async_runtime: MockAsyncRuntimeMock::new(),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<DeployDependencies> {
        Arc::new(DeployDependencies {
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            api_client: Arc::new(self.api_client) as Arc<dyn FtlApiClient>,
            clock: Arc::new(self.clock) as Arc<dyn Clock>,
            credentials_provider: Arc::new(self.credentials_provider)
                as Arc<dyn CredentialsProvider>,
            ui: self.ui as Arc<dyn UserInterface>,
            build_executor: self.build_executor as Arc<dyn BuildExecutor>,
            async_runtime: Arc::new(self.async_runtime) as Arc<dyn AsyncRuntime>,
        })
    }
}

// Mock implementation of BuildExecutor
struct MockBuildExecutor {
    should_fail: bool,
    error_message: Option<String>,
}

impl MockBuildExecutor {
    fn new() -> Self {
        Self {
            should_fail: false,
            error_message: None,
        }
    }

    #[allow(dead_code)]
    fn with_failure(mut self, message: &str) -> Self {
        self.should_fail = true;
        self.error_message = Some(message.to_string());
        self
    }
}

#[async_trait::async_trait]
impl BuildExecutor for MockBuildExecutor {
    async fn execute(&self, _path: Option<&Path>, _release: bool) -> Result<()> {
        if self.should_fail {
            Err(anyhow::anyhow!(
                self.error_message
                    .clone()
                    .unwrap_or_else(|| "Build failed".to_string())
            ))
        } else {
            Ok(())
        }
    }
}

/// Helper to set up basic ftl.toml existence mocks
fn setup_project_file_mocks(fixture: &mut TestFixture, has_ftl_toml: bool) {
    setup_project_file_mocks_with_content(fixture, has_ftl_toml, None);
}

/// Helper to set up project file mocks with custom ftl.toml content
fn setup_project_file_mocks_with_content(
    fixture: &mut TestFixture,
    has_ftl_toml: bool,
    ftl_content: Option<String>,
) {
    // With generate_temp_spin_toml, we check for ftl.toml
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(move |_| has_ftl_toml);

    // If ftl.toml exists, we read it to generate temp spin.toml
    if has_ftl_toml {
        let content = ftl_content.unwrap_or_else(|| {
            r#"[project]
name = "test-project"
version = "0.1.0"

[tools.test-tool]
path = "test"
wasm = "test/test-tool.wasm"

[tools.test-tool.build]
command = "echo 'Building test tool'"
"#
            .to_string()
        });

        // Clone content for both mocks
        let content_for_transpiler = content.clone();
        let content_for_build = content.clone();

        // Mock: read ftl.toml for temp spin.toml generation
        fixture
            .file_system
            .expect_read_to_string()
            .with(eq(Path::new("./ftl.toml")))
            .times(1)
            .returning(move |_| Ok(content_for_transpiler.clone()));

        // Mock: read ftl.toml again to determine build profiles
        fixture
            .file_system
            .expect_read_to_string()
            .with(eq(Path::new("ftl.toml")))
            .times(1)
            .returning(move |_| Ok(content_for_build.clone()));

        // Note: We don't mock the temp file creation since it happens outside our FileSystem trait
    }
    // No else clause - we always require ftl.toml now
}

#[tokio::test]
async fn test_deploy_no_ftl_toml() {
    let mut fixture = TestFixture::new();

    // No ftl.toml - should fail
    setup_project_file_mocks(&mut fixture, false);

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No ftl.toml found")
    );
}

#[tokio::test]
async fn test_deploy_authentication_expired() {
    let mut fixture = TestFixture::new();

    // Set up project files - no ftl.toml, but spin.toml exists
    setup_project_file_mocks(&mut fixture, true);

    // Mock: clock for progress bar
    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    // Build executor succeeds (it's not a mock, just a test implementation)

    // Mock: credentials expired
    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .times(1)
        .returning(|| Err(anyhow::anyhow!("Token expired")));

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Authentication token has expired")
    );
}

#[tokio::test]
async fn test_deploy_no_components() {
    let mut fixture = TestFixture::new();

    // Set up project files - ftl.toml exists but with no tools
    setup_project_file_mocks_with_content(
        &mut fixture,
        true,
        Some(
            r#"[project]
name = "test-app"
version = "0.1.0"
"#
            .to_string(),
        ),
    );

    // Mock: clock for progress bar
    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    // Build executor succeeds (it's not a mock, just a test implementation)

    // Mock: credentials succeed
    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .times(1)
        .returning(|| Ok(test_credentials()));

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No user components found")
    );
}

#[tokio::test]
async fn test_deploy_docker_login_failure() {
    let mut fixture = TestFixture::new();

    // Setup basic mocks
    setup_basic_mocks(&mut fixture);

    // Mock: get ECR credentials
    fixture
        .api_client
        .expect_create_ecr_token()
        .times(1)
        .returning(|| Ok(test_ecr_credentials()));

    // Mock: docker login fails
    fixture
        .command_executor
        .expect_execute_with_stdin()
        .withf(|cmd: &str, args: &[&str], _stdin: &str| cmd == "docker" && args.contains(&"login"))
        .times(1)
        .returning(|_, _, _| {
            Ok(CommandOutput {
                success: false,
                stdout: vec![],
                stderr: b"Login failed".to_vec(),
            })
        });

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Docker login failed")
    );
}

#[tokio::test]
async fn test_deploy_wkg_not_found() {
    let mut fixture = TestFixture::new();

    // Setup basic mocks including successful docker login
    setup_basic_mocks(&mut fixture);
    setup_docker_login_success(&mut fixture);

    // Mock: list apps returns empty (app doesn't exist)
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    // Mock: create app succeeds
    let app_id = uuid::Uuid::new_v4();
    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(move |_| {
            Ok(types::CreateAppResponse {
                app_id,
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: wkg not found
    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("wkg"))
        .times(1)
        .returning(|_| Err(anyhow::anyhow!("wkg not found")));

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("wkg not found"));
}

#[tokio::test]
async fn test_deploy_repository_creation_failure() {
    let mut fixture = TestFixture::new();

    // Setup all basic mocks
    setup_full_mocks(&mut fixture);

    // Mock: list apps returns empty (app doesn't exist)
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    // Mock: create app succeeds
    let app_id = uuid::Uuid::new_v4();
    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(move |_| {
            Ok(types::CreateAppResponse {
                app_id,
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: component update fails
    fixture
        .api_client
        .expect_update_components()
        .times(1)
        .returning(|_, _| Err(anyhow::anyhow!("Failed to update components")));

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to update components")
    );
}

#[tokio::test]
async fn test_deploy_success() {
    let mut fixture = TestFixture::new();

    // Setup all mocks for successful deployment
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);
    setup_successful_deployment(&mut fixture);

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Deployed!")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("https://test-app.example.com"))
    );
}

#[tokio::test]
async fn test_parse_component_versions() {
    let mut fs = MockFileSystemMock::new();

    // Mock Cargo.toml with version
    fs.expect_exists()
        .with(eq(Path::new("api/Cargo.toml")))
        .returning(|_| true);
    fs.expect_read_to_string()
        .with(eq(Path::new("api/Cargo.toml")))
        .returning(|_| {
            Ok(r#"
[package]
name = "api"
version = "1.2.3"
"#
            .to_string())
        });

    let fs: Arc<dyn FileSystem> = Arc::new(fs);
    let version = extract_component_version(&fs, "api", "api/api.wasm").unwrap();
    assert_eq!(version, "1.2.3");
}

#[tokio::test]
async fn test_parse_package_json_version() {
    let mut fs = MockFileSystemMock::new();

    // Mock: Cargo.toml doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("worker/Cargo.toml")))
        .returning(|_| false);

    // Mock: package.json exists
    fs.expect_exists()
        .with(eq(Path::new("worker/package.json")))
        .returning(|_| true);
    fs.expect_read_to_string()
        .with(eq(Path::new("worker/package.json")))
        .returning(|_| Ok(r#"{"name": "worker", "version": "2.0.0"}"#.to_string()));

    let fs: Arc<dyn FileSystem> = Arc::new(fs);
    let version = extract_component_version(&fs, "worker", "worker/worker.wasm").unwrap();
    assert_eq!(version, "2.0.0");
}

#[tokio::test]
async fn test_deployment_timeout() {
    let mut fixture = TestFixture::new();

    // Setup all mocks for successful push
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);

    // Mock: list apps returns empty (app doesn't exist)
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    // Mock: create app succeeds
    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(|_| {
            Ok(types::CreateAppResponse {
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: create deployment succeeds
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(|_, req| {
            // Verify we have at least one component
            assert!(!req.components.is_empty());

            Ok(types::CreateDeploymentResponse {
                deployment_id: uuid::Uuid::new_v4(),
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: "DEPLOYING".to_string(),
                message: "Deployment started".to_string(),
            })
        });

    // Mock: status always returns "Creating" (60 times = timeout)
    fixture
        .api_client
        .expect_get_app()
        .times(60)
        .returning(|_| {
            Ok(types::App {
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::AppStatus::Creating,
                provider_url: None,
                provider_error: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: async sleep
    fixture
        .async_runtime
        .expect_sleep()
        .times(60)
        .returning(|_| ());

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Engine deployment timeout")
    );
}

#[tokio::test]
async fn test_deployment_failed_status() {
    let mut fixture = TestFixture::new();

    // Setup all mocks for successful push
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);

    // Mock: list apps returns empty (app doesn't exist)
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    // Mock: create app succeeds
    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(|_| {
            Ok(types::CreateAppResponse {
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: create deployment succeeds
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(|_, req| {
            // Verify we have at least one component
            assert!(!req.components.is_empty());

            Ok(types::CreateDeploymentResponse {
                deployment_id: uuid::Uuid::new_v4(),
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: "DEPLOYING".to_string(),
                message: "Deployment started".to_string(),
            })
        });

    // Mock: status returns failed
    fixture.api_client.expect_get_app().times(1).returning(|_| {
        Ok(types::App {
            app_id: uuid::Uuid::new_v4(),
            app_name: "test-app".to_string(),
            status: types::AppStatus::Failed,
            provider_url: None,
            provider_error: Some("Build failed".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        })
    });

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Engine deployment failed: Build failed")
    );
}

// Helper functions to setup common mock scenarios

fn setup_basic_mocks(fixture: &mut TestFixture) {
    // Set up project files - ftl.toml exists with a tool
    setup_project_file_mocks(fixture, true);

    // Mock: check for ftl.toml again when extracting auth variables
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(0..=1)
        .returning(|_| true);

    // Mock: read ftl.toml for auth variables extraction
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(0..=1)
        .returning(|_| {
            Ok(r#"[project]
name = "test-project"
version = "0.1.0"

[tools.test-tool]
path = "test"
wasm = "test/test-tool.wasm"

[tools.test-tool.build]
command = "echo 'Building test tool'"
"#
            .to_string())
        });

    // Note: We don't need to mock reading spin.toml because parse_deploy_config
    // reads temporary files directly from the filesystem

    // Mock: component version files don't exist (use default)
    // Be more specific about which paths we're mocking
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| path.ends_with("Cargo.toml") || path.ends_with("package.json"))
        .returning(|_| false);

    // Mock: clock for progress bars
    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    fixture
        .clock
        .expect_duration_from_secs()
        .returning(Duration::from_secs);

    fixture.clock.expect_now().returning(Instant::now);

    // Build executor succeeds (it's not a mock, just a test implementation)

    // Mock: credentials succeed (called multiple times in deploy flow)
    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .returning(|| Ok(test_credentials()));

    // ECR credentials mock is set up separately in tests that need it
}

fn setup_docker_login_success(fixture: &mut TestFixture) {
    // Mock: get ECR credentials
    fixture
        .api_client
        .expect_create_ecr_token()
        .times(1)
        .returning(|| Ok(test_ecr_credentials()));

    // Mock: docker login succeeds
    fixture
        .command_executor
        .expect_execute_with_stdin()
        .withf(|cmd: &str, args: &[&str], _stdin: &str| cmd == "docker" && args.contains(&"login"))
        .times(1)
        .returning(|_, _, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Login Succeeded".to_vec(),
                stderr: vec![],
            })
        });
}

fn setup_full_mocks(fixture: &mut TestFixture) {
    setup_basic_mocks(fixture);
    setup_docker_login_success(fixture);

    // Mock: wkg exists
    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("wkg"))
        .times(1)
        .returning(|_| Ok(()));
}

fn setup_successful_push(fixture: &mut TestFixture) {
    // Mock: update components succeeds and returns repository URIs
    fixture
        .api_client
        .expect_update_components()
        .times(1)
        .returning(|_, _| {
            Ok(types::UpdateComponentsResponse {
                components: vec![types::UpdateComponentsResponseComponentsItem {
                    component_name: "test-tool".to_string(),
                    description: None,
                    repository_uri: Some(
                        "123456789012.dkr.ecr.us-east-1.amazonaws.com/user/test-tool".to_string(),
                    ),
                    repository_name: Some("user/test-tool".to_string()),
                }],
                changes: types::UpdateComponentsResponseChanges {
                    created: vec!["test-tool".to_string()],
                    updated: vec![],
                    removed: vec![],
                },
            })
        });

    // Mock: wkg push succeeds (version tag only)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "wkg" && args.contains(&"push"))
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Pushed".to_vec(),
                stderr: vec![],
            })
        });
}

fn setup_successful_push_for_api(fixture: &mut TestFixture) {
    // Mock: update components succeeds and returns repository URIs for "api" component
    fixture
        .api_client
        .expect_update_components()
        .times(1)
        .returning(|_, _| {
            Ok(types::UpdateComponentsResponse {
                components: vec![types::UpdateComponentsResponseComponentsItem {
                    component_name: "api".to_string(),
                    description: None,
                    repository_uri: Some(
                        "123456789012.dkr.ecr.us-east-1.amazonaws.com/user/api".to_string(),
                    ),
                    repository_name: Some("user/api".to_string()),
                }],
                changes: types::UpdateComponentsResponseChanges {
                    created: vec!["api".to_string()],
                    updated: vec![],
                    removed: vec![],
                },
            })
        });

    // Mock: wkg push succeeds (version tag only)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "wkg" && args.contains(&"push"))
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Pushed".to_vec(),
                stderr: vec![],
            })
        });
}

fn setup_successful_deployment(fixture: &mut TestFixture) {
    // Mock: list apps returns empty (app doesn't exist)
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    // Mock: create app succeeds
    let app_id = uuid::Uuid::new_v4();
    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(move |_| {
            Ok(types::CreateAppResponse {
                app_id,
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: create deployment succeeds
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(|_, req| {
            // Verify we have at least one component
            assert!(!req.components.is_empty());

            Ok(types::CreateDeploymentResponse {
                deployment_id: uuid::Uuid::new_v4(),
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: "DEPLOYING".to_string(),
                message: "Deployment started".to_string(),
            })
        });

    // Mock: app status checks - first returns creating, then active
    let call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    let call_count_clone = call_count.clone();
    fixture
        .api_client
        .expect_get_app()
        .times(2)
        .returning(move |_| {
            let mut count = call_count_clone.lock().unwrap();
            *count += 1;
            if *count == 1 {
                Ok(types::App {
                    app_id,
                    app_name: "test-app".to_string(),
                    status: types::AppStatus::Creating,
                    provider_url: None,
                    provider_error: None,
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                    updated_at: "2024-01-01T00:00:00Z".to_string(),
                })
            } else {
                Ok(types::App {
                    app_id,
                    app_name: "test-app".to_string(),
                    status: types::AppStatus::Active,
                    provider_url: Some("https://test-app.example.com".to_string()),
                    provider_error: None,
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                    updated_at: "2024-01-01T00:00:00Z".to_string(),
                })
            }
        });

    // Mock: async sleep (called once)
    fixture
        .async_runtime
        .expect_sleep()
        .times(1)
        .returning(|_| ());
}

// Mock implementations for testing
struct MockFileSystem {
    files: HashMap<PathBuf, String>,
}

impl MockFileSystem {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    fn add_file(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
        self.files.insert(path.into(), content.into());
    }
}

impl FileSystem for MockFileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow!("File not found: {}", path.display()))
    }

    fn write_string(&self, _path: &Path, _content: &str) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_parse_deploy_config() {
    let mut fs = MockFileSystem::new();
    fs.add_file(
        "spin.toml",
        r#"
[application]
name = "test-app"

[component.api]
source = "api/target/wasm32-wasi/release/api.wasm"
allowed_outbound_hosts = ["https://*.amazonaws.com"]

[component.worker]
source = "worker/target/wasm32-wasi/release/worker.wasm"

[component.system]
source = { registry = "ghcr.io/example/system:latest" }
"#,
    );
    // Add version files in the expected locations
    fs.add_file(
        "api/Cargo.toml",
        r#"
[package]
name = "api"
version = "1.2.3"
"#,
    );
    fs.add_file(
        "worker/package.json",
        r#"{"name": "worker", "version": "2.0.0"}"#,
    );

    let fs: Arc<dyn FileSystem> = Arc::new(fs);
    let config = parse_deploy_config(&fs, Path::new("spin.toml")).unwrap();

    assert_eq!(config.app_name, "test-app");
    assert_eq!(config.components.len(), 2);

    let api = &config.components[0];
    assert_eq!(api.name, "api");
    assert_eq!(api.source_path, "api/target/wasm32-wasi/release/api.wasm");
    assert_eq!(api.version, "1.2.3");
    assert_eq!(
        api.allowed_outbound_hosts,
        Some(vec!["https://*.amazonaws.com".to_string()])
    );

    let worker = &config.components[1];
    assert_eq!(worker.name, "worker");
    assert_eq!(
        worker.source_path,
        "worker/target/wasm32-wasi/release/worker.wasm"
    );
    assert_eq!(worker.version, "2.0.0");
    assert_eq!(worker.allowed_outbound_hosts, None);
}

#[test]
fn test_extract_component_version_default() {
    let fs: Arc<dyn FileSystem> = Arc::new(MockFileSystem::new());
    let version = extract_component_version(&fs, "test", "test.wasm").unwrap();
    assert_eq!(version, "0.1.0");
}

#[test]
fn test_parse_variables() {
    // Test valid variable formats
    let vars = vec![
        "KEY=value".to_string(),
        "API_KEY=12345".to_string(),
        "DEBUG=true".to_string(),
    ];
    let parsed = parse_variables(&vars).unwrap();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed.get("KEY"), Some(&"value".to_string()));
    assert_eq!(parsed.get("API_KEY"), Some(&"12345".to_string()));
    assert_eq!(parsed.get("DEBUG"), Some(&"true".to_string()));

    // Test variable with equals sign in value
    let vars = vec!["URL=https://example.com?key=value".to_string()];
    let parsed = parse_variables(&vars).unwrap();
    assert_eq!(
        parsed.get("URL"),
        Some(&"https://example.com?key=value".to_string())
    );

    // Test invalid format (no equals sign)
    let vars = vec!["INVALID".to_string()];
    let result = parse_variables(&vars);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid variable format")
    );

    // Test empty key
    let vars = vec!["=value".to_string()];
    let result = parse_variables(&vars);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Variable key cannot be empty")
    );
}

#[tokio::test]
async fn test_deploy_with_variables() {
    let mut fixture = TestFixture::new();

    // Setup all mocks for successful deployment
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);

    // Verify variables are passed through to deployment request
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(|_| {
            Ok(types::CreateAppResponse {
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: create deployment with variables
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(|_, req| {
            // Verify variables are passed correctly
            assert!(req.variables.contains_key("API_KEY"));
            assert_eq!(req.variables.get("API_KEY"), Some(&"test123".to_string()));

            Ok(types::CreateDeploymentResponse {
                deployment_id: uuid::Uuid::new_v4(),
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: "DEPLOYING".to_string(),
                message: "Deployment started".to_string(),
            })
        });

    // Mock: app becomes active
    fixture.api_client.expect_get_app().times(1).returning(|_| {
        Ok(types::App {
            app_id: uuid::Uuid::new_v4(),
            app_name: "test-app".to_string(),
            status: types::AppStatus::Active,
            provider_url: Some("https://test-app.example.com".to_string()),
            provider_error: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        })
    });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        deps,
        deploy_args_with_variables(vec!["API_KEY=test123".to_string()]),
    )
    .await;

    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Deployed!")));
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_deploy_with_auth_from_ftl_toml() {
    let mut fixture = TestFixture::new();

    // Mock: Check for ftl.toml (for generate_temp_spin_toml)
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: read ftl.toml with auth configuration
    let ftl_content = r#"
[project]
name = "test-app"
version = "0.1.0"

[auth]
enabled = true

[auth.authkit]
issuer = "https://test.authkit.app"
audience = "my-api"

[tools.api]
path = "api"
wasm = "api/target/wasm32-wasip1/release/api.wasm"
allowed_outbound_hosts = ["https://*.amazonaws.com"]

[tools.api.build]
command = "cargo build --release --target wasm32-wasip1"
"#
    .to_string();

    // Mock: read ftl.toml for temp spin.toml generation
    let ftl_content_clone = ftl_content.clone();
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(move |_| Ok(ftl_content_clone.clone()));

    // Mock: read ftl.toml again to determine build profiles
    let ftl_content_for_build = ftl_content.clone();
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(move |_| Ok(ftl_content_for_build.clone()));

    // Mock: check for ftl.toml again when extracting auth variables
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: read ftl.toml again for auth variables extraction
    let ftl_content2 = ftl_content.clone();
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(move |_| Ok(ftl_content2.clone()));

    // Mock: component version files don't exist (use default)
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| path.ends_with("Cargo.toml") || path.ends_with("package.json"))
        .returning(|_| false);

    // Mock: clock for progress bars
    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    fixture
        .clock
        .expect_duration_from_secs()
        .returning(Duration::from_secs);

    fixture.clock.expect_now().returning(Instant::now);

    // Mock: credentials succeed (called multiple times in deploy flow)
    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .returning(|| Ok(test_credentials()));

    setup_docker_login_success(&mut fixture);

    // Mock: wkg exists
    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("wkg"))
        .times(1)
        .returning(|_| Ok(()));

    setup_successful_push_for_api(&mut fixture);

    // Verify auth variables are passed through to deployment request
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(|_| {
            Ok(types::CreateAppResponse {
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: create deployment with auth variables
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(|_, req| {
            // Verify auth variables are passed correctly
            assert!(req.variables.contains_key("auth_enabled"));
            assert_eq!(req.variables.get("auth_enabled"), Some(&"true".to_string()));

            assert!(req.variables.contains_key("auth_provider_type"));
            assert_eq!(
                req.variables.get("auth_provider_type"),
                Some(&"authkit".to_string())
            );

            assert!(req.variables.contains_key("auth_provider_issuer"));
            assert_eq!(
                req.variables.get("auth_provider_issuer"),
                Some(&"https://test.authkit.app".to_string())
            );

            assert!(req.variables.contains_key("auth_provider_audience"));
            assert_eq!(
                req.variables.get("auth_provider_audience"),
                Some(&"my-api".to_string())
            );

            // Check that OIDC variables are not set (since we're using authkit)
            assert!(
                !req.variables.contains_key("auth_provider_name")
                    || req.variables.get("auth_provider_name") == Some(&String::new())
            );

            Ok(types::CreateDeploymentResponse {
                deployment_id: uuid::Uuid::new_v4(),
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: "DEPLOYING".to_string(),
                message: "Deployment started".to_string(),
            })
        });

    // Mock: app becomes active
    fixture.api_client.expect_get_app().times(1).returning(|_| {
        Ok(types::App {
            app_id: uuid::Uuid::new_v4(),
            app_name: "test-app".to_string(),
            status: types::AppStatus::Active,
            provider_url: Some("https://test-app.example.com".to_string()),
            provider_error: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        })
    });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Deployed!")));
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_deploy_cli_variables_override_ftl_toml() {
    let mut fixture = TestFixture::new();

    // Mock: Check for ftl.toml (for generate_temp_spin_toml)
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: read ftl.toml with auth configuration
    let ftl_content = r#"
[project]
name = "test-app"
version = "0.1.0"

[auth]
enabled = true

[auth.authkit]
issuer = "https://test.authkit.app"
audience = "my-api"

[tools.api]
path = "api"
wasm = "api/target/wasm32-wasip1/release/api.wasm"
allowed_outbound_hosts = ["https://*.amazonaws.com"]

[tools.api.build]
command = "cargo build --release --target wasm32-wasip1"
"#
    .to_string();

    // Mock: read ftl.toml for temp spin.toml generation
    let ftl_content_clone = ftl_content.clone();
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(move |_| Ok(ftl_content_clone.clone()));

    // Mock: read ftl.toml again to determine build profiles
    let ftl_content_for_build = ftl_content.clone();
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(move |_| Ok(ftl_content_for_build.clone()));

    // Mock: check for ftl.toml again when extracting auth variables
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: read ftl.toml again for auth variables extraction
    let ftl_content2 = ftl_content.clone();
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(move |_| Ok(ftl_content2.clone()));

    // Mock: component version files don't exist (use default)
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| path.ends_with("Cargo.toml") || path.ends_with("package.json"))
        .returning(|_| false);

    // Mock: clock for progress bars
    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    fixture
        .clock
        .expect_duration_from_secs()
        .returning(Duration::from_secs);

    fixture.clock.expect_now().returning(Instant::now);

    // Mock: credentials succeed (called multiple times in deploy flow)
    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .returning(|| Ok(test_credentials()));

    setup_docker_login_success(&mut fixture);

    // Mock: wkg exists
    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("wkg"))
        .times(1)
        .returning(|_| Ok(()));

    setup_successful_push_for_api(&mut fixture);

    // Verify CLI variables override ftl.toml values
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(|_| {
            Ok(types::CreateAppResponse {
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: create deployment with auth variables
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(|_, req| {
            // Verify CLI override values are used instead of ftl.toml values
            assert!(req.variables.contains_key("auth_enabled"));
            assert_eq!(
                req.variables.get("auth_enabled"),
                Some(&"false".to_string())
            ); // CLI override

            assert!(req.variables.contains_key("auth_provider_issuer"));
            assert_eq!(
                req.variables.get("auth_provider_issuer"),
                Some(&"https://override.authkit.app".to_string())
            ); // CLI override

            // ftl.toml values should still be present for non-overridden variables
            assert!(req.variables.contains_key("auth_provider_type"));
            assert_eq!(
                req.variables.get("auth_provider_type"),
                Some(&"authkit".to_string())
            );

            Ok(types::CreateDeploymentResponse {
                deployment_id: uuid::Uuid::new_v4(),
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: "DEPLOYING".to_string(),
                message: "Deployment started".to_string(),
            })
        });

    // Mock: app becomes active
    fixture.api_client.expect_get_app().times(1).returning(|_| {
        Ok(types::App {
            app_id: uuid::Uuid::new_v4(),
            app_name: "test-app".to_string(),
            status: types::AppStatus::Active,
            provider_url: Some("https://test-app.example.com".to_string()),
            provider_error: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        })
    });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    // Pass CLI variables that should override ftl.toml values
    let result = execute_with_deps(
        deps,
        deploy_args_with_variables(vec![
            "auth_enabled=false".to_string(),
            "auth_provider_issuer=https://override.authkit.app".to_string(),
        ]),
    )
    .await;

    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Deployed!")));
}

// Helper function to create default DeployArgs for tests
fn default_deploy_args() -> DeployArgs {
    DeployArgs {
        variables: vec![],
        auth_mode: None,
        auth_users: None,
        auth_provider: None,
        auth_issuer: None,
        auth_audience: None,
    }
}

// Helper function to create DeployArgs with variables
fn deploy_args_with_variables(variables: Vec<String>) -> DeployArgs {
    DeployArgs {
        variables,
        auth_mode: None,
        auth_users: None,
        auth_provider: None,
        auth_issuer: None,
        auth_audience: None,
    }
}

#[tokio::test]
async fn test_auth_config_updated_before_deployment() {
    use std::sync::Mutex;

    let mut fixture = TestFixture::new();
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);

    // Track the order of API calls
    let call_order = Arc::new(Mutex::new(vec![]));
    let call_order_clone1 = call_order.clone();
    let call_order_clone2 = call_order.clone();

    // Mock: auth config update happens BEFORE deployment
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(move |app_id, request| {
            call_order_clone1.lock().unwrap().push("update_auth_config");
            assert_eq!(app_id, "12345678-1234-1234-1234-123456789012");
            match request.mode {
                types::UpdateAuthConfigRequestMode::Public => {}
                _ => panic!("Expected public mode"),
            }
            // We don't care about the response structure for this test
            // Just tracking that the call happened in the right order
            // Return error to avoid dealing with complex generated types
            Err(anyhow!("Test succeeded - auth config was called"))
        });

    // Mock: deployment happens AFTER auth config
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(move |_, _| {
            call_order_clone2.lock().unwrap().push("create_deployment");
            Ok(types::CreateDeploymentResponse {
                deployment_id: uuid::Uuid::new_v4(),
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: "DEPLOYING".to_string(),
                message: "Deployment started".to_string(),
            })
        });

    // Mock: list apps returns existing app
    fixture
        .api_client
        .expect_list_apps()
        .times(1)
        .returning(|_, _, _| {
            Ok(types::ListAppsResponse {
                apps: vec![],
                next_token: None,
            })
        });

    // Mock: create app
    fixture
        .api_client
        .expect_create_app()
        .times(1)
        .returning(|_| {
            Ok(types::CreateAppResponse {
                app_id: uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: get app status
    fixture.api_client.expect_get_app().times(1).returning(|_| {
        Ok(types::App {
            app_id: uuid::Uuid::new_v4(),
            app_name: "test-app".to_string(),
            status: types::AppStatus::Active,
            provider_url: Some("https://test-app.example.com".to_string()),
            provider_error: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        })
    });

    let call_order_final = call_order.clone();
    let deps = fixture.to_deps();

    // Deploy with auth configuration
    let args = DeployArgs {
        variables: vec![],
        auth_mode: Some("public".to_string()),
        auth_users: None,
        auth_provider: None,
        auth_issuer: None,
        auth_audience: None,
    };

    let result = execute_with_deps(deps, args).await;

    // The test will fail because auth config returns an error, but that's okay
    // We're only interested in verifying the call order
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("auth config was called")
    );

    // Verify that auth config was updated BEFORE deployment (or attempted to)
    let calls = call_order_final.lock().unwrap();
    assert!(!calls.is_empty());
    assert_eq!(
        calls[0], "update_auth_config",
        "Auth config should be updated first"
    );
    // Deployment won't happen because auth config failed, which is fine for this test
}
