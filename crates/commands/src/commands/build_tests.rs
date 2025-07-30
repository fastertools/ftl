//! Unit tests for the build command

use std::path::Path;
use std::sync::Arc;

use mockall::predicate::*;

use crate::commands::build::*;
use crate::test_helpers::*;
use ftl_common::SpinInstaller;

struct TestFixture {
    file_system: MockFileSystemMock,
    command_executor: MockCommandExecutorMock,
    ui: Arc<TestUserInterface>,
    spin_installer: MockSpinInstallerMock,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            file_system: MockFileSystemMock::new(),
            command_executor: MockCommandExecutorMock::new(),
            ui: Arc::new(TestUserInterface::new()),
            spin_installer: MockSpinInstallerMock::new(),
        }
    }

    /// Mock that ftl.toml doesn't exist  
    fn mock_no_ftl_toml(&mut self) {
        self.file_system
            .expect_exists()
            .with(eq(Path::new("./ftl.toml")))
            .times(1)
            .returning(|_| false);
    }

    /// Mock that ftl.toml exists with the given content
    fn mock_ftl_toml_with_content(&mut self, content: &str) {
        let content = content.to_string();

        // Check if ftl.toml exists (yes)
        self.file_system
            .expect_exists()
            .with(eq(Path::new("./ftl.toml")))
            .times(1)
            .returning(|_| true);

        // Read ftl.toml content
        self.file_system
            .expect_read_to_string()
            .with(eq(Path::new("./ftl.toml")))
            .times(1)
            .returning(move |_| Ok(content.clone()));
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<BuildDependencies> {
        Arc::new(BuildDependencies {
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            ui: self.ui as Arc<dyn UserInterface>,
            spin_installer: Arc::new(self.spin_installer) as Arc<dyn SpinInstaller>,
        })
    }
}

#[tokio::test]
async fn test_build_no_ftl_toml() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml doesn't exist
    fixture.mock_no_ftl_toml();

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No ftl.toml found")
    );
}

#[tokio::test]
async fn test_build_no_components() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists with no tools (hence no build commands)
    fixture.mock_ftl_toml_with_content(
        r#"[project]
name = "test-app"
version = "0.1.0"
"#,
    );

    // No need to mock spin installer since we don't have components to build

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("No components with build commands found"))
    );
}

#[tokio::test]
async fn test_build_single_component_success() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists with one build component
    fixture.mock_ftl_toml_with_content(
        r#"
[project]
name = "test-app"
version = "0.1.0"

[tools.backend]
path = "backend"

[tools.backend.build]
command = "cargo build --target wasm32-wasi"
"#,
    );

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: build command execution - the command will have cd prefix
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| {
            (cfg!(target_os = "windows") && cmd == "cmd" && args.len() == 2 && args[0] == "/C")
                || (!cfg!(target_os = "windows")
                    && cmd == "sh"
                    && args.len() == 2
                    && args[0] == "-c")
        })
        .times(1)
        .returning(|_: &str, args: &[&str]| {
            // Verify the command contains the build command
            let command = args.get(1).unwrap_or(&"");
            assert!(command.contains("cargo build --target wasm32-wasi"));
            Ok(CommandOutput {
                success: true,
                stdout: b"Build successful".to_vec(),
                stderr: vec![],
            })
        });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Building 1 component")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("All components built successfully"))
    );
}

#[tokio::test]
async fn test_build_with_release_flag() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists with one build component
    fixture.mock_ftl_toml_with_content(
        r#"
[project]
name = "test-app"
version = "0.1.0"

[tools.backend]
path = "backend"

[tools.backend.build]
command = "cargo build --target wasm32-wasi"
"#,
    );

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: build command execution with --release (the command is modified by the build system)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| {
            (cfg!(target_os = "windows") && cmd == "cmd" && args.len() == 2 && args[0] == "/C")
                || (!cfg!(target_os = "windows")
                    && cmd == "sh"
                    && args.len() == 2
                    && args[0] == "-c")
        })
        .times(1)
        .returning(|_: &str, args: &[&str]| {
            // Verify the command contains the build command with --release
            let command = args.get(1).unwrap_or(&"");
            assert!(command.contains("cargo build --release --target wasm32-wasi"));
            Ok(CommandOutput {
                success: true,
                stdout: b"Build successful".to_vec(),
                stderr: vec![],
            })
        });

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: true,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_build_with_workdir() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists with workdir
    fixture.mock_ftl_toml_with_content(
        r#"
[project]
name = "test-app"
version = "0.1.0"

[tools.frontend]
path = "frontend"

[tools.frontend.build]
command = "npm run build"
workdir = "frontend"
"#,
    );

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: build command execution (now includes cd)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| {
            if cfg!(target_os = "windows") {
                cmd == "cmd"
                    && args.len() == 2
                    && args[0] == "/C"
                    && args[1].contains("cd")
                    && args[1].contains("frontend")
                    && args[1].contains("npm run build")
            } else {
                cmd == "sh"
                    && args.len() == 2
                    && args[0] == "-c"
                    && args[1].contains("cd")
                    && args[1].contains("frontend")
                    && args[1].contains("npm run build")
            }
        })
        .times(1)
        .returning(|_: &str, _: &[&str]| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Build successful".to_vec(),
                stderr: vec![],
            })
        });

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_build_multiple_components() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists with multiple components
    fixture.mock_ftl_toml_with_content(
        r#"
[project]
name = "test-app"
version = "0.1.0"

[tools.backend]
path = "backend"

[tools.backend.build]
command = "cargo build --target wasm32-wasi"

[tools.frontend]
path = "frontend"

[tools.frontend.build]
command = "npm run build"
workdir = "frontend"
"#,
    );

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: build commands execution (both should be called)
    fixture
        .command_executor
        .expect_execute()
        .times(2)
        .returning(|cmd, args| {
            // Accept either backend or frontend build command
            if (cfg!(target_os = "windows") && cmd == "cmd")
                || (!cfg!(target_os = "windows") && cmd == "sh")
            {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Build successful".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {cmd} {args:?}");
            }
        });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Building 2 components")));
}

#[tokio::test]
async fn test_build_failure() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists
    fixture.mock_ftl_toml_with_content(
        r#"
[project]
name = "test-app"
version = "0.1.0"

[tools.backend]
path = "backend"

[tools.backend.build]
command = "cargo build --target wasm32-wasi"
"#,
    );

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: build command fails
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_: &str, _: &[&str]| {
            Ok(CommandOutput {
                success: false,
                stdout: vec![],
                stderr: b"error: could not compile `backend`".to_vec(),
            })
        });

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Build failed"));
}

#[tokio::test]
async fn test_build_invalid_toml() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: invalid ftl.toml
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(|_| Ok("invalid toml content".to_string()));

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: None,
            release: false,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    // The error will be about parsing ftl.toml now
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse ftl.toml") || error_msg.contains("missing field"));
}

#[tokio::test]
async fn test_build_with_custom_path() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists in custom path
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("/projects/myapp/ftl.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: ftl.toml content
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("/projects/myapp/ftl.toml")))
        .times(1)
        .returning(|_| {
            Ok(r#"
[project]
name = "test-app"
version = "0.1.0"

[tools.backend]
path = "backend"

[tools.backend.build]
command = "cargo build --target wasm32-wasi"
"#
            .to_string())
        });

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: build command
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_: &str, _: &[&str]| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Build successful".to_vec(),
                stderr: vec![],
            })
        });

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        BuildConfig {
            path: Some("/projects/myapp".into()),
            release: false,
            export: None,
            export_out: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
}

#[test]
fn test_parse_component_builds_empty() {
    let mut mock = MockFileSystemMock::new();

    mock.expect_read_to_string().times(1).returning(|_| {
        Ok(r#"
spin_manifest_version = "1"
name = "test-app"
"#
        .to_string())
    });

    let fs = Arc::new(mock) as Arc<dyn FileSystem>;
    let result = parse_component_builds(&fs, Path::new("spin.toml"));

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_parse_component_builds_multiple() {
    let mut mock = MockFileSystemMock::new();

    mock.expect_read_to_string().times(1).returning(|_| {
        Ok(r#"
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
"#
        .to_string())
    });

    let fs = Arc::new(mock) as Arc<dyn FileSystem>;
    let result = parse_component_builds(&fs, Path::new("spin.toml"));

    assert!(result.is_ok());
    let components = result.unwrap();
    assert_eq!(components.len(), 2);

    assert_eq!(components[0].name, "backend");
    assert_eq!(components[0].build_command, Some("cargo build".to_string()));
    assert_eq!(components[0].workdir, None);

    assert_eq!(components[1].name, "frontend");
    assert_eq!(
        components[1].build_command,
        Some("npm run build".to_string())
    );
    assert_eq!(components[1].workdir, Some("frontend".to_string()));
}

#[test]
fn test_prepare_build_command() {
    // Test cargo build
    assert_eq!(
        prepare_build_command("cargo build", true),
        "cargo build --release"
    );
    assert_eq!(
        prepare_build_command("cargo build --target wasm32-wasi", true),
        "cargo build --release --target wasm32-wasi"
    );
    assert_eq!(
        prepare_build_command("cargo build --release", true),
        "cargo build --release"
    );

    // Test npm
    assert_eq!(
        prepare_build_command("npm run build", true),
        "npm run build"
    );

    // Test other commands
    assert_eq!(prepare_build_command("make", true), "make");

    // Test non-release mode
    assert_eq!(prepare_build_command("cargo build", false), "cargo build");
}

#[test]
fn test_get_shell_command() {
    let command = "cargo build --release";

    #[cfg(target_os = "windows")]
    {
        let (cmd, args) = get_shell_command(command);
        assert_eq!(cmd, "cmd");
        assert_eq!(args, vec!["/C", command]);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let (cmd, args) = get_shell_command(command);
        assert_eq!(cmd, "sh");
        assert_eq!(args, vec!["-c", command]);
    }
}
