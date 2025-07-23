//! Unit tests for the test command

use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Output};
use std::sync::Arc;

use crate::commands::test::{
    DirectoryReader, FileChecker, TestCommandExecutor, TestDependencies, execute_with_deps,
};
use ftl_common::ui::TestUserInterface;
use ftl_core::deps::UserInterface;

// Mock implementation of DirectoryReader
struct MockDirectoryReader {
    directory_contents: Vec<PathBuf>,
}

impl MockDirectoryReader {
    fn new() -> Self {
        Self {
            directory_contents: Vec::new(),
        }
    }

    fn with_directory_contents(mut self, contents: Vec<PathBuf>) -> Self {
        self.directory_contents = contents;
        self
    }
}

impl DirectoryReader for MockDirectoryReader {
    fn read_dir(&self, _path: &Path) -> Result<Vec<PathBuf>, anyhow::Error> {
        Ok(self.directory_contents.clone())
    }

    fn is_dir(&self, path: &Path) -> Result<bool, anyhow::Error> {
        // For our tests, anything in directory_contents is a directory
        Ok(self.directory_contents.iter().any(|p| p == path))
    }
}

// Mock implementation of FileChecker
struct MockFileChecker {
    existing_files: Vec<PathBuf>,
}

impl MockFileChecker {
    fn new() -> Self {
        Self {
            existing_files: Vec::new(),
        }
    }

    fn with_existing_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.existing_files.push(path.into());
        self
    }
}

impl FileChecker for MockFileChecker {
    fn exists(&self, path: &Path) -> Result<bool, anyhow::Error> {
        Ok(self.existing_files.iter().any(|p| p == path))
    }
}

// Mock implementation of TestCommandExecutor
struct MockTestCommandExecutor {
    expected_commands: Vec<(String, Vec<String>, Option<String>, Output)>,
    call_count: std::sync::Mutex<usize>,
}

impl MockTestCommandExecutor {
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
        cwd: Option<&str>,
        output: Output,
    ) -> Self {
        self.expected_commands.push((
            command.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
            cwd.map(std::string::ToString::to_string),
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

impl TestCommandExecutor for MockTestCommandExecutor {
    #[allow(clippy::similar_names)]
    fn execute(
        &self,
        command: &str,
        args: &[&str],
        working_dir: Option<&str>,
    ) -> Result<Output, anyhow::Error> {
        let mut count = self.call_count.lock().unwrap();
        let index = *count;
        *count += 1;

        if index >= self.expected_commands.len() {
            return Err(anyhow::anyhow!("Unexpected command execution"));
        }

        let (expected_cmd, expected_args, expected_cwd, expected_output) =
            &self.expected_commands[index];

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

        if working_dir != expected_cwd.as_deref() {
            return Err(anyhow::anyhow!(
                "Expected cwd {:?}, got {:?}",
                expected_cwd,
                working_dir
            ));
        }

        Ok(expected_output.clone())
    }
}

struct TestFixture {
    ui: Arc<TestUserInterface>,
    directory_reader: Arc<MockDirectoryReader>,
    file_checker: Arc<MockFileChecker>,
    command_executor: Arc<MockTestCommandExecutor>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            directory_reader: Arc::new(MockDirectoryReader::new()),
            file_checker: Arc::new(MockFileChecker::new()),
            command_executor: Arc::new(MockTestCommandExecutor::new()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<TestDependencies> {
        Arc::new(TestDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            directory_reader: self.directory_reader as Arc<dyn DirectoryReader>,
            file_checker: self.file_checker as Arc<dyn FileChecker>,
            command_executor: self.command_executor as Arc<dyn TestCommandExecutor>,
        })
    }
}

#[tokio::test]
async fn test_single_rust_tool() {
    let mut fixture = TestFixture::new();

    fixture.file_checker = Arc::new(MockFileChecker::new().with_existing_file("./Cargo.toml"));

    fixture.command_executor = Arc::new(MockTestCommandExecutor::new().expect_execute(
        "cargo",
        &["test"],
        Some("."),
        MockTestCommandExecutor::successful_output("running 10 tests\ntest result: ok. 10 passed"),
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Running tests")));
    assert!(output.iter().any(|s| s.contains("running 10 tests")));
    assert!(output.iter().any(|s| s.contains("All tests passed!")));
}

#[tokio::test]
async fn test_single_npm_tool() {
    let mut fixture = TestFixture::new();

    fixture.file_checker = Arc::new(MockFileChecker::new().with_existing_file("./package.json"));

    fixture.command_executor = Arc::new(MockTestCommandExecutor::new().expect_execute(
        "npm",
        &["test"],
        Some("."),
        MockTestCommandExecutor::successful_output("Test Suites: 5 passed, 5 total"),
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Test Suites: 5 passed")));
    assert!(output.iter().any(|s| s.contains("All tests passed!")));
}

#[tokio::test]
async fn test_single_makefile_tool() {
    let mut fixture = TestFixture::new();

    fixture.file_checker = Arc::new(MockFileChecker::new().with_existing_file("./Makefile"));

    fixture.command_executor = Arc::new(MockTestCommandExecutor::new().expect_execute(
        "make",
        &["test"],
        Some("."),
        MockTestCommandExecutor::successful_output("All tests passing"),
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("All tests passing")));
}

#[tokio::test]
async fn test_project_directory_multiple_tools() {
    let mut fixture = TestFixture::new();

    fixture.file_checker = Arc::new(
        MockFileChecker::new()
            .with_existing_file("./spin.toml")
            .with_existing_file("tool1/Cargo.toml")
            .with_existing_file("tool2/package.json"),
    );

    fixture.directory_reader = Arc::new(
        MockDirectoryReader::new()
            .with_directory_contents(vec![PathBuf::from("tool1"), PathBuf::from("tool2")]),
    );

    fixture.command_executor = Arc::new(
        MockTestCommandExecutor::new()
            .expect_execute(
                "cargo",
                &["test"],
                Some("tool1"),
                MockTestCommandExecutor::successful_output("test result: ok"),
            )
            .expect_execute(
                "npm",
                &["test"],
                Some("tool2"),
                MockTestCommandExecutor::successful_output("Tests passed"),
            ),
    );

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Testing all tools in project"))
    );
    assert!(output.iter().any(|s| s.contains("Testing tool1")));
    assert!(output.iter().any(|s| s.contains("Testing tool2")));
    assert!(output.iter().any(|s| s.contains("All tests passed!")));
}

#[tokio::test]
async fn test_project_directory_no_tools() {
    let mut fixture = TestFixture::new();

    fixture.file_checker = Arc::new(MockFileChecker::new().with_existing_file("./spin.toml"));

    fixture.directory_reader = Arc::new(MockDirectoryReader::new().with_directory_contents(vec![]));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("No tools found to test")));
}

#[tokio::test]
async fn test_no_test_configuration() {
    let fixture = TestFixture::new();
    // No Cargo.toml, package.json, or Makefile

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(None, &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("No test configuration found"))
    );
}

#[tokio::test]
async fn test_cargo_test_failure() {
    let mut fixture = TestFixture::new();

    fixture.file_checker = Arc::new(MockFileChecker::new().with_existing_file("./Cargo.toml"));

    fixture.command_executor = Arc::new(MockTestCommandExecutor::new().expect_execute(
        "cargo",
        &["test"],
        Some("."),
        MockTestCommandExecutor::failed_output("error: test failed"),
    ));

    let deps = fixture.to_deps();

    let result = execute_with_deps(None, &deps);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tests failed"));
}

#[tokio::test]
async fn test_custom_path() {
    let mut fixture = TestFixture::new();

    fixture.file_checker =
        Arc::new(MockFileChecker::new().with_existing_file("custom/path/Cargo.toml"));

    fixture.command_executor = Arc::new(MockTestCommandExecutor::new().expect_execute(
        "cargo",
        &["test"],
        Some("custom/path"),
        MockTestCommandExecutor::successful_output("test result: ok"),
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(Some(PathBuf::from("custom/path")), &deps);
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("All tests passed!")));
}

#[tokio::test]
async fn test_mixed_success_and_failure() {
    let mut fixture = TestFixture::new();

    fixture.file_checker = Arc::new(
        MockFileChecker::new()
            .with_existing_file("./spin.toml")
            .with_existing_file("tool1/Cargo.toml")
            .with_existing_file("tool2/package.json"),
    );

    fixture.directory_reader = Arc::new(
        MockDirectoryReader::new()
            .with_directory_contents(vec![PathBuf::from("tool1"), PathBuf::from("tool2")]),
    );

    fixture.command_executor = Arc::new(
        MockTestCommandExecutor::new()
            .expect_execute(
                "cargo",
                &["test"],
                Some("tool1"),
                MockTestCommandExecutor::successful_output("test result: ok"),
            )
            .expect_execute(
                "npm",
                &["test"],
                Some("tool2"),
                MockTestCommandExecutor::failed_output("npm test failed"),
            ),
    );

    let deps = fixture.to_deps();

    let result = execute_with_deps(None, &deps);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Tests failed"));
}
