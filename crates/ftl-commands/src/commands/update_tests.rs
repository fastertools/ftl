//! Unit tests for the update command

use std::sync::Arc;

use crate::commands::update::{
    CommandExecutor, CommandOutput, Environment, HttpClient, UpdateDependencies, execute_with_deps,
};
use ftl_core::deps::UserInterface;
use ftl_common::ui::TestUserInterface;

// Mock implementation of HttpClient
struct MockHttpClient {
    response: Option<String>,
    should_fail: bool,
}

impl MockHttpClient {
    fn new() -> Self {
        Self {
            response: Some(r#"{"crate": {"newest_version": "0.2.0"}}"#.to_string()),
            should_fail: false,
        }
    }

    fn with_response(mut self, response: String) -> Self {
        self.response = Some(response);
        self
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, _url: &str, _user_agent: &str) -> Result<String, anyhow::Error> {
        if self.should_fail {
            Err(anyhow::anyhow!("Network error"))
        } else {
            Ok(self
                .response
                .clone()
                .unwrap_or_else(|| r#"{"crate": {"newest_version": "0.2.0"}}"#.to_string()))
        }
    }
}

// Mock implementation of CommandExecutor
struct MockCommandExecutor {
    expected_commands: Vec<(String, Vec<String>, CommandOutput)>,
    call_count: std::sync::Mutex<usize>,
}

impl MockCommandExecutor {
    fn new() -> Self {
        Self {
            expected_commands: Vec::new(),
            call_count: std::sync::Mutex::new(0),
        }
    }

    fn expect_execute(mut self, command: &str, args: &[&str], output: CommandOutput) -> Self {
        self.expected_commands.push((
            command.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
            output,
        ));
        self
    }
}

impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput, anyhow::Error> {
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

        Ok(CommandOutput {
            success: output.success,
            stderr: output.stderr.clone(),
        })
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

    fn with_version(mut self, version: &'static str) -> Self {
        self.cargo_pkg_version = version;
        self
    }
}

impl Environment for MockEnvironment {
    fn get_cargo_pkg_version(&self) -> &'static str {
        self.cargo_pkg_version
    }
}

struct TestFixture {
    ui: Arc<TestUserInterface>,
    http_client: Arc<MockHttpClient>,
    command_executor: Arc<MockCommandExecutor>,
    environment: Arc<MockEnvironment>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            http_client: Arc::new(MockHttpClient::new()),
            command_executor: Arc::new(MockCommandExecutor::new()),
            environment: Arc::new(MockEnvironment::new()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<UpdateDependencies> {
        Arc::new(UpdateDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            http_client: self.http_client as Arc<dyn HttpClient>,
            command_executor: self.command_executor as Arc<dyn CommandExecutor>,
            environment: self.environment as Arc<dyn Environment>,
        })
    }
}

#[tokio::test]
async fn test_update_already_on_latest() {
    let mut fixture = TestFixture::new();
    fixture.environment = Arc::new(MockEnvironment::new().with_version("0.2.0"));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(false, deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Updating FTL CLI")));
    assert!(output.iter().any(|s| s.contains("Current version: 0.2.0")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Already on latest version"))
    );
}

#[tokio::test]
async fn test_update_new_version_available() {
    let mut fixture = TestFixture::new();

    // Mock: cargo install succeeds
    fixture.command_executor = Arc::new(MockCommandExecutor::new().expect_execute(
        "cargo",
        &["install", "ftl-cli", "--force"],
        CommandOutput {
            success: true,
            stderr: Vec::new(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(false, deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Current version: 0.1.0")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Latest version available: 0.2.0"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Installing latest version"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("FTL CLI updated successfully"))
    );
}

#[tokio::test]
async fn test_update_force_flag() {
    let mut fixture = TestFixture::new();
    fixture.environment = Arc::new(MockEnvironment::new().with_version("0.2.0"));

    // Mock: cargo install succeeds
    fixture.command_executor = Arc::new(MockCommandExecutor::new().expect_execute(
        "cargo",
        &["install", "ftl-cli", "--force"],
        CommandOutput {
            success: true,
            stderr: Vec::new(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // With force=true, should update even if on latest version
    let result = execute_with_deps(true, deps).await;
    assert!(result.is_ok());

    // Verify output - should NOT check version
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Installing latest version"))
    );
    assert!(
        !output
            .iter()
            .any(|s| s.contains("Already on latest version"))
    );
}

#[tokio::test]
async fn test_update_version_check_fails() {
    let mut fixture = TestFixture::new();
    fixture.http_client = Arc::new(MockHttpClient::new().with_failure());

    // Mock: cargo install succeeds
    fixture.command_executor = Arc::new(MockCommandExecutor::new().expect_execute(
        "cargo",
        &["install", "ftl-cli", "--force"],
        CommandOutput {
            success: true,
            stderr: Vec::new(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(false, deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Could not check for latest version"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Installing latest version"))
    );
}

#[tokio::test]
async fn test_update_install_fails() {
    let mut fixture = TestFixture::new();

    // Mock: cargo install fails
    fixture.command_executor = Arc::new(MockCommandExecutor::new().expect_execute(
        "cargo",
        &["install", "ftl-cli", "--force"],
        CommandOutput {
            success: false,
            stderr: b"error: failed to compile ftl-cli".to_vec(),
        },
    ));

    let deps = fixture.to_deps();

    let result = execute_with_deps(false, deps).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to update FTL CLI")
    );
}

#[tokio::test]
async fn test_update_invalid_version_response() {
    let mut fixture = TestFixture::new();
    fixture.http_client =
        Arc::new(MockHttpClient::new().with_response(r#"{"invalid": "json"}"#.to_string()));

    // Mock: cargo install succeeds
    fixture.command_executor = Arc::new(MockCommandExecutor::new().expect_execute(
        "cargo",
        &["install", "ftl-cli", "--force"],
        CommandOutput {
            success: true,
            stderr: Vec::new(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(false, deps).await;
    assert!(result.is_ok());

    // Verify output - should proceed with update when version check fails
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Could not check for latest version"))
    );
}

#[tokio::test]
async fn test_update_newer_current_version() {
    let mut fixture = TestFixture::new();
    fixture.environment = Arc::new(MockEnvironment::new().with_version("0.3.0"));
    fixture.http_client = Arc::new(
        MockHttpClient::new()
            .with_response(r#"{"crate": {"newest_version": "0.2.0"}}"#.to_string()),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(false, deps).await;
    assert!(result.is_ok());

    // Verify output - should say already on latest when current > remote
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Already on latest version"))
    );
}

#[tokio::test]
async fn test_update_prerelease_version() {
    let mut fixture = TestFixture::new();
    fixture.environment = Arc::new(MockEnvironment::new().with_version("0.2.0-beta.1"));
    fixture.http_client = Arc::new(
        MockHttpClient::new()
            .with_response(r#"{"crate": {"newest_version": "0.2.0"}}"#.to_string()),
    );

    // Mock: cargo install succeeds
    fixture.command_executor = Arc::new(MockCommandExecutor::new().expect_execute(
        "cargo",
        &["install", "ftl-cli", "--force"],
        CommandOutput {
            success: true,
            stderr: Vec::new(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(false, deps).await;
    assert!(result.is_ok());

    // Verify output - prerelease should update to stable
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Latest version available: 0.2.0"))
    );
}

#[tokio::test]
async fn test_update_output_completeness() {
    let mut fixture = TestFixture::new();

    // Mock: cargo install succeeds
    fixture.command_executor = Arc::new(MockCommandExecutor::new().expect_execute(
        "cargo",
        &["install", "ftl-cli", "--force"],
        CommandOutput {
            success: true,
            stderr: Vec::new(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(false, deps).await;
    assert!(result.is_ok());

    // Verify all expected output lines
    let output = ui.get_output();
    let expected_patterns = [
        "→ Updating FTL CLI",
        "Current version: 0.1.0",
        "Latest version available: 0.2.0",
        "→ Installing latest version...",
        "✓ FTL CLI updated successfully!",
        "Run 'ftl --version' to verify the new version",
    ];

    for pattern in &expected_patterns {
        assert!(
            output.iter().any(|s| s.contains(pattern)),
            "Expected to find '{pattern}' in output"
        );
    }
}
