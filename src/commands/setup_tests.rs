//! Unit tests for the setup command

use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{ExitStatus, Output};
use std::sync::Arc;

use crate::commands::setup::{
    Environment, SetupCommandExecutor, SetupDependencies, SpinInstaller, info_with_deps,
    templates_with_deps,
};
use crate::deps::UserInterface;
use crate::ui::TestUserInterface;

// Mock implementation of SpinInstaller
struct MockSpinInstaller {
    should_fail: bool,
}

impl MockSpinInstaller {
    fn new() -> Self {
        Self { should_fail: false }
    }

    fn with_error() -> Self {
        Self { should_fail: true }
    }
}

impl SpinInstaller for MockSpinInstaller {
    fn check_and_install(&self) -> Result<PathBuf, anyhow::Error> {
        if self.should_fail {
            Err(anyhow::anyhow!("Spin not found"))
        } else {
            Ok(PathBuf::from("/usr/local/bin/spin"))
        }
    }

    fn get_spin_path(&self) -> Result<PathBuf, anyhow::Error> {
        if self.should_fail {
            Err(anyhow::anyhow!("Spin not found"))
        } else {
            Ok(PathBuf::from("/usr/local/bin/spin"))
        }
    }
}

// Mock implementation of SetupCommandExecutor
struct MockSetupCommandExecutor {
    expected_commands: Vec<(String, Vec<String>, Output)>,
    call_count: std::sync::Mutex<usize>,
}

impl MockSetupCommandExecutor {
    fn new() -> Self {
        Self {
            expected_commands: Vec::new(),
            call_count: std::sync::Mutex::new(0),
        }
    }

    fn expect_execute(mut self, command: &str, args: &[&str], output: Output) -> Self {
        self.expected_commands.push((
            command.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
            output,
        ));
        self
    }

    fn successful_output(stdout: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(0),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    fn failed_output(stderr: &str) -> Output {
        Output {
            status: ExitStatus::from_raw(256), // Exit code 1
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }
}

impl SetupCommandExecutor for MockSetupCommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<Output, anyhow::Error> {
        let mut count = self.call_count.lock().unwrap();
        let index = *count;
        *count += 1;

        if index >= self.expected_commands.len() {
            return Err(anyhow::anyhow!("Unexpected command execution"));
        }

        let (expected_cmd, expected_args, output) = &self.expected_commands[index];

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

        Ok(output.clone())
    }
}

// Mock implementation of Environment
struct MockEnvironment {
    cargo_pkg_version: &'static str,
}

impl MockEnvironment {
    fn new() -> Self {
        Self {
            cargo_pkg_version: "0.1.0",
        }
    }
}

impl Environment for MockEnvironment {
    fn get_cargo_pkg_version(&self) -> &'static str {
        self.cargo_pkg_version
    }
}

struct TestFixture {
    ui: Arc<TestUserInterface>,
    spin_installer: Arc<MockSpinInstaller>,
    command_executor: Arc<MockSetupCommandExecutor>,
    environment: Arc<MockEnvironment>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            spin_installer: Arc::new(MockSpinInstaller::new()),
            command_executor: Arc::new(MockSetupCommandExecutor::new()),
            environment: Arc::new(MockEnvironment::new()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<SetupDependencies> {
        Arc::new(SetupDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            spin_installer: self.spin_installer as Arc<dyn SpinInstaller>,
            command_executor: self.command_executor as Arc<dyn SetupCommandExecutor>,
            environment: self.environment as Arc<dyn Environment>,
        })
    }
}

#[tokio::test]
async fn test_templates_already_installed() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(MockSetupCommandExecutor::new().expect_execute(
        "/usr/local/bin/spin",
        &["templates", "list"],
        MockSetupCommandExecutor::successful_output(
            "ftl-mcp-server 0.1.0 [installed]\nftl-mcp-rust 0.1.0 [installed]",
        ),
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = templates_with_deps(false, None, None, None, None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Managing FTL templates")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("ftl-mcp templates are already installed"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Use --force to reinstall"))
    );
}

#[tokio::test]
async fn test_templates_install_default() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output(""),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &[
                    "templates",
                    "install",
                    "--git",
                    "https://github.com/fastertools/ftl-mcp",
                    "--upgrade",
                ],
                MockSetupCommandExecutor::successful_output("Templates installed"),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output(
                    "ftl-mcp-server 0.1.0 [installed]\nftl-mcp-rust 0.1.0 [installed]",
                ),
            ),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = templates_with_deps(false, None, None, None, None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| {
        s.contains("Installing ftl-mcp templates from https://github.com/fastertools/ftl-mcp")
    }));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Templates installed successfully!"))
    );
    assert!(output.iter().any(|s| s.contains("ftl-mcp-server")));
}

#[tokio::test]
async fn test_templates_install_from_git() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output(""),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &[
                    "templates",
                    "install",
                    "--git",
                    "https://github.com/user/repo",
                    "--upgrade",
                ],
                MockSetupCommandExecutor::successful_output("Templates installed"),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output("ftl-mcp-custom 0.1.0 [installed]"),
            ),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = templates_with_deps(
        false,
        Some("https://github.com/user/repo"),
        None,
        None,
        None,
        &deps,
    );
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Installing templates from Git: https://github.com/user/repo"))
    );
}

#[tokio::test]
async fn test_templates_install_from_git_with_branch() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output(""),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &[
                    "templates",
                    "install",
                    "--git",
                    "https://github.com/user/repo",
                    "--branch",
                    "dev",
                    "--upgrade",
                ],
                MockSetupCommandExecutor::successful_output("Templates installed"),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output("ftl-mcp-custom 0.1.0 [installed]"),
            ),
    );

    let deps = fixture.to_deps();

    let result = templates_with_deps(
        false,
        Some("https://github.com/user/repo"),
        Some("dev"),
        None,
        None,
        &deps,
    );
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_templates_install_from_dir() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output(""),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &[
                    "templates",
                    "install",
                    "--dir",
                    "/path/to/templates",
                    "--upgrade",
                ],
                MockSetupCommandExecutor::successful_output("Templates installed"),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output("ftl-mcp-local 0.1.0 [installed]"),
            ),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let path = PathBuf::from("/path/to/templates");
    let result = templates_with_deps(false, None, None, Some(&path), None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Installing templates from directory: /path/to/templates"))
    );
}

#[tokio::test]
async fn test_templates_install_from_tar() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output(""),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &[
                    "templates",
                    "install",
                    "--tar",
                    "templates.tar.gz",
                    "--upgrade",
                ],
                MockSetupCommandExecutor::successful_output("Templates installed"),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output("ftl-mcp-archive 0.1.0 [installed]"),
            ),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = templates_with_deps(false, None, None, None, Some("templates.tar.gz"), &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Installing templates from tarball: templates.tar.gz"))
    );
}

#[tokio::test]
async fn test_templates_force_reinstall() {
    let mut fixture = TestFixture::new();

    // With force=true, it skips the check and proceeds directly to install
    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &[
                    "templates",
                    "install",
                    "--git",
                    "https://github.com/fastertools/ftl-mcp",
                    "--upgrade",
                ],
                MockSetupCommandExecutor::successful_output("Templates installed"),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output("ftl-mcp-server 0.1.0 [installed]"),
            ),
    );

    let deps = fixture.to_deps();

    let result = templates_with_deps(true, None, None, None, None, &deps);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_templates_install_failure() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output(""),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &[
                    "templates",
                    "install",
                    "--git",
                    "https://github.com/fastertools/ftl-mcp",
                    "--upgrade",
                ],
                MockSetupCommandExecutor::failed_output("Failed to clone repository"),
            ),
    );

    let deps = fixture.to_deps();

    let result = templates_with_deps(false, None, None, None, None, &deps);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to install templates")
    );
}

#[tokio::test]
async fn test_info_all_installed() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &["--version"],
                MockSetupCommandExecutor::successful_output("spin 2.0.0"),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output(
                    "ftl-mcp-server 0.1.0 [installed]\nftl-mcp-rust 0.1.0 [installed]",
                ),
            )
            .expect_execute(
                "cargo",
                &["component", "--version"],
                MockSetupCommandExecutor::successful_output("cargo-component 0.5.0"),
            )
            .expect_execute(
                "wkg",
                &["--version"],
                MockSetupCommandExecutor::successful_output("wkg 0.1.0"),
            ),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    info_with_deps(&deps);

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("FTL Configuration")));
    assert!(output.iter().any(|s| s.contains("FTL CLI version: 0.1.0")));
    assert!(output.iter().any(|s| s.contains("Spin: ✓")));
    assert!(output.iter().any(|s| s.contains("spin 2.0.0")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("ftl-mcp Templates: ✓ Installed"))
    );
    assert!(output.iter().any(|s| s.contains("cargo-component: ✓")));
    assert!(output.iter().any(|s| s.contains("wkg: ✓")));
}

#[tokio::test]
async fn test_info_spin_not_installed() {
    let mut fixture = TestFixture::new();

    fixture.spin_installer = Arc::new(MockSpinInstaller::with_error());

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "cargo",
                &["component", "--version"],
                MockSetupCommandExecutor::successful_output("cargo-component 0.5.0"),
            )
            .expect_execute(
                "wkg",
                &["--version"],
                MockSetupCommandExecutor::successful_output("wkg 0.1.0"),
            ),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    info_with_deps(&deps);

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Spin: ✗ Not installed")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Run 'ftl setup templates' to install"))
    );
}

#[tokio::test]
async fn test_info_no_templates() {
    let mut fixture = TestFixture::new();

    fixture.command_executor = Arc::new(
        MockSetupCommandExecutor::new()
            .expect_execute(
                "/usr/local/bin/spin",
                &["--version"],
                MockSetupCommandExecutor::successful_output("spin 2.0.0"),
            )
            .expect_execute(
                "/usr/local/bin/spin",
                &["templates", "list"],
                MockSetupCommandExecutor::successful_output("other-template 1.0.0 [installed]"),
            )
            .expect_execute(
                "cargo",
                &["component", "--version"],
                MockSetupCommandExecutor::failed_output("command not found"),
            )
            .expect_execute(
                "wkg",
                &["--version"],
                MockSetupCommandExecutor::failed_output("command not found"),
            ),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    info_with_deps(&deps);

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("ftl-mcp Templates: ✗ Not installed"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("cargo-component: ✗ Not installed"))
    );
    assert!(output.iter().any(|s| s.contains("wkg: ✗ Not installed")));
}
