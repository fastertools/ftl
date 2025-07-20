//! Comprehensive unit tests for deploy_v2 module

use crate::commands::deploy::*;
use crate::test_helpers::*;
use crate::deps::*;
use crate::api_client::types;
use std::sync::Arc;
use std::path::Path;
use std::time::{Duration, Instant};
use mockall::predicate::*;

#[tokio::test]
async fn test_deploy_no_spin_toml() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml doesn't exist
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| false);
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No spin.toml found"));
}

#[tokio::test]
async fn test_deploy_authentication_expired() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: clock for progress bar
    fixture.clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);
    
    // Mock: build executor succeeds
    fixture.build_executor
        .expect_execute()
        .times(1)
        .returning(|_, _| Ok(()));
    
    // Mock: credentials expired
    fixture.credentials_provider
        .expect_get_or_refresh_credentials()
        .times(1)
        .returning(|| Err(anyhow::anyhow!("Token expired")));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Authentication token has expired"));
}

#[tokio::test]
async fn test_deploy_no_components() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: read spin.toml with no components
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
[application]
name = "test-app"
"#.to_string()));
    
    // Mock: clock for progress bar
    fixture.clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);
    
    // Mock: build executor succeeds
    fixture.build_executor
        .expect_execute()
        .times(1)
        .returning(|_, _| Ok(()));
    
    // Mock: credentials succeed
    fixture.credentials_provider
        .expect_get_or_refresh_credentials()
        .times(1)
        .returning(|| Ok(test_credentials()));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No user components found"));
}

#[tokio::test]
async fn test_deploy_docker_login_failure() {
    let mut fixture = TestFixture::new();
    
    // Setup basic mocks
    setup_basic_mocks(&mut fixture);
    
    // Mock: docker login fails
    fixture.command_executor
        .expect_execute_with_stdin()
        .withf(|cmd: &str, args: &[&str], _: &str| cmd == "docker" && args.contains(&"login"))
        .times(1)
        .returning(|_, _, _| Ok(CommandOutput {
            success: false,
            stdout: vec![],
            stderr: b"Login failed".to_vec(),
        }));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Docker login failed"));
}

#[tokio::test]
async fn test_deploy_wkg_not_found() {
    let mut fixture = TestFixture::new();
    
    // Setup basic mocks including successful docker login
    setup_basic_mocks(&mut fixture);
    setup_docker_login_success(&mut fixture);
    
    // Mock: wkg not found
    fixture.command_executor
        .expect_check_command_exists()
        .with(eq("wkg"))
        .times(1)
        .returning(|_| Err(anyhow::anyhow!("wkg not found")));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("wkg not found"));
}

#[tokio::test]
async fn test_deploy_repository_creation_failure() {
    let mut fixture = TestFixture::new();
    
    // Setup all basic mocks
    setup_full_mocks(&mut fixture);
    
    // Mock: repository creation fails
    fixture.api_client
        .expect_create_ecr_repository()
        .times(1)
        .returning(|_| Err(anyhow::anyhow!("Repository creation failed")));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to create repository"));
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
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_ok());
    
    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Deployment successful!")));
    assert!(output.iter().any(|s| s.contains("https://test-app.example.com")));
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
        .returning(|_| Ok(r#"
[package]
name = "api"
version = "1.2.3"
"#.to_string()));
    
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
    
    // Mock: deploy app succeeds
    fixture.api_client
        .expect_deploy_app()
        .times(1)
        .returning(|_| Ok(test_deployment_response("test-deployment-id")));
    
    // Mock: status always returns "deploying" (60 times = timeout)
    fixture.api_client
        .expect_get_deployment_status()
        .times(60)
        .returning(|_| Ok(test_deployment_status(
            "test-deployment-id",
            types::DeploymentStatusDeploymentStatus::Deploying,
        )));
    
    // Mock: async sleep
    fixture.async_runtime
        .expect_sleep()
        .times(60)
        .returning(|_| ());
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Deployment timeout"));
}

#[tokio::test]
async fn test_deployment_failed_status() {
    let mut fixture = TestFixture::new();
    
    // Setup all mocks for successful push
    setup_full_mocks(&mut fixture);
    setup_successful_push(&mut fixture);
    
    // Mock: deploy app succeeds
    fixture.api_client
        .expect_deploy_app()
        .times(1)
        .returning(|_| Ok(test_deployment_response("test-deployment-id")));
    
    // Mock: status returns failed
    fixture.api_client
        .expect_get_deployment_status()
        .times(1)
        .returning(|_| {
            let status = test_deployment_status(
                "test-deployment-id",
                types::DeploymentStatusDeploymentStatus::Failed,
            );
            Ok(types::DeploymentStatus {
                deployment: types::DeploymentStatusDeployment {
                    error: Some("Build failed".to_string()),
                    ..status.deployment
                },
            })
        });
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(deps).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Build failed"));
}

// Helper functions to setup common mock scenarios

fn setup_basic_mocks(fixture: &mut TestFixture) {
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: read spin.toml with components
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
[application]
name = "test-app"

[component.api]
source = "target/wasm32-wasi/release/api.wasm"
allowed_outbound_hosts = ["https://*.amazonaws.com"]
"#.to_string()));
    
    // Mock: component version files don't exist (use default)
    fixture.file_system
        .expect_exists()
        .returning(|_| false);
    
    // Mock: clock for progress bars
    fixture.clock
        .expect_duration_from_millis()
        .returning(Duration::from_millis);
    
    fixture.clock
        .expect_duration_from_secs()
        .returning(Duration::from_secs);
    
    fixture.clock
        .expect_now()
        .returning(Instant::now);
    
    // Mock: build executor succeeds
    fixture.build_executor
        .expect_execute()
        .times(1)
        .returning(|_, _| Ok(()));
    
    // Mock: credentials succeed
    fixture.credentials_provider
        .expect_get_or_refresh_credentials()
        .returning(|| Ok(test_credentials()));
    
    // Mock: API client returns ECR credentials
    fixture.api_client
        .expect_get_ecr_credentials()
        .times(1)
        .returning(|| Ok(test_ecr_credentials()));
}

fn setup_docker_login_success(fixture: &mut TestFixture) {
    fixture.command_executor
        .expect_execute_with_stdin()
        .withf(|cmd: &str, args: &[&str], _: &str| cmd == "docker" && args.contains(&"login"))
        .times(1)
        .returning(|_, _, _| Ok(CommandOutput {
            success: true,
            stdout: b"Login Succeeded".to_vec(),
            stderr: vec![],
        }));
}

fn setup_full_mocks(fixture: &mut TestFixture) {
    setup_basic_mocks(fixture);
    setup_docker_login_success(fixture);
    
    // Mock: wkg exists
    fixture.command_executor
        .expect_check_command_exists()
        .with(eq("wkg"))
        .times(1)
        .returning(|_| Ok(()));
}

fn setup_successful_push(fixture: &mut TestFixture) {
    // Mock: create repository succeeds
    fixture.api_client
        .expect_create_ecr_repository()
        .times(1)
        .returning(|_req| {
            // Extract tool name from request
            let tool_name = "api"; // For simplicity in test
            Ok(test_repository_response(tool_name))
        });
    
    // Mock: wkg push succeeds (both version and latest tags)
    fixture.command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "wkg" && args.contains(&"push"))
        .times(2)
        .returning(|_, _| Ok(CommandOutput {
            success: true,
            stdout: b"Pushed successfully".to_vec(),
            stderr: vec![],
        }));
}

fn setup_successful_deployment(fixture: &mut TestFixture) {
    // Mock: deploy app succeeds
    fixture.api_client
        .expect_deploy_app()
        .times(1)
        .returning(|_| Ok(test_deployment_response("test-deployment-id")));
    
    // Mock: deployment status checks - first returns deploying, then deployed
    let mut call_count = 0;
    fixture.api_client
        .expect_get_deployment_status()
        .times(2)
        .returning(move |_| {
            call_count += 1;
            if call_count == 1 {
                Ok(test_deployment_status(
                    "test-deployment-id",
                    types::DeploymentStatusDeploymentStatus::Deploying,
                ))
            } else {
                Ok(test_deployment_status(
                    "test-deployment-id",
                    types::DeploymentStatusDeploymentStatus::Deployed,
                ))
            }
        });
    
    // Mock: async sleep (called once)
    fixture.async_runtime
        .expect_sleep()
        .times(1)
        .returning(|_| ());
}