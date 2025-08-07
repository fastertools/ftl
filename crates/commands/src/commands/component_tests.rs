//! Unit tests for component commands

use std::path::Path;
use std::sync::Arc;

use crate::commands::component::*;
use crate::test_helpers::*;
use anyhow::Result;
use ftl_common::ui::TestUserInterface;
use ftl_runtime::deps::{
    CommandExecutor, CommandOutput, FileSystem, MessageStyle, MultiProgressManager,
    ProgressIndicator, UserInterface,
};
use mockall::predicate::*;

struct TestFixture {
    ui: Arc<TestUserInterface>,
    file_system: MockFileSystemMock,
    command_executor: MockCommandExecutorMock,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            file_system: MockFileSystemMock::new(),
            command_executor: MockCommandExecutorMock::new(),
        }
    }

    fn mock_ftl_toml_exists(&mut self, exists: bool) {
        self.file_system
            .expect_exists()
            .with(eq(Path::new("ftl.toml")))
            .times(1)
            .returning(move |_| exists);
    }

    fn mock_ftl_toml_with_registry(&mut self, registry: &'static str) {
        self.file_system
            .expect_exists()
            .with(eq(Path::new("ftl.toml")))
            .times(1)
            .returning(|_| true);

        self.file_system
            .expect_read_to_string()
            .with(eq(Path::new("ftl.toml")))
            .times(1)
            .returning(move |_| {
                Ok(format!(
                    r#"
[project]
name = "test-project"
default_registry = "{registry}"
"#
                ))
            });
    }

    fn mock_wasm_exists(&mut self, path: &'static Path, exists: bool) {
        self.file_system
            .expect_exists()
            .with(eq(path))
            .times(1)
            .returning(move |_| exists);
    }

    fn mock_wkg_push_success(&mut self) {
        self.command_executor
            .expect_execute()
            .withf(|cmd, args| cmd == "wkg" && args[0] == "oci" && args[1] == "push")
            .times(1)
            .returning(|_, _| {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Component pushed successfully".to_vec(),
                    stderr: Vec::new(),
                })
            });
    }

    fn mock_wkg_push_failure(&mut self) {
        self.command_executor
            .expect_execute()
            .withf(|cmd, args| cmd == "wkg" && args[0] == "oci" && args[1] == "push")
            .times(1)
            .returning(|_, _| {
                Ok(CommandOutput {
                    success: false,
                    stdout: Vec::new(),
                    stderr: b"Authentication failed".to_vec(),
                })
            });
    }

    fn mock_wkg_pull_success(&mut self) {
        self.command_executor
            .expect_execute()
            .withf(|cmd, args| cmd == "wkg" && args[0] == "oci" && args[1] == "pull")
            .times(1)
            .returning(|_, _| {
                Ok(CommandOutput {
                    success: true,
                    stdout: b"Component pulled successfully".to_vec(),
                    stderr: Vec::new(),
                })
            });
    }

    fn mock_crane_list_success(&mut self, tags: &[&'static str]) {
        let tags_string = tags.join("\n");
        self.command_executor
            .expect_execute()
            .withf(|cmd, args| cmd == "crane" && args[0] == "ls")
            .times(1)
            .returning(move |_, _| {
                Ok(CommandOutput {
                    success: true,
                    stdout: tags_string.clone().into_bytes(),
                    stderr: Vec::new(),
                })
            });
    }

    fn mock_crane_manifest_exists(&mut self, exists: bool) {
        self.command_executor
            .expect_execute()
            .withf(|cmd, args| cmd == "crane" && args[0] == "manifest")
            .times(1)
            .returning(move |_, _| {
                Ok(CommandOutput {
                    success: exists,
                    stdout: if exists {
                        b"{\"config\": {}}".to_vec()
                    } else {
                        Vec::new()
                    },
                    stderr: if exists {
                        Vec::new()
                    } else {
                        b"MANIFEST_UNKNOWN".to_vec()
                    },
                })
            });
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<ComponentDependencies> {
        Arc::new(ComponentDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
        })
    }
}

#[tokio::test]
async fn test_publish_with_wasm_file_success() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    // Mock ftl.toml with default registry
    fixture.mock_ftl_toml_with_registry("ghcr.io/myorg");

    // Mock WASM file exists
    let wasm_path = Path::new("my-component.wasm");
    fixture.mock_wasm_exists(wasm_path, true);

    // Mock successful push
    fixture.mock_wkg_push_success();

    // The TestUserInterface will return "test-value" for prompts
    // Since we check for "y", we need to skip confirmation

    let deps = fixture.to_deps();

    let result = publish_with_deps(
        &deps,
        wasm_path,
        None,
        Some("my-component"),
        Some("1.0.0"),
        true, // Skip confirmation since TestUserInterface returns "test-value" not "y"
    )
    .await;

    assert!(result.is_ok());

    // Check output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Publishing component")));
    // When yes=true, we skip the confirmation details
    assert!(
        output
            .iter()
            .any(|s| s.contains("✓ Component published successfully"))
    );
}

#[tokio::test]
async fn test_publish_cancelled_by_user() {
    // Create a custom test UI that returns false for confirmation
    struct CancelTestUI {
        base: TestUserInterface,
    }

    impl UserInterface for CancelTestUI {
        fn print(&self, message: &str) {
            self.base.print(message);
        }

        fn print_styled(&self, message: &str, style: MessageStyle) {
            self.base.print_styled(message, style);
        }

        fn prompt_input(&self, prompt: &str, default: Option<&str>) -> Result<String> {
            self.base.prompt_input(prompt, default)
        }

        fn prompt_select(&self, prompt: &str, items: &[&str], default: usize) -> Result<usize> {
            self.base.prompt_select(prompt, items, default)
        }

        fn prompt_confirm(&self, _prompt: &str, _default: bool) -> Result<bool> {
            // Return false to cancel
            Ok(false)
        }

        fn clear_screen(&self) {
            self.base.clear_screen();
        }

        fn create_spinner(&self) -> Box<dyn ProgressIndicator> {
            self.base.create_spinner()
        }

        fn create_multi_progress(&self) -> Box<dyn MultiProgressManager> {
            self.base.create_multi_progress()
        }

        fn is_interactive(&self) -> bool {
            self.base.is_interactive()
        }
    }

    let test_ui = TestUserInterface::new();
    let cancel_ui = Arc::new(CancelTestUI { base: test_ui });
    let mut fixture = TestFixture::new();

    // Mock ftl.toml with default registry
    fixture.mock_ftl_toml_with_registry("ghcr.io/myorg");

    // Don't need to mock WASM file exists - user cancels before that check
    let wasm_path = Path::new("my-component.wasm");

    let deps = Arc::new(ComponentDependencies {
        ui: cancel_ui.clone() as Arc<dyn UserInterface>,
        file_system: Arc::new(fixture.file_system) as Arc<dyn FileSystem>,
        command_executor: Arc::new(fixture.command_executor) as Arc<dyn CommandExecutor>,
    });

    let result = publish_with_deps(
        &deps,
        wasm_path,
        None,
        Some("my-component"),
        Some("1.0.0"),
        false,
    )
    .await;

    assert!(result.is_ok());

    let output = cancel_ui.base.get_output();
    assert!(output.iter().any(|s| s.contains("Publish cancelled")));
}

#[tokio::test]
async fn test_publish_with_yes_flag() {
    let mut fixture = TestFixture::new();
    let _ui = fixture.ui.clone();

    // Mock ftl.toml with default registry
    fixture.mock_ftl_toml_with_registry("ghcr.io/myorg");

    // Mock WASM file exists
    let wasm_path = Path::new("my-component.wasm");
    fixture.mock_wasm_exists(wasm_path, true);

    // Mock successful push
    fixture.mock_wkg_push_success();

    let deps = fixture.to_deps();

    let result = publish_with_deps(
        &deps,
        wasm_path,
        None,
        Some("my-component"),
        Some("1.0.0"),
        true, // yes flag
    )
    .await;

    assert!(result.is_ok());

    // With yes flag, no prompt should have been shown
}

#[tokio::test]
async fn test_publish_push_failure() {
    let mut fixture = TestFixture::new();

    // Mock ftl.toml with default registry
    fixture.mock_ftl_toml_with_registry("ghcr.io/myorg");

    // Mock WASM file exists
    let wasm_path = Path::new("my-component.wasm");
    fixture.mock_wasm_exists(wasm_path, true);

    // Mock push failure
    fixture.mock_wkg_push_failure();

    let deps = fixture.to_deps();

    let result = publish_with_deps(
        &deps,
        wasm_path,
        None,
        Some("my-component"),
        Some("1.0.0"),
        true,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Authentication failed")
    );
}

#[tokio::test]
async fn test_publish_no_default_registry() {
    let mut fixture = TestFixture::new();

    // Mock no ftl.toml
    fixture.mock_ftl_toml_exists(false);

    // Don't mock WASM file exists - the function fails before checking it
    let wasm_path = Path::new("my-component.wasm");

    let deps = fixture.to_deps();

    // Should fail without registry
    let result = publish_with_deps(
        &deps,
        wasm_path,
        None, // No registry override
        Some("my-component"),
        Some("1.0.0"),
        true,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No registry specified")
    );
}

#[tokio::test]
async fn test_publish_with_registry_override() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    // No need to mock ftl.toml - registry override bypasses that check

    // Mock WASM file exists
    let wasm_path = Path::new("my-component.wasm");
    fixture.mock_wasm_exists(wasm_path, true);

    // Mock successful push
    fixture.mock_wkg_push_success();

    let deps = fixture.to_deps();

    let result = publish_with_deps(
        &deps,
        wasm_path,
        Some("docker.io/myuser"), // Registry override
        Some("my-component"),
        Some("2.0.0"),
        true,
    )
    .await;

    assert!(result.is_ok());

    // Check that override was used
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("docker.io/myuser")));
}

#[tokio::test]
async fn test_pull_success() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    // Mock ftl.toml with default registry
    fixture.mock_ftl_toml_with_registry("ghcr.io/myorg");

    // Mock output file doesn't exist
    let output_path = Path::new("my-component.wasm");
    fixture
        .file_system
        .expect_exists()
        .with(eq(output_path))
        .times(1)
        .returning(|_| false);

    // Mock successful pull
    fixture.mock_wkg_pull_success();

    let deps = fixture.to_deps();

    let result = pull_with_deps(&deps, "my-component:1.0.0", Some(output_path), false).await;

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Pulling component from registry"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("✓ Component pulled successfully"))
    );
}

#[tokio::test]
async fn test_pull_with_existing_file_no_force() {
    let mut fixture = TestFixture::new();

    // Mock ftl.toml
    fixture.mock_ftl_toml_exists(false);

    // Mock output file exists
    let output_path = Path::new("output.wasm");
    fixture
        .file_system
        .expect_exists()
        .with(eq(output_path))
        .times(1)
        .returning(|_| true);

    let deps = fixture.to_deps();

    let result = pull_with_deps(
        &deps,
        "ghcr.io/org/comp:1.0.0",
        Some(output_path),
        false, // No force
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_list_versions() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    // Mock ftl.toml with default registry
    fixture.mock_ftl_toml_with_registry("ghcr.io/myorg");

    // Mock crane list success
    fixture.mock_crane_list_success(&["1.0.0", "1.1.0", "2.0.0", "latest"]);

    let deps = fixture.to_deps();

    let result = list_with_deps(&deps, "my-component", None).await;

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Listing component versions"))
    );
    assert!(output.iter().any(|s| s.contains("1.0.0")));
    assert!(output.iter().any(|s| s.contains("2.0.0")));
    assert!(output.iter().any(|s| s.contains("Total: 4 versions")));
}

#[tokio::test]
async fn test_inspect_component_exists() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    // Mock ftl.toml
    fixture.mock_ftl_toml_with_registry("ghcr.io/myorg");

    // Mock component exists
    fixture.mock_crane_manifest_exists(true);

    let deps = fixture.to_deps();

    let result = inspect_with_deps(&deps, "my-component:1.0.0").await;

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Inspecting component")));
    assert!(output.iter().any(|s| s.contains("✓ Component exists")));
    assert!(output.iter().any(|s| s.contains("ftl component pull")));
}

#[tokio::test]
async fn test_inspect_component_not_found() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    // Mock ftl.toml
    fixture.mock_ftl_toml_with_registry("ghcr.io/myorg");

    // Mock component doesn't exist
    fixture.mock_crane_manifest_exists(false);

    // Mock listing available versions (extract_repository will be called)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd, args| cmd == "crane" && args[0] == "ls")
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"1.0.0\n1.1.0\nlatest".to_vec(),
                stderr: Vec::new(),
            })
        });

    let deps = fixture.to_deps();

    let result = inspect_with_deps(&deps, "my-component:999.0.0").await;

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("✗ Component not found")));
    assert!(output.iter().any(|s| s.contains("Available versions")));
    assert!(output.iter().any(|s| s.contains("1.0.0")));
}
