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

/// Setup for file system mocks that handles all ftl.toml access patterns
/// This function sets up mocks for:
/// 1. deploy checking if ftl.toml exists
/// 2. deploy reading ftl.toml for `FtlConfig` parsing
/// 3. `parse_deploy_config` reading the generated spin.toml
/// 4. Component version file checks
fn setup_comprehensive_ftl_mocks(fixture: &mut TestFixture, ftl_toml_content: &str) {
    // Parse ftl config to generate expected spin.toml content
    let ftl_config = crate::config::ftl_config::FtlConfig::parse(ftl_toml_content).unwrap();
    let resolved_mappings = std::collections::HashMap::new();
    let project_path = std::path::Path::new(".");
    let expected_spin_content = crate::config::transpiler::create_spin_toml_with_resolved_paths(
        &ftl_config,
        &resolved_mappings,
        project_path,
    )
    .unwrap();

    // Mock for any existence check of ftl.toml (with or without ./ prefix)
    let _ftl_content_for_exists = ftl_toml_content.to_string();
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| {
            let path_str = path.to_string_lossy();
            path_str == "./ftl.toml" || path_str == "ftl.toml"
        })
        .returning(move |path| {
            let path_str = path.to_string_lossy();
            path_str == "./ftl.toml" || path_str == "ftl.toml"
        });

    // Mock for any read of ftl.toml (with or without ./ prefix)
    let ftl_content_for_read = ftl_toml_content.to_string();
    fixture
        .file_system
        .expect_read_to_string()
        .withf(|path: &Path| {
            let path_str = path.to_string_lossy();
            path_str == "./ftl.toml" || path_str == "ftl.toml"
        })
        .returning(move |_| Ok(ftl_content_for_read.clone()));

    // Mock for parse_deploy_config reading the generated spin.toml
    fixture
        .file_system
        .expect_read_to_string()
        .withf(|path: &Path| path.to_string_lossy().ends_with("spin.toml"))
        .returning(move |_| Ok(expected_spin_content.clone()));

    // Mock component version file checks - handle all possible paths
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| {
            let path_str = path.to_string_lossy();
            path_str.contains("Cargo.toml")
                || path_str.contains("package.json")
                || path_str.contains("pyproject.toml")
                || path_str.contains("go.mod")
        })
        .returning(|_| false);
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

[component.test-tool]
path = "test"
wasm = "test/test-tool.wasm"

[component.test-tool.build]
command = "echo 'Building test tool'"
"#
            .to_string()
        });

        // Clone content for mocks
        let content_for_read = content.clone();

        // Mock: read ftl.toml for FtlConfig parsing
        fixture
            .file_system
            .expect_read_to_string()
            .with(eq(Path::new("./ftl.toml")))
            .times(1)
            .returning(move |_| Ok(content_for_read.clone()));

        // Generate expected spin.toml content and mock its reading
        let ftl_config = crate::config::ftl_config::FtlConfig::parse(&content).unwrap();
        let resolved_mappings = std::collections::HashMap::new();
        let project_path = std::path::Path::new(".");
        let expected_spin_content =
            crate::config::transpiler::create_spin_toml_with_resolved_paths(
                &ftl_config,
                &resolved_mappings,
                project_path,
            )
            .unwrap();

        // Mock: read generated spin.toml
        fixture
            .file_system
            .expect_read_to_string()
            .withf(|path: &Path| path.to_string_lossy().ends_with("spin.toml"))
            .times(1)
            .returning(move |_| Ok(expected_spin_content.clone()));
    }
    // No else clause - we always require ftl.toml now
}

#[tokio::test]
async fn test_deploy_no_ftl_toml() {
    let mut fixture = TestFixture::new();

    // Mock: Check if ftl.toml exists (will return false)
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(|_| false);

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

    // This test checks authentication failure with ftl.toml
    let ftl_content = r#"[project]
name = "test-app"
version = "0.1.0"

[component.test-tool]
wasm = "test.wasm"
[component.test-tool.build]
command = "echo 'Building test tool'"
"#;
    setup_comprehensive_ftl_mocks(&mut fixture, ftl_content);

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

    // Mock: update auth config
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(|_, _| Ok(crate::test_helpers::test_auth_config_response()));

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

    // Mock: package.json doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("api/package.json")))
        .returning(|_| false);

    // Mock: pyproject.toml doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("api/pyproject.toml")))
        .returning(|_| false);

    // Mock: go.mod doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("api/go.mod")))
        .returning(|_| false);

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

    // Mock: pyproject.toml doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("worker/pyproject.toml")))
        .returning(|_| false);

    // Mock: go.mod doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("worker/go.mod")))
        .returning(|_| false);

    let fs: Arc<dyn FileSystem> = Arc::new(fs);
    let version = extract_component_version(&fs, "worker", "worker/worker.wasm").unwrap();
    assert_eq!(version, "2.0.0");
}

#[tokio::test]
async fn test_parse_pyproject_toml_version() {
    let mut fs = MockFileSystemMock::new();

    // Mock: Cargo.toml doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("python-tool/Cargo.toml")))
        .returning(|_| false);

    // Mock: package.json doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("python-tool/package.json")))
        .returning(|_| false);

    // Mock: pyproject.toml exists
    fs.expect_exists()
        .with(eq(Path::new("python-tool/pyproject.toml")))
        .returning(|_| true);
    fs.expect_read_to_string()
        .with(eq(Path::new("python-tool/pyproject.toml")))
        .returning(|_| {
            Ok(r#"
[project]
name = "python-tool"
version = "3.0.0"
"#
            .to_string())
        });

    // Mock: go.mod doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("python-tool/go.mod")))
        .returning(|_| false);

    let fs: Arc<dyn FileSystem> = Arc::new(fs);
    let version = extract_component_version(&fs, "python-tool", "python-tool/app.wasm").unwrap();
    assert_eq!(version, "3.0.0");
}

#[tokio::test]
async fn test_parse_go_mod_version() {
    let mut fs = MockFileSystemMock::new();

    // Mock: Cargo.toml doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("go-tool/Cargo.toml")))
        .returning(|_| false);

    // Mock: package.json doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("go-tool/package.json")))
        .returning(|_| false);

    // Mock: pyproject.toml doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("go-tool/pyproject.toml")))
        .returning(|_| false);

    // Mock: go.mod exists
    fs.expect_exists()
        .with(eq(Path::new("go-tool/go.mod")))
        .returning(|_| true);
    fs.expect_read_to_string()
        .with(eq(Path::new("go-tool/go.mod")))
        .returning(|_| {
            Ok(r"module github.com/example/go-tool

go 1.21

// Version: v4.0.0
"
            .to_string())
        });

    let fs: Arc<dyn FileSystem> = Arc::new(fs);
    let version = extract_component_version(&fs, "go-tool", "go-tool/main.wasm").unwrap();
    assert_eq!(version, "4.0.0");
}

#[tokio::test]
async fn test_deployment_timeout() {
    let mut fixture = TestFixture::new();

    // Setup all mocks for successful push
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);
    setup_auth_config_update(&mut fixture);

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
                app_id: uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: auth config update
    setup_auth_config_update(&mut fixture);

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

    // Mock: check for ftl.toml for variables and auth config
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(0..=2)  // May be called for variables and/or resolve_auth_config
        .returning(|_| true);

    // Mock: read ftl.toml for variables extraction and auth config
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(0..=3)  // Once for general variables, once for auth variables, once for resolve_auth_config
        .returning(|_| {
            Ok(r#"[project]
name = "test-project"
version = "0.1.0"

[component.test-tool]
path = "test"
wasm = "test/test-tool.wasm"

[component.test-tool.build]
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
        .withf(|path: &Path| {
            path.ends_with("Cargo.toml")
                || path.ends_with("package.json")
                || path.ends_with("pyproject.toml")
                || path.ends_with("go.mod")
        })
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

fn setup_auth_config_update(fixture: &mut TestFixture) {
    // Mock: update auth config
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(|_, _| Ok(crate::test_helpers::test_auth_config_response()));
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
    let mut fs = MockFileSystemMock::new();

    // Mock: Cargo.toml doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("Cargo.toml")))
        .returning(|_| false);

    // Mock: package.json doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("package.json")))
        .returning(|_| false);

    // Mock: pyproject.toml doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("pyproject.toml")))
        .returning(|_| false);

    // Mock: go.mod doesn't exist
    fs.expect_exists()
        .with(eq(Path::new("go.mod")))
        .returning(|_| false);

    let fs: Arc<dyn FileSystem> = Arc::new(fs);
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
                app_id: uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: auth config update
    setup_auth_config_update(&mut fixture);

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

    assert!(result.is_ok(), "Error: {result:?}");
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
access_control = "private"

[oauth]
issuer = "https://test.authkit.app"
audience = "my-api"

[component.api]
path = "api"
wasm = "api/target/wasm32-wasip1/release/api.wasm"
allowed_outbound_hosts = ["https://*.amazonaws.com"]

[component.api.build]
command = "cargo build --release --target wasm32-wasip1"
"#
    .to_string();

    // Mock: read ftl.toml for FtlConfig parsing
    let ftl_content_clone = ftl_content.clone();
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(move |_| Ok(ftl_content_clone.clone()));

    // Mock: resolve_auth_config checks for ftl.toml
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: resolve_auth_config reads ftl.toml
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
        .withf(|path: &Path| {
            path.ends_with("Cargo.toml")
                || path.ends_with("package.json")
                || path.ends_with("pyproject.toml")
                || path.ends_with("go.mod")
        })
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

    // Generate expected spin.toml content and mock its reading
    let ftl_config = crate::config::ftl_config::FtlConfig::parse(&ftl_content).unwrap();
    let resolved_mappings = std::collections::HashMap::new();
    let project_path = std::path::Path::new(".");
    let expected_spin_content = crate::config::transpiler::create_spin_toml_with_resolved_paths(
        &ftl_config,
        &resolved_mappings,
        project_path,
    )
    .unwrap();

    // Mock: read generated spin.toml for parse_deploy_config
    fixture
        .file_system
        .expect_read_to_string()
        .withf(|path: &Path| path.to_string_lossy().ends_with("spin.toml"))
        .times(1)
        .returning(move |_| Ok(expected_spin_content.clone()));

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

    // Mock: update components
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

    // Mock: wkg push succeeds
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

    // Mock: update auth config based on ftl.toml (private mode)
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(|_, _| Ok(crate::test_helpers::test_auth_config_response()));

    // Mock: create deployment with auth variables
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(|_, req| {
            // Verify auth variables are passed correctly
            assert!(req.variables.contains_key("auth_enabled"));
            assert_eq!(req.variables.get("auth_enabled"), Some(&"true".to_string()));

            assert!(req.variables.contains_key("mcp_provider_type"));
            assert_eq!(
                req.variables.get("mcp_provider_type"),
                Some(&"jwt".to_string())
            );

            assert!(req.variables.contains_key("mcp_jwt_issuer"));
            assert_eq!(
                req.variables.get("mcp_jwt_issuer"),
                Some(&"https://test.authkit.app".to_string())
            );

            assert!(req.variables.contains_key("mcp_jwt_audience"));
            assert_eq!(
                req.variables.get("mcp_jwt_audience"),
                Some(&"my-api".to_string())
            );

            // Check that OAuth variables are not set (since we're using authkit)
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

    assert!(result.is_ok(), "Error: {result:?}");
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
access_control = "private"

[oauth]
issuer = "https://test.authkit.app"
audience = "my-api"

[component.api]
path = "api"
wasm = "api/target/wasm32-wasip1/release/api.wasm"
allowed_outbound_hosts = ["https://*.amazonaws.com"]

[component.api.build]
command = "cargo build --release --target wasm32-wasip1"
"#
    .to_string();

    // Mock: read ftl.toml for FtlConfig parsing
    let ftl_content_clone = ftl_content.clone();
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(move |_| Ok(ftl_content_clone.clone()));

    // Mock: resolve_auth_config checks for ftl.toml
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: resolve_auth_config reads ftl.toml
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
        .withf(|path: &Path| {
            path.ends_with("Cargo.toml")
                || path.ends_with("package.json")
                || path.ends_with("pyproject.toml")
                || path.ends_with("go.mod")
        })
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

    // Generate expected spin.toml content and mock its reading
    let ftl_config = crate::config::ftl_config::FtlConfig::parse(&ftl_content).unwrap();
    let resolved_mappings = std::collections::HashMap::new();
    let project_path = std::path::Path::new(".");
    let expected_spin_content = crate::config::transpiler::create_spin_toml_with_resolved_paths(
        &ftl_config,
        &resolved_mappings,
        project_path,
    )
    .unwrap();

    // Mock: read generated spin.toml for parse_deploy_config
    fixture
        .file_system
        .expect_read_to_string()
        .withf(|path: &Path| path.to_string_lossy().ends_with("spin.toml"))
        .times(1)
        .returning(move |_| Ok(expected_spin_content.clone()));

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

    // Mock: update components
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

    // Mock: wkg push succeeds
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

    // Mock: update auth config based on ftl.toml (private mode)
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(|_, _| Ok(crate::test_helpers::test_auth_config_response()));

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

            assert!(req.variables.contains_key("mcp_jwt_issuer"));
            assert_eq!(
                req.variables.get("mcp_jwt_issuer"),
                Some(&"https://override.authkit.app".to_string())
            ); // CLI override

            // ftl.toml values should still be present for non-overridden variables
            assert!(req.variables.contains_key("mcp_provider_type"));
            assert_eq!(
                req.variables.get("mcp_provider_type"),
                Some(&"jwt".to_string())
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
            "mcp_jwt_issuer=https://override.authkit.app".to_string(),
        ]),
    )
    .await;

    assert!(result.is_ok(), "Error: {result:?}");
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Deployed!")));
}

// Helper function to create default DeployArgs for tests
fn default_deploy_args() -> DeployArgs {
    DeployArgs {
        variables: vec![],
        access_control: None,
        jwt_issuer: None,
        dry_run: false,
        yes: true, // Skip confirmation in tests
    }
}

// Helper function to create DeployArgs with variables
fn deploy_args_with_variables(variables: Vec<String>) -> DeployArgs {
    DeployArgs {
        variables,
        access_control: None,
        jwt_issuer: None,
        dry_run: false,
        yes: true, // Skip confirmation in tests
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
            match request.access_control {
                types::UpdateAuthConfigRequestAccessControl::Public => {}
                _ => panic!("Expected public access control"),
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
        access_control: Some("public".to_string()),
        jwt_issuer: None,
        dry_run: false,
        yes: true,
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

#[test]
#[allow(clippy::cognitive_complexity)]
fn test_is_sensitive_variable() {
    // Import the function from the parent module
    use super::is_sensitive_variable;

    // Test sensitive patterns
    assert!(is_sensitive_variable("api_token"));
    assert!(is_sensitive_variable("API_TOKEN"));
    assert!(is_sensitive_variable("secret_key"));
    assert!(is_sensitive_variable("password"));
    assert!(is_sensitive_variable("my_password"));
    assert!(is_sensitive_variable("pwd"));
    assert!(is_sensitive_variable("auth_key"));
    assert!(is_sensitive_variable("credential"));
    assert!(is_sensitive_variable("api_key"));
    assert!(is_sensitive_variable("apikey"));
    assert!(is_sensitive_variable("private_key"));
    assert!(is_sensitive_variable("priv_data"));
    assert!(is_sensitive_variable("certificate"));
    assert!(is_sensitive_variable("cert_data"));
    assert!(is_sensitive_variable("signing_key"));
    assert!(is_sensitive_variable("jwt_secret"));
    assert!(is_sensitive_variable("bearer_token"));
    assert!(is_sensitive_variable("oauth_secret"));
    assert!(is_sensitive_variable("access_token"));
    assert!(is_sensitive_variable("refresh_token"));
    assert!(is_sensitive_variable("GITHUB_ACCESS_TOKEN"));
    assert!(is_sensitive_variable("aws_secret_access_key"));

    // Test non-sensitive patterns
    assert!(!is_sensitive_variable("api_url"));
    assert!(!is_sensitive_variable("environment"));
    assert!(!is_sensitive_variable("debug_mode"));
    assert!(!is_sensitive_variable("port"));
    assert!(!is_sensitive_variable("host"));
    assert!(!is_sensitive_variable("timeout"));
    assert!(!is_sensitive_variable("max_retries"));
    assert!(!is_sensitive_variable("api_version"));

    // Test auth configuration exceptions (not sensitive)
    assert!(!is_sensitive_variable("auth_enabled"));
    assert!(!is_sensitive_variable("AUTH_ENABLED"));
    assert!(!is_sensitive_variable("mcp_jwt_issuer"));
    assert!(!is_sensitive_variable("mcp_jwt_audience"));
    assert!(!is_sensitive_variable("mcp_provider_type"));
    assert!(!is_sensitive_variable("mcp_jwt_jwks_uri"));
}

#[tokio::test]
async fn test_deploy_with_sensitive_variables() {
    let mut fixture = TestFixture::new();
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);
    setup_successful_deployment(&mut fixture);

    // Mock: auth config update
    setup_auth_config_update(&mut fixture);

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Deploy with various sensitive and non-sensitive variables
    let args = DeployArgs {
        variables: vec![
            "api_token=super-secret-token".to_string(),
            "api_url=https://api.example.com".to_string(),
            "database_password=db-pass-123".to_string(),
            "environment=production".to_string(),
            "jwt_secret=jwt-secret-value".to_string(),
            "debug_mode=false".to_string(),
        ],
        access_control: None,
        jwt_issuer: None,
        dry_run: false,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_ok());

    let output = ui.get_output();

    // Check that sensitive variables are redacted in output
    assert!(!output.iter().any(|s| s.contains("super-secret-token")));
    assert!(!output.iter().any(|s| s.contains("db-pass-123")));
    assert!(!output.iter().any(|s| s.contains("jwt-secret-value")));

    // Check that partial values might be shown (first 2 chars)
    assert!(output.iter().any(|s| s.contains("su***"))); // api_token
    assert!(output.iter().any(|s| s.contains("db***"))); // database_password
    assert!(output.iter().any(|s| s.contains("jw***"))); // jwt_secret

    // Check that non-sensitive variables are shown in full
    assert!(output.iter().any(|s| s.contains("https://api.example.com")));
    assert!(output.iter().any(|s| s.contains("production")));
    assert!(output.iter().any(|s| s.contains("false")));

    // Check that lock icons are shown for sensitive vars
    assert!(output.iter().any(|s| s.contains("🔒 api_token")));
    assert!(output.iter().any(|s| s.contains("🔒 database_password")));
    assert!(output.iter().any(|s| s.contains("🔒 jwt_secret")));

    // Check that non-sensitive vars don't have lock icons
    assert!(output.iter().any(|s| s.contains("   api_url")));
    assert!(output.iter().any(|s| s.contains("   environment")));
    assert!(output.iter().any(|s| s.contains("   debug_mode")));
}

#[tokio::test]
async fn test_deploy_with_short_sensitive_values() {
    let mut fixture = TestFixture::new();
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);
    setup_successful_deployment(&mut fixture);

    // Mock: auth config update
    setup_auth_config_update(&mut fixture);

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Deploy with short sensitive values (4 chars or less)
    let args = DeployArgs {
        variables: vec![
            "key=abc".to_string(),
            "token=xyz".to_string(),
            "secret=1234".to_string(),
        ],
        access_control: None,
        jwt_issuer: None,
        dry_run: false,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_ok());

    let output = ui.get_output();

    // Check that short sensitive values are fully redacted
    assert!(!output.iter().any(|s| s.contains("abc")));
    assert!(!output.iter().any(|s| s.contains("xyz")));
    assert!(!output.iter().any(|s| s.contains("1234")));

    // Should show *** for short values
    assert!(output.iter().any(|s| s.contains("key = ***")));
    assert!(output.iter().any(|s| s.contains("token = ***")));
    assert!(output.iter().any(|s| s.contains("secret = ***")));
}

#[tokio::test]
async fn test_deploy_dry_run() {
    let mut fixture = TestFixture::new();

    // Setup project file mocks
    setup_project_file_mocks(&mut fixture, true);

    // Mock: resolve_auth_config checks for ftl.toml
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: resolve_auth_config reads ftl.toml
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| {
            Ok(r#"[project]
name = "test-project"
version = "0.1.0"

[component.test-tool]
path = "test"
wasm = "test/test-tool.wasm"

[component.test-tool.build]
command = "echo 'Building test tool'"
"#
            .to_string())
        });

    // Mock: component version files don't exist (use default)
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| {
            path.ends_with("Cargo.toml")
                || path.ends_with("package.json")
                || path.ends_with("pyproject.toml")
                || path.ends_with("go.mod")
        })
        .returning(|_| false);

    // Mock: clock for progress bar
    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    // Build executor succeeds

    // Mock: credentials succeed (for authentication check)
    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .times(1)
        .returning(|| Ok(test_credentials()));

    // No API calls should be made in dry-run mode
    // No ECR token creation, no docker login, no app creation, etc.

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Deploy with dry-run flag
    let args = DeployArgs {
        variables: vec![
            "api_token=test-token-123".to_string(),
            "api_url=https://api.example.com".to_string(),
            "debug_mode=true".to_string(),
        ],
        access_control: Some("public".to_string()),
        jwt_issuer: None,
        dry_run: true,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_ok());

    let output = ui.get_output();

    // Check that dry-run mode is indicated
    assert!(output.iter().any(|s| s.contains("DRY RUN MODE")));
    assert!(output.iter().any(|s| s.contains("No changes will be made")));

    // Check engine configuration
    assert!(output.iter().any(|s| s.contains("Engine Configuration:")));
    assert!(output.iter().any(|s| s.contains("Name: test-project")));
    assert!(output.iter().any(|s| s.contains("Build Profile: release")));

    // Check components section
    assert!(output.iter().any(|s| s.contains("Components to Deploy:")));
    assert!(output.iter().any(|s| s.contains("test-tool")));

    // Check variables section with proper redaction
    assert!(output.iter().any(|s| s.contains("Variables (4):")));
    assert!(output.iter().any(|s| s.contains("🔒 api_token = te***")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("   api_url = https://api.example.com"))
    );
    assert!(output.iter().any(|s| s.contains("   debug_mode = true")));

    // Check auth configuration
    assert!(
        output
            .iter()
            .any(|s| s.contains("Authorization Configuration:"))
    );
    assert!(output.iter().any(|s| s.contains("Mode: public")));

    // Check completion message
    assert!(output.iter().any(|s| s.contains("Dry run complete")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("run the command without --dry-run"))
    );

    // Make sure no deployment success message appears
    assert!(!output.iter().any(|s| s.contains("Deployed!")));
    assert!(!output.iter().any(|s| s.contains("MCP URL:")));
}

#[tokio::test]
async fn test_deploy_dry_run_no_variables() {
    let mut fixture = TestFixture::new();

    // Setup project file mocks
    setup_project_file_mocks(&mut fixture, true);

    // Mock: check for ftl.toml when resolving auth config
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)  // Only for resolve_auth_config now
        .returning(|_| true);

    // Mock: read ftl.toml for auth config resolution
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(1)  // Only for resolve_auth_config now (variables use parsed config)
        .returning(|_| {
            Ok(r#"[project]
name = "test-project"
version = "0.1.0"

[component.test-tool]
path = "test"
wasm = "test/test-tool.wasm"

[component.test-tool.build]
command = "echo 'Building test tool'"
"#
            .to_string())
        });

    // Mock: component version files don't exist (use default)
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| {
            path.ends_with("Cargo.toml")
                || path.ends_with("package.json")
                || path.ends_with("pyproject.toml")
                || path.ends_with("go.mod")
        })
        .returning(|_| false);

    // Mock: clock for progress bar
    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    // Mock: credentials succeed
    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .times(1)
        .returning(|| Ok(test_credentials()));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Deploy with dry-run flag but no variables
    let args = DeployArgs {
        variables: vec![],
        access_control: None,
        jwt_issuer: None,
        dry_run: true,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_ok());

    let output = ui.get_output();

    // Should show variables section with auth_enabled (not redacted)
    assert!(output.iter().any(|s| s.contains("Variables (1):")));
    assert!(output.iter().any(|s| s.contains("   auth_enabled = false")));

    // Should show auth section with public mode
    assert!(
        output
            .iter()
            .any(|s| s.contains("Authorization Configuration:"))
    );
    assert!(output.iter().any(|s| s.contains("Mode: public")));
}

#[tokio::test]
async fn test_deploy_auth_mode_user_only() {
    let mut fixture = TestFixture::new();

    // Setup all basic mocks
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);
    setup_successful_deployment(&mut fixture);

    // Mock: update auth config is called
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(|_, _| Ok(crate::test_helpers::test_auth_config_response()));

    // Mock: create app returns specific ID
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

    let deps = fixture.to_deps();
    let args = DeployArgs {
        variables: vec![],
        access_control: Some("private".to_string()),
        jwt_issuer: None,
        dry_run: false,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_deploy_auth_mode_custom() {
    let mut fixture = TestFixture::new();

    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);
    setup_successful_deployment(&mut fixture);

    // Mock: update auth config with custom configuration
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(|_, _| Ok(crate::test_helpers::test_auth_config_response()));

    // Mock: create app
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

    let deps = fixture.to_deps();
    let args = DeployArgs {
        variables: vec![],
        access_control: Some("private".to_string()),
        jwt_issuer: Some("https://auth.example.com".to_string()),
        dry_run: false,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_ok());
}

// Test removed: private mode without issuer is now valid (uses FTL's AuthKit)

#[tokio::test]
async fn test_deploy_invalid_auth_mode() {
    let mut fixture = TestFixture::new();

    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);

    // Mock: list apps
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
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    let deps = fixture.to_deps();
    let args = DeployArgs {
        variables: vec![],
        access_control: Some("invalid-mode".to_string()),
        jwt_issuer: None,
        dry_run: false,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid access control mode")
    );
}

#[tokio::test]
async fn test_deploy_with_deploy_name_override() {
    let mut fixture = TestFixture::new();

    let ftl_toml_content = r#"[project]
name = "test-app"
version = "0.1.0"

[component.my-component]
path = "my-component"
wasm = "my-component/target/wasm32-wasip1/release/my_component.wasm"

[component.my-component.deploy]
name = "custom-deployed-name"
profile = "release"

[component.my-component.build]
command = "cargo build --release"
"#;

    // Use comprehensive mock setup
    setup_comprehensive_ftl_mocks(&mut fixture, ftl_toml_content);

    // Setup remaining mocks
    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    fixture
        .clock
        .expect_duration_from_secs()
        .returning(Duration::from_secs);

    fixture.clock.expect_now().returning(Instant::now);

    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .returning(|| Ok(test_credentials()));

    setup_docker_login_success(&mut fixture);

    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("wkg"))
        .times(1)
        .returning(|_| Ok(()));

    // Mock: update components should receive the custom deploy name
    fixture
        .api_client
        .expect_update_components()
        .times(1)
        .returning(|_, _| {
            Ok(types::UpdateComponentsResponse {
                components: vec![types::UpdateComponentsResponseComponentsItem {
                    component_name: "custom-deployed-name".to_string(),
                    description: None,
                    repository_uri: Some(
                        "123456789012.dkr.ecr.us-east-1.amazonaws.com/user/custom-deployed-name"
                            .to_string(),
                    ),
                    repository_name: Some("user/custom-deployed-name".to_string()),
                }],
                changes: types::UpdateComponentsResponseChanges {
                    created: vec!["custom-deployed-name".to_string()],
                    updated: vec![],
                    removed: vec![],
                },
            })
        });

    // Rest of the deployment mocks
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

    setup_successful_deployment(&mut fixture);

    // Mock: update auth config
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(|_, _| Ok(crate::test_helpers::test_auth_config_response()));

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_deploy_build_profile_debug() {
    let mut fixture = TestFixture::new();

    let ftl_toml_content = r#"[project]
name = "test-app"
version = "0.1.0"

[component.debug-component]
path = "debug-component"
wasm = "debug-component/target/wasm32-wasip1/debug/debug_component.wasm"

[component.debug-component.deploy]
profile = "debug"

[component.debug-component.build]
command = "cargo build"
"#;

    // Use comprehensive mock setup
    setup_comprehensive_ftl_mocks(&mut fixture, ftl_toml_content);

    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .returning(|| Ok(test_credentials()));

    // Mock build executor to verify debug mode is used
    let build_executor = Arc::new(MockBuildExecutor::new());
    let build_executor_clone = build_executor.clone();

    let deps = Arc::new(DeployDependencies {
        file_system: Arc::new(fixture.file_system),
        command_executor: Arc::new(fixture.command_executor),
        api_client: Arc::new(fixture.api_client),
        clock: Arc::new(fixture.clock),
        credentials_provider: Arc::new(fixture.credentials_provider),
        ui: fixture.ui,
        build_executor: build_executor_clone,
        async_runtime: Arc::new(fixture.async_runtime),
    });

    // Dry run to test profile detection without full deployment
    let args = DeployArgs {
        variables: vec![],
        access_control: None,
        jwt_issuer: None,
        dry_run: true,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_ok());

    // In debug mode, use_release should be false
    // This would be better tested with a custom build executor that tracks the release flag
}

#[tokio::test]
async fn test_deploy_required_variables_from_ftl() {
    let mut fixture = TestFixture::new();

    let ftl_toml_content = r#"[project]
name = "test-app"
version = "0.1.0"

[variables]
api_key = { required = true }
database_url = { required = true }
optional_var = { default = "default-value" }

[component.test-tool]
path = "test"
wasm = "test/test-tool.wasm"

[component.test-tool.build]
command = "echo 'Building test tool'"
"#;

    // Use comprehensive mock setup
    setup_comprehensive_ftl_mocks(&mut fixture, ftl_toml_content);

    fixture
        .clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);

    fixture
        .credentials_provider
        .expect_get_or_refresh_credentials()
        .returning(|| Ok(test_credentials()));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Dry run to check variable handling
    let args = DeployArgs {
        variables: vec!["api_key=provided-key".to_string()], // Only provide one required var
        access_control: None,
        jwt_issuer: None,
        dry_run: true,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    assert!(result.is_ok());

    let output = ui.get_output();
    // Should have the provided required var and the default
    assert!(output.iter().any(|s| s.contains("api_key = pr***")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("optional_var = default-value"))
    );
    // Should NOT have database_url since it's required but not provided
    assert!(!output.iter().any(|s| s.contains("database_url")));
}

#[tokio::test]
async fn test_parse_variables_edge_cases() {
    // Test empty value
    let vars = vec!["KEY=".to_string()];
    let parsed = parse_variables(&vars).unwrap();
    assert_eq!(parsed.get("KEY"), Some(&String::new()));

    // Test value with multiple equals signs
    let vars = vec!["URL=https://example.com?key=value&other=123".to_string()];
    let parsed = parse_variables(&vars).unwrap();
    assert_eq!(
        parsed.get("URL"),
        Some(&"https://example.com?key=value&other=123".to_string())
    );

    // Test whitespace handling
    let vars = vec![" KEY = value ".to_string()];
    let parsed = parse_variables(&vars).unwrap();
    assert_eq!(parsed.get("KEY"), Some(&"value".to_string()));

    // Test special characters in value
    let vars = vec!["KEY=!@#$%^&*()[]{}".to_string()];
    let parsed = parse_variables(&vars).unwrap();
    assert_eq!(parsed.get("KEY"), Some(&"!@#$%^&*()[]{}".to_string()));
}

#[tokio::test]
async fn test_deploy_partial_component_push_failure() {
    let mut fixture = TestFixture::new();

    setup_full_mocks(&mut fixture);

    // Mock: list apps
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
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: update components succeeds
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

    // Mock: wkg push fails
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "wkg" && args.contains(&"push"))
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: false,
                stdout: vec![],
                stderr: b"Failed to push component: Network error".to_vec(),
            })
        });

    let deps = fixture.to_deps();
    let result = execute_with_deps(deps, default_deploy_args()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to push"));
}

#[tokio::test]
async fn test_deploy_auth_enabled_always_included() {
    let mut fixture = TestFixture::new();

    // Setup basic mocks
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);

    // Mock: list apps returns empty
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
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: types::CreateAppResponseStatus::Creating,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            })
        });

    // Mock: update auth config - should be called even when auth is disabled
    fixture
        .api_client
        .expect_update_auth_config()
        .times(1)
        .returning(|_, _| Ok(crate::test_helpers::test_auth_config_response()));

    // Mock: create deployment - verify auth_enabled is always present
    fixture
        .api_client
        .expect_create_deployment()
        .times(1)
        .returning(|_, req| {
            // This is the key assertion - auth_enabled should always be present
            assert!(
                req.variables.contains_key("auth_enabled"),
                "auth_enabled must always be included in deployment variables"
            );
            assert_eq!(
                req.variables.get("auth_enabled"),
                Some(&"false".to_string()),
                "auth_enabled should be 'false' for public access control"
            );

            Ok(types::CreateDeploymentResponse {
                deployment_id: uuid::Uuid::new_v4(),
                app_id: uuid::Uuid::new_v4(),
                app_name: "test-app".to_string(),
                status: "DEPLOYING".to_string(),
                message: "Deployment started".to_string(),
            })
        });

    // Mock: update components
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

    let deps = fixture.to_deps();
    let args = DeployArgs {
        variables: vec![],
        access_control: None, // Public access control
        jwt_issuer: None,
        dry_run: false,
        yes: true,
    };

    let result = execute_with_deps(deps, args).await;
    if let Err(e) = &result {
        eprintln!("Test failed with error: {e}");
    }
    assert!(result.is_ok());
}

#[test]
fn test_add_auth_variables_from_config() {
    use crate::config::ftl_config::FtlConfig;

    // Test 1: Public access control (auth disabled)
    let ftl_config_str = r#"[project]
name = "test-app"
version = "0.1.0"
access_control = "public"
"#;
    let config = FtlConfig::parse(ftl_config_str).unwrap();

    let mut variables = HashMap::new();
    add_auth_variables_from_config(&config, &mut variables);

    // auth_enabled should be present and set to "false"
    assert_eq!(variables.get("auth_enabled"), Some(&"false".to_string()));
    // When auth is disabled, other auth variables should NOT be present
    assert_eq!(variables.get("mcp_provider_type"), None);
    assert_eq!(variables.get("mcp_jwt_issuer"), None);

    // Test 2: Private access control (auth enabled)
    let ftl_config_str2 = r#"[project]
name = "test-app"
version = "0.1.0"
access_control = "private"
"#;
    let config2 = FtlConfig::parse(ftl_config_str2).unwrap();

    let mut variables = HashMap::new();
    add_auth_variables_from_config(&config2, &mut variables);

    // auth_enabled should be present and set to "true"
    assert_eq!(variables.get("auth_enabled"), Some(&"true".to_string()));
    // Provider type should be "jwt"
    assert_eq!(variables.get("mcp_provider_type"), Some(&"jwt".to_string()));
    // Should use FTL's built-in AuthKit issuer
    assert_eq!(
        variables.get("mcp_jwt_issuer"),
        Some(&"https://divine-lion-50-staging.authkit.app".to_string())
    );

    // Test 3: With the new design, variables are always overwritten since
    // precedence is handled at a higher level
    let mut variables = HashMap::new();
    variables.insert("auth_enabled".to_string(), "custom_value".to_string());
    variables.insert(
        "mcp_jwt_issuer".to_string(),
        "https://custom.issuer.com".to_string(),
    );

    add_auth_variables_from_config(&config2, &mut variables);

    // With the new design, ftl.toml values always get set (precedence is handled elsewhere)
    assert_eq!(variables.get("auth_enabled"), Some(&"true".to_string()));
    assert_eq!(
        variables.get("mcp_jwt_issuer"),
        Some(&"https://divine-lion-50-staging.authkit.app".to_string())
    );
}

#[test]
fn test_resolve_auth_config_public_access() {
    use crate::commands::deploy::DeployArgs;
    use crate::commands::deploy::resolve_auth_config;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Test 1: Public access control in ftl.toml should be resolved
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"[project]
name = "test-app"
version = "0.1.0"
access_control = "public"
"#
    )
    .unwrap();

    let content = std::fs::read_to_string(file.path()).unwrap();
    let mut mock_fs = MockFileSystemMock::new();
    mock_fs
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| true);
    mock_fs
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(move |_| Ok(content.clone()));
    let fs_arc: Arc<dyn FileSystem> = Arc::new(mock_fs);

    let args = DeployArgs {
        variables: vec![],
        access_control: None,
        jwt_issuer: None,
        dry_run: false,
        yes: true,
    };

    let result = resolve_auth_config(&fs_arc, &args).unwrap();

    // Should resolve to public mode
    assert!(result.is_some());
    let (mode, provider, issuer, audience) = result.unwrap();
    assert_eq!(mode, "public");
    assert!(provider.is_none());
    assert!(issuer.is_none());
    assert!(audience.is_none());

    // Test 2: Private access control should include auth details
    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(
        file2,
        r#"[project]
name = "test-app"
version = "0.1.0"
access_control = "private"
"#
    )
    .unwrap();

    let content2 = std::fs::read_to_string(file2.path()).unwrap();
    let mut mock_fs2 = MockFileSystemMock::new();
    mock_fs2
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| true);
    mock_fs2
        .expect_read_to_string()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(move |_| Ok(content2.clone()));
    let fs_arc2: Arc<dyn FileSystem> = Arc::new(mock_fs2);

    let result2 = resolve_auth_config(&fs_arc2, &args).unwrap();

    // Should resolve to private mode with FTL AuthKit details
    assert!(result2.is_some());
    let (mode, provider, issuer, audience) = result2.unwrap();
    assert_eq!(mode, "private");
    assert_eq!(provider, Some("jwt".to_string()));
    assert_eq!(
        issuer,
        Some("https://divine-lion-50-staging.authkit.app".to_string())
    );
    assert!(audience.is_none());
}
