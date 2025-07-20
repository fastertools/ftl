//! Unit tests for the init command

use std::path::Path;
use std::sync::Arc;

use mockall::predicate::*;

use crate::commands::init::*;
use crate::deps::*;
use crate::test_helpers::*;
use crate::ui::TestUserInterface;

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
async fn test_init_templates_not_installed() {
    let mut fixture = TestFixture::new();

    setup_basic_init_mocks(&mut fixture);

    // Mock: templates list doesn't contain ftl-mcp-server
    fixture
        .command_executor
        .expect_execute()
        .withf(|_: &str, args: &[&str]| args == ["templates", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"some-other-template\nanother-template".to_vec(),
                stderr: vec![],
            })
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
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("ftl-mcp templates not installed")
    );
}

#[tokio::test]
async fn test_init_spin_new_fails() {
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

    // Setup all command executor expectations in one returning function
    fixture.command_executor
        .expect_execute()
        .times(2) // templates list + spin new
        .returning(|path, args| {
            println!("Mock execute called with path: {}, args: {:?}", path, args);
            if args == ["templates", "list"] {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"ftl-mcp-server\nsome-other-template".to_vec(),
                    stderr: vec![],
                })
            } else if args.len() >= 2 && args[0] == "new" && args.contains(&"my-project") {
                // Simulate failure for spin new
                Ok(CommandOutput {
                    success: false,
                    stdout: vec![],
                    stderr: b"Failed to create project: some error".to_vec(),
                })
            } else {
                panic!("Unexpected command: {} {:?}", path, args);
            }
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
        err_msg.contains("Failed to create project"),
        "Expected 'Failed to create project', got: {}",
        err_msg
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

    // Setup all command executor expectations in one returning function
    fixture.command_executor
        .expect_execute()
        .times(2) // templates list + spin new
        .returning(|path, args| {
            println!("Mock execute called with path: {}, args: {:?}", path, args);
            if args == ["templates", "list"] {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"ftl-mcp-server\nsome-other-template".to_vec(),
                    stderr: vec![],
                })
            } else if args.len() >= 2 && args[0] == "new" && args.contains(&"my-project") {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Project created successfully".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {} {:?}", path, args);
            }
        });

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

    if let Err(e) = &result {
        eprintln!("Init error: {}", e);
    }
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("MCP project initialized!"))
    );
    assert!(output.iter().any(|s| s.contains("cd my-project &&")));
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

    // Setup all command executor expectations in one returning function
    fixture.command_executor
        .expect_execute()
        .times(2) // templates list + spin new
        .returning(|path, args| {
            println!("Mock execute called with path: {}, args: {:?}", path, args);
            if args == ["templates", "list"] {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"ftl-mcp-server\nsome-other-template".to_vec(),
                    stderr: vec![],
                })
            } else if args.len() >= 2 && args[0] == "new" && args.contains(&".") {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Project created successfully".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {} {:?}", path, args);
            }
        });

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

    if let Err(e) = &result {
        eprintln!("Init error: {}", e);
    }
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

    // Mock: directory doesn't exist - expects "my-project" since that's the default provided to prompt_input
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("my-project")))
        .times(1)
        .returning(|_| false);

    // Mock: command executor for both templates check and spin new
    fixture.command_executor
        .expect_execute()
        .times(2) // Called twice: once for templates, once for spin new
        .returning(|_path, args| {
            if args == ["templates", "list"] {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"ftl-mcp-server\nsome-other-template".to_vec(),
                    stderr: vec![],
                })
            } else if args[0] == "new" && args.contains(&"my-project") {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Project created successfully".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {:?}", args);
            }
        });

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

    if let Err(e) = &result {
        eprintln!("Init error: {}", e);
    }
    assert!(result.is_ok());

    // Verify output - it will use "my-project" as the default from TestUserInterface
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

fn setup_templates_installed(fixture: &mut TestFixture) {
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|path, args| {
            println!("Mock execute called with path: {}, args: {:?}", path, args);
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
