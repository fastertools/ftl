//! Unit tests for the publish command

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
}

impl MockProcessExecutor {
    fn new() -> Self {
        Self {
            expected_commands: Vec::new(),
            call_count: std::sync::Mutex::new(0),
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

        let args_vec: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
        if args_vec != *expected_args {
            return Err(anyhow::anyhow!(
                "Expected args {:?}, got {:?}",
                expected_args,
                args_vec
            ));
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

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| path == Path::new("./spin.toml"))
        .times(1)
        .returning(|_| true);

    // Mock: spin registry push succeeds
    fixture.process_executor = Arc::new(MockProcessExecutor::new().expect_execute(
        "/usr/local/bin/spin",
        &["registry", "push"],
        Some(PathBuf::from(".")),
        ProcessOutput {
            success: true,
            stdout: "Published to registry successfully!".to_string(),
            stderr: String::new(),
        },
    ));

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
async fn test_publish_no_spin_toml() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml doesn't exist
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| path == Path::new("./spin.toml"))
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
            .contains("No spin.toml found")
    );
}

#[tokio::test]
async fn test_publish_with_custom_path() {
    let mut fixture = TestFixture::new();

    let custom_path = PathBuf::from("/my/project");

    // Mock: spin.toml exists at custom path
    fixture
        .file_system
        .expect_exists()
        .withf(|path: &Path| path == Path::new("/my/project/spin.toml"))
        .times(1)
        .returning(|_| true);

    // Mock: spin registry push with custom path
    fixture.process_executor = Arc::new(MockProcessExecutor::new().expect_execute(
        "/usr/local/bin/spin",
        &["registry", "push"],
        Some(custom_path.clone()),
        ProcessOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        },
    ));

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

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .times(1)
        .returning(|_| true);

    // Mock: spin registry push with custom registry
    fixture.process_executor = Arc::new(MockProcessExecutor::new().expect_execute(
        "/usr/local/bin/spin",
        &["registry", "push", "--registry", "https://my.registry.com"],
        Some(PathBuf::from(".")),
        ProcessOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        },
    ));

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

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .times(1)
        .returning(|_| true);

    // Mock: spin registry push with tag
    fixture.process_executor = Arc::new(MockProcessExecutor::new().expect_execute(
        "/usr/local/bin/spin",
        &["registry", "push", "--tag", "v1.0.0"],
        Some(PathBuf::from(".")),
        ProcessOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        },
    ));

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

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .times(1)
        .returning(|_| true);

    // Mock: spin registry push with both registry and tag
    fixture.process_executor = Arc::new(MockProcessExecutor::new().expect_execute(
        "/usr/local/bin/spin",
        &[
            "registry",
            "push",
            "--registry",
            "https://my.registry.com",
            "--tag",
            "v2.0.0",
        ],
        Some(PathBuf::from(".")),
        ProcessOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        },
    ));

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

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .times(1)
        .returning(|_| true);

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

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .times(1)
        .returning(|_| true);

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

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .times(1)
        .returning(|_| true);

    // Mock: spin registry push fails
    fixture.process_executor = Arc::new(MockProcessExecutor::new().expect_execute(
        "/usr/local/bin/spin",
        &["registry", "push"],
        Some(PathBuf::from(".")),
        ProcessOutput {
            success: false,
            stdout: "Error: Authentication failed".to_string(),
            stderr: "Please login first".to_string(),
        },
    ));

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
