//! Unit tests for the build command

use std::path::Path;
use std::sync::Arc;

use mockall::predicate::*;

use crate::commands::build::*;
use crate::deps::*;
use crate::test_helpers::*;
use crate::ui::TestUserInterface;

struct TestFixture {
    file_system: MockFileSystemMock,
    command_executor: MockCommandExecutorMock,
    ui: Arc<TestUserInterface>,
    spin_installer: MockSpinInstallerMock,
    async_runtime: MockAsyncRuntimeMock,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            file_system: MockFileSystemMock::new(),
            command_executor: MockCommandExecutorMock::new(),
            ui: Arc::new(TestUserInterface::new()),
            spin_installer: MockSpinInstallerMock::new(),
            async_runtime: MockAsyncRuntimeMock::new(),
        }
    }

    fn to_deps(self) -> Arc<BuildDependencies> {
        Arc::new(BuildDependencies {
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            ui: self.ui as Arc<dyn UserInterface>,
            spin_installer: Arc::new(self.spin_installer) as Arc<dyn SpinInstaller>,
            async_runtime: Arc::new(self.async_runtime) as Arc<dyn AsyncRuntime>,
        })
    }
}

#[tokio::test]
async fn test_build_no_spin_toml() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml doesn't exist
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| false);
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
        },
        deps,
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No spin.toml found"));
}

#[tokio::test]
async fn test_build_no_components() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: spin.toml with no build components
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"
version = "0.1.0"

[[trigger.http]]
route = "/..."

[component.test]
source = "target/wasm32-wasi/release/test.wasm"
"#.to_string()));
    
    // No need to mock spin installer since we don't have components to build
    
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
        },
        deps,
    ).await;
    
    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("No components with build commands found")));
}

#[tokio::test]
async fn test_build_single_component_success() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: spin.toml with one build component
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"
version = "0.1.0"

[[trigger.http]]
route = "/..."

[component.backend]
source = "target/wasm32-wasi/release/backend.wasm"
[component.backend.build]
command = "cargo build --target wasm32-wasi"
"#.to_string()));
    
    // Mock: spin installer
    fixture.spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));
    
    // Mock: build command execution
    fixture.command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| {
            (cfg!(target_os = "windows") && cmd == "cmd" && args == ["/C", "cargo build --target wasm32-wasi"]) ||
            (!cfg!(target_os = "windows") && cmd == "sh" && args == ["-c", "cargo build --target wasm32-wasi"])
        })
        .times(1)
        .returning(|_: &str, _: &[&str]| Ok(CommandOutput {
            success: true,
            stdout: b"Build successful".to_vec(),
            stderr: vec![],
        }));
    
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
        },
        deps,
    ).await;
    
    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Building 1 component")));
    assert!(output.iter().any(|s| s.contains("All components built successfully")));
}

#[tokio::test]
async fn test_build_with_release_flag() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: spin.toml with one build component
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"

[component.backend]
source = "target/wasm32-wasi/release/backend.wasm"
[component.backend.build]
command = "cargo build --target wasm32-wasi"
"#.to_string()));
    
    // Mock: spin installer
    fixture.spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));
    
    // Mock: build command execution with --release
    fixture.command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| {
            (cfg!(target_os = "windows") && cmd == "cmd" && args == ["/C", "cargo build --release --target wasm32-wasi"]) ||
            (!cfg!(target_os = "windows") && cmd == "sh" && args == ["-c", "cargo build --release --target wasm32-wasi"])
        })
        .times(1)
        .returning(|_: &str, _: &[&str]| Ok(CommandOutput {
            success: true,
            stdout: b"Build successful".to_vec(),
            stderr: vec![],
        }));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: true,
        },
        deps,
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_build_with_workdir() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: spin.toml with workdir
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"

[component.frontend]
source = "frontend/dist/frontend.wasm"
[component.frontend.build]
command = "npm run build"
workdir = "frontend"
"#.to_string()));
    
    // Mock: spin installer
    fixture.spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));
    
    // Mock: build command execution
    fixture.command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| {
            (cfg!(target_os = "windows") && cmd == "cmd" && args == ["/C", "npm run build"]) ||
            (!cfg!(target_os = "windows") && cmd == "sh" && args == ["-c", "npm run build"])
        })
        .times(1)
        .returning(|_: &str, _: &[&str]| Ok(CommandOutput {
            success: true,
            stdout: b"Build successful".to_vec(),
            stderr: vec![],
        }));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
        },
        deps,
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_build_multiple_components() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: spin.toml with multiple components
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"

[component.backend]
source = "target/wasm32-wasi/release/backend.wasm"
[component.backend.build]
command = "cargo build --target wasm32-wasi"

[component.frontend]
source = "frontend/dist/frontend.wasm"
[component.frontend.build]
command = "npm run build"
workdir = "frontend"
"#.to_string()));
    
    // Mock: spin installer
    fixture.spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));
    
    // Mock: build commands execution (both should be called)
    fixture.command_executor
        .expect_execute()
        .times(2)
        .returning(|cmd, args| {
            // Accept either backend or frontend build command
            if (cfg!(target_os = "windows") && cmd == "cmd") ||
               (!cfg!(target_os = "windows") && cmd == "sh") {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Build successful".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {} {:?}", cmd, args);
            }
        });
    
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
        },
        deps,
    ).await;
    
    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Building 2 components")));
}

#[tokio::test]
async fn test_build_failure() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: spin.toml
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"

[component.backend]
source = "target/wasm32-wasi/release/backend.wasm"
[component.backend.build]
command = "cargo build --target wasm32-wasi"
"#.to_string()));
    
    // Mock: spin installer
    fixture.spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));
    
    // Mock: build command fails
    fixture.command_executor
        .expect_execute()
        .times(1)
        .returning(|_: &str, _: &[&str]| Ok(CommandOutput {
            success: false,
            stdout: vec![],
            stderr: b"error: could not compile `backend`".to_vec(),
        }));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
        },
        deps,
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Build failed"));
}

#[tokio::test]
async fn test_build_invalid_toml() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: invalid spin.toml
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| Ok("invalid toml content".to_string()));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
        },
        deps,
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse spin.toml"));
}

#[tokio::test]
async fn test_build_with_custom_path() {
    let mut fixture = TestFixture::new();
    
    // Mock: spin.toml exists in custom path
    fixture.file_system
        .expect_exists()
        .with(eq(Path::new("/projects/myapp/spin.toml")))
        .times(1)
        .returning(|_| true);
    
    // Mock: spin.toml
    fixture.file_system
        .expect_read_to_string()
        .with(eq(Path::new("/projects/myapp/spin.toml")))
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"

[component.backend]
source = "target/wasm32-wasi/release/backend.wasm"
[component.backend.build]
command = "cargo build --target wasm32-wasi"
"#.to_string()));
    
    // Mock: spin installer
    fixture.spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));
    
    // Mock: build command
    fixture.command_executor
        .expect_execute()
        .times(1)
        .returning(|_: &str, _: &[&str]| Ok(CommandOutput {
            success: true,
            stdout: b"Build successful".to_vec(),
            stderr: vec![],
        }));
    
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: Some("/projects/myapp".into()),
            release: false,
        },
        deps,
    ).await;
    
    assert!(result.is_ok());
}

#[test]
fn test_parse_component_builds_empty() {
    let mut mock = MockFileSystemMock::new();
    
    mock.expect_read_to_string()
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"
"#.to_string()));
    
    let fs = Arc::new(mock) as Arc<dyn FileSystem>;
    let result = parse_component_builds(&fs, Path::new("spin.toml"));
    
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_parse_component_builds_multiple() {
    let mut mock = MockFileSystemMock::new();
    
    mock.expect_read_to_string()
        .times(1)
        .returning(|_| Ok(r#"
spin_manifest_version = "1"
name = "test-app"

[component.backend]
source = "backend.wasm"
[component.backend.build]
command = "cargo build"

[component.frontend]
source = "frontend.wasm"
[component.frontend.build]
command = "npm run build"
workdir = "frontend"

[component.static]
source = "static.wasm"
# No build section
"#.to_string()));
    
    let fs = Arc::new(mock) as Arc<dyn FileSystem>;
    let result = parse_component_builds(&fs, Path::new("spin.toml"));
    
    assert!(result.is_ok());
    let components = result.unwrap();
    assert_eq!(components.len(), 2);
    
    assert_eq!(components[0].name, "backend");
    assert_eq!(components[0].build_command, Some("cargo build".to_string()));
    assert_eq!(components[0].workdir, None);
    
    assert_eq!(components[1].name, "frontend");
    assert_eq!(components[1].build_command, Some("npm run build".to_string()));
    assert_eq!(components[1].workdir, Some("frontend".to_string()));
}