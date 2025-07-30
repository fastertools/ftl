//! Unit tests for the add command

use std::path::Path;
use std::sync::Arc;

use mockall::predicate::*;

use crate::commands::add::*;
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
    
    /// Mock that ftl.toml doesn't exist but spin.toml does
    fn mock_spin_toml_exists(&mut self) {
        self.file_system
            .expect_exists()
            .with(eq(Path::new("ftl.toml")))
            .times(1)
            .returning(|_| false);
            
        self.file_system
            .expect_exists()
            .with(eq(Path::new("spin.toml")))
            .times(1)
            .returning(|_| true);
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<AddDependencies> {
        Arc::new(AddDependencies {
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            ui: self.ui as Arc<dyn UserInterface>,
            spin_installer: Arc::new(self.spin_installer) as Arc<dyn SpinInstaller>,
        })
    }
}

#[tokio::test]
async fn test_add_not_in_spin_project() {
    let mut fixture = TestFixture::new();

    // Mock: ftl.toml doesn't exist
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("ftl.toml")))
        .times(1)
        .returning(|_| false);
    
    // Mock: spin.toml doesn't exist either
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| false);

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: Some("my-tool".to_string()),
            language: None,
            git: None,
            branch: None,
            dir: None,
            tar: None,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No spin.toml or ftl.toml found")
    );
}

#[tokio::test]
async fn test_add_invalid_name_uppercase() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml exists
    fixture.mock_spin_toml_exists();

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: Some("MyTool".to_string()),
            language: None,
            git: None,
            branch: None,
            dir: None,
            tar: None,
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
async fn test_add_invalid_name_leading_hyphen() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml exists
    fixture.mock_spin_toml_exists();

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: Some("-tool".to_string()),
            language: None,
            git: None,
            branch: None,
            dir: None,
            tar: None,
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
async fn test_add_templates_not_installed() {
    let mut fixture = TestFixture::new();

    setup_basic_add_mocks(&mut fixture);

    // Mock: spin add fails with template not found
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_, args| {
            if args.contains(&"add") {
                Ok(CommandOutput {
                    success: false,
                    stdout: vec![],
                    stderr: b"Error: no such template 'ftl-mcp-rust'".to_vec(),
                })
            } else {
                panic!("Unexpected command: {args:?}");
            }
        });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: Some("my-tool".to_string()),
            language: Some("rust".to_string()),
            git: None,
            branch: None,
            dir: None,
            tar: None,
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

    // Verify error message was shown
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("ftl-mcp templates not found"))
    );
    assert!(output.iter().any(|s| s.contains("ftl setup templates")));
}

#[tokio::test]
async fn test_add_success_rust() {
    let mut fixture = TestFixture::new();

    setup_basic_add_mocks(&mut fixture);

    // Mock: spin add succeeds
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_, args| {
            if args.contains(&"add") && args.contains(&"-t") && args.contains(&"ftl-mcp-rust") {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Tool added successfully".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {args:?}");
            }
        });


    // Mock: read spin.toml
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| {
            Ok(r#"
[variables]
tool_components = { default = "" }

[[trigger.http]]
route = "/mcp/..."
"#
            .to_string())
        });

    // Mock: write updated spin.toml
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("spin.toml")), always())
        .times(1)
        .returning(|_, content| {
            assert!(content.contains("my-tool"));
            Ok(())
        });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: Some("my-tool".to_string()),
            language: Some("rust".to_string()),
            git: None,
            branch: None,
            dir: None,
            tar: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify success message
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Rust tool added successfully"))
    );
    assert!(output.iter().any(|s| s.contains("my-tool/src/lib.rs")));
}

#[tokio::test]
async fn test_add_success_typescript() {
    let mut fixture = TestFixture::new();

    setup_basic_add_mocks(&mut fixture);

    // Mock: spin add succeeds
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_, args| {
            if args.contains(&"add") && args.contains(&"-t") && args.contains(&"ftl-mcp-ts") {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Tool added successfully".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {args:?}");
            }
        });

    // Mock: read spin.toml
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| {
            Ok(r#"
[variables]
tool_components = { default = "existing-tool" }
"#
            .to_string())
        });

    // Mock: write updated spin.toml
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("spin.toml")), always())
        .times(1)
        .returning(|_, content| {
            assert!(content.contains("existing-tool,my-ts-tool"));
            Ok(())
        });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: Some("my-ts-tool".to_string()),
            language: Some("typescript".to_string()),
            git: None,
            branch: None,
            dir: None,
            tar: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify success message
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("TypeScript tool added successfully"))
    );
    assert!(output.iter().any(|s| s.contains("my-ts-tool/src/index.ts")));
}

#[tokio::test]
async fn test_add_with_git_template() {
    let mut fixture = TestFixture::new();

    setup_basic_add_mocks(&mut fixture);

    // Mock: spin add with git template
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_, args| {
            if args.contains(&"add")
                && args.contains(&"--git")
                && args.contains(&"https://github.com/example/template.git")
                && args.contains(&"--branch")
                && args.contains(&"main")
            {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Tool added successfully".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {args:?}");
            }
        });

    // Mock: read/write spin.toml
    setup_spin_toml_mocks(&mut fixture);

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: Some("custom-tool".to_string()),
            language: Some("rust".to_string()),
            git: Some("https://github.com/example/template.git".to_string()),
            branch: Some("main".to_string()),
            dir: None,
            tar: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_interactive_prompts() {
    let mut fixture = TestFixture::new();

    setup_basic_add_mocks(&mut fixture);

    // UI will provide default values for prompts
    // TestUserInterface returns "test-value" for prompt_input by default

    // Mock: spin add succeeds
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_, args| {
            if args.contains(&"add") && args.contains(&"test-value") {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Tool added successfully".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {args:?}");
            }
        });

    setup_spin_toml_mocks(&mut fixture);

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: None,        // Will prompt for name
            language: None,    // Will prompt for language
            git: None,
            branch: None,
            dir: None,
            tar: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_javascript_mapped_to_typescript() {
    let mut fixture = TestFixture::new();

    setup_basic_add_mocks(&mut fixture);

    // Mock: spin add succeeds with TypeScript template
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_, args| {
            // Should use ftl-mcp-ts template even though user specified javascript
            if args.contains(&"add") && args.contains(&"-t") && args.contains(&"ftl-mcp-ts") {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Tool added successfully".to_vec(),
                    stderr: vec![],
                })
            } else {
                panic!("Unexpected command: {args:?}");
            }
        });

    setup_spin_toml_mocks(&mut fixture);

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        AddConfig {
            name: Some("js-tool".to_string()),
            language: Some("javascript".to_string()), // Should be mapped to TypeScript
            git: None,
            branch: None,
            dir: None,
            tar: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
}

// Helper functions
fn setup_basic_add_mocks(fixture: &mut TestFixture) {
    // Mock: spin.toml exists
    fixture.mock_spin_toml_exists();

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));
}

fn setup_spin_toml_mocks(fixture: &mut TestFixture) {
    // Mock: read spin.toml
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("spin.toml")))
        .times(1)
        .returning(|_| {
            Ok(r#"
[variables]
tool_components = { default = "" }
"#
            .to_string())
        });

    // Mock: write updated spin.toml
    fixture
        .file_system
        .expect_write_string()
        .with(eq(Path::new("spin.toml")), always())
        .times(1)
        .returning(|_, _| Ok(()));
}
