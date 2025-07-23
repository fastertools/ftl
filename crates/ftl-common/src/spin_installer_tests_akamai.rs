//! Additional tests for `ensure_akamai_plugin` functionality

use crate::common::spin_installer::*;
use crate::deps::*;
use crate::test_helpers::*;
use crate::ui::TestUserInterface;
use mockall::predicate::*;
use std::sync::Arc;

struct TestFixture {
    command_executor: MockCommandExecutorMock,
    ui: Arc<TestUserInterface>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            command_executor: MockCommandExecutorMock::new(),
            ui: Arc::new(TestUserInterface::new()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<SpinInstallerDependencies> {
        Arc::new(SpinInstallerDependencies {
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            ui: self.ui as Arc<dyn UserInterface>,
        })
    }
}

#[tokio::test]
async fn test_ensure_akamai_plugin_already_installed() {
    let mut fixture = TestFixture::new();

    // Mock: check command exists
    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("spin"))
        .times(1)
        .returning(|_| Ok(()));

    // Mock: plugin list shows aka is already installed
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == ["plugin", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"aka (0.1.0)\ncloud (0.2.0)".to_vec(),
                stderr: vec![],
            })
        });

    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    // Call check_and_install which internally calls ensure_akamai_plugin
    let result = installer.check_and_install().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ensure_akamai_plugin_needs_install_success() {
    let mut fixture = TestFixture::new();

    // Mock: check command exists
    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("spin"))
        .times(1)
        .returning(|_| Ok(()));

    // Mock: plugin list doesn't show aka
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == ["plugin", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"cloud (0.2.0)\nother-plugin (1.0.0)".to_vec(),
                stderr: vec![],
            })
        });

    // Mock: install aka plugin succeeds
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == ["plugin", "install", "aka"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Plugin 'aka' installed successfully".to_vec(),
                stderr: vec![],
            })
        });

    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    let result = installer.check_and_install().await;
    assert!(result.is_ok());
    // The test verifies that when aka plugin is not installed, it attempts to install it
}

#[tokio::test]
async fn test_ensure_akamai_plugin_install_fails() {
    let mut fixture = TestFixture::new();

    // Mock: check command exists
    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("spin"))
        .times(1)
        .returning(|_| Ok(()));

    // Mock: plugin list doesn't show aka
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == ["plugin", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"cloud (0.2.0)".to_vec(),
                stderr: vec![],
            })
        });

    // Mock: install aka plugin fails
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == ["plugin", "install", "aka"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: false,
                stdout: vec![],
                stderr: b"Failed to install plugin: network error".to_vec(),
            })
        });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    // Should still succeed even if plugin install fails
    let result = installer.check_and_install().await;
    assert!(result.is_ok());

    // Verify warning was shown
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Warning: Failed to install Akamai plugin"))
    );
    assert!(output.iter().any(|s| s.contains("spin plugin install aka")));
}

#[tokio::test]
async fn test_ensure_akamai_plugin_list_fails() {
    let mut fixture = TestFixture::new();

    // Mock: check command exists
    fixture
        .command_executor
        .expect_check_command_exists()
        .with(eq("spin"))
        .times(1)
        .returning(|_| Ok(()));

    // Mock: plugin list fails (returns success=false)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == ["plugin", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: false,
                stdout: vec![],
                stderr: b"Failed to list plugins".to_vec(),
            })
        });

    // Mock: install aka plugin is still attempted
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == ["plugin", "install", "aka"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Plugin installed".to_vec(),
                stderr: vec![],
            })
        });

    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    let result = installer.check_and_install().await;
    assert!(result.is_ok());
}
