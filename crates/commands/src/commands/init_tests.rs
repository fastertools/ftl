//! Unit tests for the init command

use std::path::Path;
use std::sync::Arc;

use mockall::predicate::*;

use crate::commands::init::*;
use crate::test_helpers::*;
use ftl_common::SpinInstaller;
use ftl_runtime::deps::*;

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

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<InitDependencies> {
        Arc::new(InitDependencies {
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            ui: self.ui as Arc<dyn UserInterface>,
            spin_installer: Arc::new(self.spin_installer) as Arc<dyn SpinInstaller>,
        })
    }
}

#[tokio::test]
async fn test_init_invalid_name_uppercase() {
    let mut fixture = TestFixture::new();

    // Mock: spin installer - still needs to be called
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    let deps = fixture.to_deps();

    let result = execute_with_deps(
        InitConfig {
            name: Some("TestProject".to_string()),
            here: false,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("must be lowercase")
    );
}

#[tokio::test]
async fn test_init_invalid_name_leading_hyphen() {
    let mut fixture = TestFixture::new();

    // Mock: spin installer - still needs to be called
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    let deps = fixture.to_deps();

    let result = execute_with_deps(
        InitConfig {
            name: Some("-project".to_string()),
            here: false,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("cannot start or end with hyphens")
    );
}

#[tokio::test]
async fn test_init_directory_already_exists() {
    let mut fixture = TestFixture::new();

    // Mock: directory exists
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("my-project")))
        .times(1)
        .returning(|_| true);

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        InitConfig {
            name: Some("my-project".to_string()),
            here: false,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_init_here_not_empty() {
    let mut fixture = TestFixture::new();

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: current directory has files - check all common files
    // The is_directory_empty function checks up to 7 files but stops early if it finds one
    fixture.file_system
        .expect_exists()
        .times(3) // It will stop after finding spin.toml (3rd file in the list)
        .returning(|path| {
            // Only spin.toml exists
            path == Path::new("./spin.toml")
        });

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        InitConfig {
            name: Some("my-project".to_string()),
            here: true,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not empty"));
}

#[tokio::test]
async fn test_init_creates_ftl_toml() {
    let mut fixture = TestFixture::new();

    setup_basic_init_mocks(&mut fixture);


    // Mock: write ftl.toml
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/ftl.toml")), always())
        .times(1)
        .returning(|_, content| {
            assert!(content.contains("[project]"));
            assert!(content.contains("name = \"my-project\""));
            Ok(())
        });

    // Mock: write README.md
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/README.md")), always())
        .times(1)
        .returning(|_, _| Ok(()));

    // Mock: write .gitignore
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/.gitignore")), always())
        .times(1)
        .returning(|_, _| Ok(()));
    

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        InitConfig {
            name: Some("my-project".to_string()),
            here: false,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_init_write_fails() {
    let mut fixture = TestFixture::new();

    // Setup basic mocks
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("my-project")))
        .times(1)
        .returning(|_| false);

    // Mock: write ftl.toml fails
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/ftl.toml")), always())
        .times(1)
        .returning(|_, _| {
            Err(anyhow::anyhow!("Failed to write ftl.toml: permission denied"))
        });

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        InitConfig {
            name: Some("my-project".to_string()),
            here: false,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Failed to write ftl.toml"),
        "Expected 'Failed to write ftl.toml', got: {err_msg}"
    );
}

#[tokio::test]
async fn test_init_success() {
    let mut fixture = TestFixture::new();

    // Setup basic mocks
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("my-project")))
        .times(1)
        .returning(|_| false);


    // Mock: write ftl.toml
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/ftl.toml")), always())
        .times(1)
        .returning(|_, _| Ok(()));

    // Mock: write README.md
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/README.md")), always())
        .times(1)
        .returning(|_, _| Ok(()));

    // Mock: write .gitignore
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/.gitignore")), always())
        .times(1)
        .returning(|_, _| Ok(()));
    

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        InitConfig {
            name: Some("my-project".to_string()),
            here: false,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("MCP project initialized!"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("http://localhost:3000/mcp"))
    );
}

#[tokio::test]
async fn test_init_here_success() {
    let mut fixture = TestFixture::new();

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: current directory is empty - no files exist
    fixture.file_system
        .expect_exists()
        .times(7) // 7 common files we check
        .returning(|_| false);


    // Mock: write ftl.toml
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("./ftl.toml")), always())
        .times(1)
        .returning(|_, _| Ok(()));

    // Mock: write README.md
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("./README.md")), always())
        .times(1)
        .returning(|_, _| Ok(()));

    // Mock: write .gitignore
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("./.gitignore")), always())
        .times(1)
        .returning(|_, _| Ok(()));
    

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        InitConfig {
            name: Some("my-project".to_string()),
            here: true,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify output doesn't contain cd instruction
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("MCP project initialized!"))
    );
    assert!(!output.iter().any(|s| s.contains("cd my-project")));
    assert!(
        output
            .iter()
            .any(|s| s == "  ftl add           # Add a tool to the project")
    );
}

#[tokio::test]
async fn test_init_interactive_name() {
    let mut fixture = TestFixture::new();

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: directory doesn't exist - expects "my-project" since TestUserInterface returns the default value
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("my-project")))
        .times(1)
        .returning(|_| false);


    // Mock: write ftl.toml
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/ftl.toml")), always())
        .times(1)
        .returning(|_, _| Ok(()));

    // Mock: write README.md
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/README.md")), always())
        .times(1)
        .returning(|_, _| Ok(()));

    // Mock: write .gitignore
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("my-project/.gitignore")), always())
        .times(1)
        .returning(|_, _| Ok(()));
    

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        InitConfig {
            name: None, // No name provided, will prompt
            here: false,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("MCP project initialized!"))
    );
}

// Helper functions
fn setup_basic_init_mocks(fixture: &mut TestFixture) {
    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: directory doesn't exist
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("my-project")))
        .times(1)
        .returning(|_| false);
}

#[allow(dead_code)]
fn setup_templates_installed(fixture: &mut TestFixture) {
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|path, args| {
            println!("Mock execute called with path: {path}, args: {args:?}");
            if args == ["templates", "list"] {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"ftl-mcp-server\nsome-other-template".to_vec(),
                    stderr: vec![],
                })
            } else {
                // Return default for other commands
                Ok(CommandOutput {
                    success: true,
                    stdout: vec![],
                    stderr: vec![],
                })
            }
        });
}
