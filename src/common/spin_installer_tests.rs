//! Unit tests for the spin installer

use std::sync::Arc;

use crate::common::spin_installer::*;
use crate::deps::*;
use crate::test_helpers::*;
use crate::ui::TestUserInterface;

use mockall::predicate::*;

// Helper to setup command exists expectation
fn expect_command_exists(mock: &mut MockCommandExecutorMock, command: &str, should_succeed: bool) {
    let expected_command = command.to_string();
    mock.expect_check_command_exists()
        .times(1)
        .returning(move |cmd| {
            if cmd == expected_command {
                if should_succeed {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Command not found"))
                }
            } else {
                Ok(())
            }
        });
}

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

    fn to_deps(self) -> Arc<SpinInstallerDependencies> {
        Arc::new(SpinInstallerDependencies {
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            ui: self.ui as Arc<dyn UserInterface>,
        })
    }
}

#[tokio::test]
async fn test_check_and_install_spin_found_with_plugin() {
    let mut fixture = TestFixture::new();

    // Mock: spin exists in PATH
    expect_command_exists(&mut fixture.command_executor, "spin", true);

    // Mock: check for akamai plugin (already installed)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == &["plugin", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"aka 0.1.0\nother-plugin 0.2.0".to_vec(),
                stderr: vec![],
            })
        });

    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    let result = installer.check_and_install().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "spin");
}

#[tokio::test]
async fn test_check_and_install_spin_found_without_plugin() {
    let mut fixture = TestFixture::new();

    // Mock: spin exists in PATH
    expect_command_exists(&mut fixture.command_executor, "spin", true);

    // Mock: check for akamai plugin (not installed)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == &["plugin", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"other-plugin 0.2.0".to_vec(),
                stderr: vec![],
            })
        });

    // Mock: install akamai plugin
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == &["plugin", "install", "aka"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Plugin installed successfully".to_vec(),
                stderr: vec![],
            })
        });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    let result = installer.check_and_install().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "spin");

    // Verify plugin installation message
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Installing Akamai plugin for Spin"))
    );
}

#[tokio::test]
async fn test_check_and_install_spin_not_found() {
    let mut fixture = TestFixture::new();

    // Mock: spin doesn't exist in PATH
    expect_command_exists(&mut fixture.command_executor, "spin", false);

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    let result = installer.check_and_install().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Spin not found"));

    // Verify warning messages
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Please install Spin from"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("https://github.com/fermyon/spin"))
    );
}

#[tokio::test]
async fn test_check_and_install_plugin_list_fails() {
    let mut fixture = TestFixture::new();

    // Mock: spin exists in PATH
    expect_command_exists(&mut fixture.command_executor, "spin", true);

    // Mock: check for akamai plugin fails
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == &["plugin", "list"])
        .times(1)
        .returning(|_, _| Err(anyhow::anyhow!("Failed to execute command")));

    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    let result = installer.check_and_install().await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to list Spin plugins")
    );
}

#[tokio::test]
async fn test_check_and_install_plugin_install_fails() {
    let mut fixture = TestFixture::new();

    // Mock: spin exists in PATH
    expect_command_exists(&mut fixture.command_executor, "spin", true);

    // Mock: check for akamai plugin (not installed)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == &["plugin", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"other-plugin 0.2.0".to_vec(),
                stderr: vec![],
            })
        });

    // Mock: install akamai plugin fails
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == &["plugin", "install", "aka"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: false,
                stdout: vec![],
                stderr: b"Error: Plugin installation failed".to_vec(),
            })
        });

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    let result = installer.check_and_install().await;
    // Should still succeed but with warning
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "spin");

    // Verify warning message
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Failed to install Akamai plugin"))
    );
    assert!(output.iter().any(|s| s.contains("spin plugin install aka")));
}

#[tokio::test]
async fn test_check_and_install_plugin_list_command_fails() {
    let mut fixture = TestFixture::new();

    // Mock: spin exists in PATH
    expect_command_exists(&mut fixture.command_executor, "spin", true);

    // Mock: plugin list command returns failure status
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == &["plugin", "list"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: false,
                stdout: vec![],
                stderr: b"Failed to list plugins".to_vec(),
            })
        });

    // Mock: install akamai plugin (since we couldn't check if it exists)
    fixture
        .command_executor
        .expect_execute()
        .withf(|cmd: &str, args: &[&str]| cmd == "spin" && args == &["plugin", "install", "aka"])
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Plugin installed successfully".to_vec(),
                stderr: vec![],
            })
        });

    let deps = fixture.to_deps();
    let installer = RealSpinInstallerV2::new(deps);

    let result = installer.check_and_install().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "spin");
}
