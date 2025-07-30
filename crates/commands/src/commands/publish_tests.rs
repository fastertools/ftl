//! Unit tests for the publish command

use mockall::predicate::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::commands::publish::{
    BuildExecutor, ProcessExecutor, ProcessOutput, PublishConfig, PublishDependencies,
    SpinInstaller, execute_with_deps,
};
use crate::test_helpers::*;
use ftl_common::ui::TestUserInterface;
use ftl_runtime::deps::{FileSystem, UserInterface};

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

    fn with_failure(mut self, message: &str) -> Self {
        self.should_fail = true;
        self.error_message = Some(message.to_string());
        self
    }
}

#[async_trait::async_trait]
impl BuildExecutor for MockBuildExecutor {
    async fn execute(&self, _path: Option<PathBuf>, _release: bool) -> Result<(), anyhow::Error> {
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

// Mock implementation of ProcessExecutor
struct MockProcessExecutor {
    expected_commands: Vec<(String, Vec<String>, Option<PathBuf>, ProcessOutput)>,
    call_count: std::sync::Mutex<usize>,
    flexible_args: bool,
}

impl MockProcessExecutor {
    fn new() -> Self {
        Self {
            expected_commands: Vec::new(),
            call_count: std::sync::Mutex::new(0),
            flexible_args: false,
        }
    }

    fn new_flexible() -> Self {
        Self {
            expected_commands: Vec::new(),
            call_count: std::sync::Mutex::new(0),
            flexible_args: true,
        }
    }

    fn expect_execute(
        mut self,
        command: &str,
        args: &[&str],
        working_dir: Option<PathBuf>,
        output: ProcessOutput,
    ) -> Self {
        self.expected_commands.push((
            command.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
            working_dir,
            output,
        ));
        self
    }
}

impl ProcessExecutor for MockProcessExecutor {
    fn execute(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<ProcessOutput, anyhow::Error> {
        let mut count = self.call_count.lock().unwrap();
        let index = *count;
        *count += 1;

        if index >= self.expected_commands.len() {
            return Err(anyhow::anyhow!("Unexpected command execution"));
        }

        let (expected_cmd, expected_args, expected_dir, output) = &self.expected_commands[index];

        if command != expected_cmd {
            return Err(anyhow::anyhow!(
                "Expected command '{}', got '{}'",
                expected_cmd,
                command
            ));
        }

        if self.flexible_args {
            // In flexible mode, only check that key args are present
            let args_vec: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();

            // Check that the command has the basic structure we expect
            if !args_vec.contains(&"registry".to_string())
                || !args_vec.contains(&"push".to_string())
            {
                return Err(anyhow::anyhow!(
                    "Expected 'registry push' in args, got {:?}",
                    args_vec
                ));
            }

            // Check for expected optional args
            for expected_arg in expected_args {
                if expected_arg.starts_with("--") && !args_vec.contains(expected_arg) {
                    return Err(anyhow::anyhow!(
                        "Expected arg '{}' not found in {:?}",
                        expected_arg,
                        args_vec
                    ));
                }
            }
        } else {
            let args_vec: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
            if args_vec != *expected_args {
                return Err(anyhow::anyhow!(
                    "Expected args {:?}, got {:?}",
                    expected_args,
                    args_vec
                ));
            }
        }

        if working_dir.map(std::path::Path::to_path_buf) != *expected_dir {
            return Err(anyhow::anyhow!(
                "Expected working dir {:?}, got {:?}",
                expected_dir,
                working_dir
            ));
        }

        Ok(ProcessOutput {
            success: output.success,
            stdout: output.stdout.clone(),
            stderr: output.stderr.clone(),
        })
    }
}

// Mock implementation of SpinInstaller
struct MockSpinInstaller {
    should_fail: bool,
    spin_path: PathBuf,
}

impl MockSpinInstaller {
    fn new() -> Self {
        Self {
            should_fail: false,
            spin_path: PathBuf::from("/usr/local/bin/spin"),
        }
    }

    fn with_failure() -> Self {
        Self {
            should_fail: true,
            spin_path: PathBuf::new(),
        }
    }
}

#[async_trait::async_trait]
impl SpinInstaller for MockSpinInstaller {
    async fn check_and_install(&self) -> Result<String, anyhow::Error> {
        if self.should_fail {
            Err(anyhow::anyhow!("Failed to install spin"))
        } else {
            Ok(self.spin_path.to_string_lossy().to_string())
        }
    }
}

struct TestFixture {
    ui: Arc<TestUserInterface>,
    file_system: MockFileSystemMock,
    process_executor: Arc<MockProcessExecutor>,
    spin_installer: Arc<MockSpinInstaller>,
    build_executor: Arc<MockBuildExecutor>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            file_system: MockFileSystemMock::new(),
            process_executor: Arc::new(MockProcessExecutor::new()),
            spin_installer: Arc::new(MockSpinInstaller::new()),
            build_executor: Arc::new(MockBuildExecutor::new()),
        }
    }

    /// Mock that ftl.toml exists (always required)
    fn mock_ftl_toml_exists(&mut self, path: Option<&Path>) {
        let base_path = path.unwrap_or(Path::new("."));
        let ftl_path = base_path.join("ftl.toml");

        // Check if ftl.toml exists (yes)
        self.file_system
            .expect_exists()
            .with(eq(ftl_path.clone()))
            .times(1)
            .returning(|_| true);

        // Read ftl.toml content
        self.file_system
            .expect_read_to_string()
            .with(eq(ftl_path))
            .times(1)
            .returning(|_| {
                Ok(r#"
[project]
name = "test-project"
version = "0.1.0"

[tools.test-tool]
path = "test-tool"

[tools.test-tool.build]
command = "cargo build --release --target wasm32-wasip1"
"#
                .to_string())
            });
    }

    /// Create a flexible process executor that expects spin registry push
    fn expect_spin_push(
        &mut self,
        extra_args: &[&str],
        working_dir: Option<PathBuf>,
        output: ProcessOutput,
    ) {
        let mut args = vec!["registry", "push"];
        args.extend_from_slice(extra_args);

        self.process_executor = Arc::new(MockProcessExecutor::new_flexible().expect_execute(
            "/usr/local/bin/spin",
            &args,
            working_dir.or_else(|| Some(PathBuf::from("."))),
            output,
        ));
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<PublishDependencies> {
        Arc::new(PublishDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            process_executor: self.process_executor as Arc<dyn ProcessExecutor>,
            spin_installer: self.spin_installer as Arc<dyn SpinInstaller>,
            build_executor: self.build_executor as Arc<dyn BuildExecutor>,
        })
    }
}

#[tokio::test]
async fn test_publish_success() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin registry push succeeds
    fixture.expect_spin_push(
        &[],
        None,
        ProcessOutput {
            success: true,
            stdout: "Published to registry successfully!".to_string(),
            stderr: String::new(),
        },
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: None,
        tag: None,
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Publishing project")));
    assert!(output.iter().any(|s| s.contains("Building project")));
    assert!(output.iter().any(|s| s.contains("Publishing to registry")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Project published successfully"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Published to registry successfully!"))
    );
}

#[tokio::test]
async fn test_publish_no_ftl_toml() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml doesn't exist
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./ftl.toml")))
        .times(1)
        .returning(|_| false);

    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: None,
        tag: None,
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No ftl.toml found")
    );
}

#[tokio::test]
async fn test_publish_with_custom_path() {
    let mut fixture = TestFixture::new();

    let custom_path = PathBuf::from("/my/project");

    // Mock: ftl.toml exists at custom path
    fixture.mock_ftl_toml_exists(Some(&custom_path));

    // Mock: spin registry push with custom path
    fixture.expect_spin_push(
        &[],
        Some(custom_path.clone()),
        ProcessOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        },
    );

    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: Some(PathBuf::from("/my/project")),
        registry: None,
        tag: None,
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_publish_with_registry() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin registry push with custom registry
    fixture.expect_spin_push(
        &["--registry", "https://my.registry.com"],
        None,
        ProcessOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        },
    );

    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: Some("https://my.registry.com".to_string()),
        tag: None,
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_publish_with_tag() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin registry push with tag
    fixture.expect_spin_push(
        &["--tag", "v1.0.0"],
        None,
        ProcessOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        },
    );

    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: None,
        tag: Some("v1.0.0".to_string()),
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_publish_with_registry_and_tag() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin registry push with both registry and tag
    fixture.expect_spin_push(
        &["--registry", "https://my.registry.com", "--tag", "v2.0.0"],
        None,
        ProcessOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        },
    );

    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: Some("https://my.registry.com".to_string()),
        tag: Some("v2.0.0".to_string()),
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_publish_spin_install_fails() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin installer fails
    fixture.spin_installer = Arc::new(MockSpinInstaller::with_failure());

    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: None,
        tag: None,
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to install spin")
    );
}

#[tokio::test]
async fn test_publish_build_fails() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: build fails
    fixture.build_executor =
        Arc::new(MockBuildExecutor::new().with_failure("Build error: missing dependencies"));

    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: None,
        tag: None,
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Build error: missing dependencies")
    );
}

#[tokio::test]
async fn test_publish_registry_push_fails() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin registry push fails
    fixture.expect_spin_push(
        &[],
        None,
        ProcessOutput {
            success: false,
            stdout: "Error: Authentication failed".to_string(),
            stderr: "Please login first".to_string(),
        },
    );

    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: None,
        tag: None,
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Publishing failed"));
    assert!(error_msg.contains("Authentication failed"));
    assert!(error_msg.contains("Please login first"));
}

#[tokio::test]
async fn test_publish_with_custom_tag() {
    let mut fixture = TestFixture::new();
    let build_executor = Arc::new(MockBuildExecutor::new());
    fixture.build_executor = build_executor.clone();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin registry push with tag
    fixture.expect_spin_push(
        &["--tag", "v1.0.0"],
        None,
        ProcessOutput {
            success: true,
            stdout: "Published with tag v1.0.0".to_string(),
            stderr: String::new(),
        },
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: None,
        tag: Some("v1.0.0".to_string()),
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Published with tag v1.0.0"))
    );
}

#[tokio::test]
async fn test_publish_with_custom_registry_and_tag() {
    let mut fixture = TestFixture::new();
    let build_executor = Arc::new(MockBuildExecutor::new());
    fixture.build_executor = build_executor.clone();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin registry push with registry and tag
    fixture.expect_spin_push(
        &["--registry", "https://my-registry.com", "--tag", "latest"],
        None,
        ProcessOutput {
            success: true,
            stdout: "Published to custom registry".to_string(),
            stderr: String::new(),
        },
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: Some("https://my-registry.com".to_string()),
        tag: Some("latest".to_string()),
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_ok());

    // Verify output includes stdout
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Published to custom registry"))
    );
}

#[tokio::test]
async fn test_publish_empty_stdout() {
    let mut fixture = TestFixture::new();
    let build_executor = Arc::new(MockBuildExecutor::new());
    fixture.build_executor = build_executor.clone();

    // Mock: ftl.toml exists (required)
    fixture.mock_ftl_toml_exists(None);

    // Mock: spin registry push with empty stdout
    fixture.expect_spin_push(
        &[],
        None,
        ProcessOutput {
            success: true,
            stdout: "   \n  \t  ".to_string(), // Whitespace only
            stderr: String::new(),
        },
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let config = PublishConfig {
        path: None,
        registry: None,
        tag: None,
    };

    let result = execute_with_deps(config, deps).await;
    assert!(result.is_ok());

    // Verify empty stdout is not printed
    let output = ui.get_output();
    assert!(!output.iter().any(|s| s.trim().is_empty() && !s.is_empty()));
}

#[tokio::test]
#[ignore = "This test creates real dependencies"]
async fn test_execute_function() {
    use crate::commands::publish::{PublishArgs, execute};

    // Test the main execute function
    let args = PublishArgs {
        path: Some(PathBuf::from("/tmp/nonexistent")),
        registry: None,
        tag: None,
    };

    // This will fail because the path doesn't exist, but we're testing
    // that the function creates all the right dependencies
    let result = execute(args).await;
    assert!(result.is_err());
}
