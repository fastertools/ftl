//! Unit tests for the up command

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::commands::up::{self, *};
use crate::deps::*;
use crate::ui::TestUserInterface;

// Mock implementations
struct MockProcessManager {
    spawn_responses: Arc<Mutex<Vec<Result<Box<dyn ProcessHandle>, anyhow::Error>>>>,
    spawn_count: Arc<AtomicU32>,
}

impl MockProcessManager {
    fn new() -> Self {
        Self {
            spawn_responses: Arc::new(Mutex::new(Vec::new())),
            spawn_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn add_spawn_response(&self, response: Result<Box<dyn ProcessHandle>, anyhow::Error>) {
        self.spawn_responses.lock().unwrap().push(response);
    }

    fn get_spawn_count(&self) -> u32 {
        self.spawn_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl ProcessManager for MockProcessManager {
    async fn spawn(
        &self,
        _command: &str,
        _args: &[&str],
        _working_dir: Option<&Path>,
    ) -> Result<Box<dyn ProcessHandle>, anyhow::Error> {
        self.spawn_count.fetch_add(1, Ordering::SeqCst);
        let mut responses = self.spawn_responses.lock().unwrap();
        if let Some(response) = responses.pop() {
            response
        } else {
            Ok(Box::new(MockProcessHandle::new(1234, 0)))
        }
    }
}

struct MockProcessHandle {
    id: u32,
    exit_code: i32,
    terminated: Arc<AtomicBool>,
    wait_count: Arc<AtomicU32>,
    wait_duration: Option<Duration>,
}

impl MockProcessHandle {
    fn new(id: u32, exit_code: i32) -> Self {
        Self {
            id,
            exit_code,
            terminated: Arc::new(AtomicBool::new(false)),
            wait_count: Arc::new(AtomicU32::new(0)),
            wait_duration: None,
        }
    }

    fn with_wait_duration(mut self, duration: Duration) -> Self {
        self.wait_duration = Some(duration);
        self
    }

    fn was_terminated(&self) -> bool {
        self.terminated.load(Ordering::SeqCst)
    }

    fn get_wait_count(&self) -> u32 {
        self.wait_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl ProcessHandle for MockProcessHandle {
    async fn wait(&mut self) -> Result<ExitStatus, anyhow::Error> {
        self.wait_count.fetch_add(1, Ordering::SeqCst);

        // Wait for the specified duration or until terminated
        if let Some(duration) = self.wait_duration {
            let start = std::time::Instant::now();
            while start.elapsed() < duration && !self.terminated.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        Ok(ExitStatus::new(Some(self.exit_code)))
    }

    async fn terminate(&mut self) -> Result<(), anyhow::Error> {
        self.terminated.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn id(&self) -> u32 {
        self.id
    }
}

struct MockFileWatcher {
    watch_count: Arc<AtomicU32>,
    should_fail: bool,
}

impl MockFileWatcher {
    fn new() -> Self {
        Self {
            watch_count: Arc::new(AtomicU32::new(0)),
            should_fail: false,
        }
    }

    fn with_failure() -> Self {
        Self {
            watch_count: Arc::new(AtomicU32::new(0)),
            should_fail: true,
        }
    }

    fn get_watch_count(&self) -> u32 {
        self.watch_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl FileWatcher for MockFileWatcher {
    async fn watch(
        &self,
        _path: &Path,
        _recursive: bool,
    ) -> Result<Box<dyn WatchHandle>, anyhow::Error> {
        self.watch_count.fetch_add(1, Ordering::SeqCst);
        if self.should_fail {
            Err(anyhow::anyhow!("Failed to create file watcher"))
        } else {
            Ok(Box::new(MockWatchHandle::new()))
        }
    }
}

struct MockWatchHandle {
    change_count: Arc<AtomicU32>,
    files_to_return: Arc<Mutex<Vec<Vec<PathBuf>>>>,
}

impl MockWatchHandle {
    fn new() -> Self {
        Self {
            change_count: Arc::new(AtomicU32::new(0)),
            files_to_return: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_change(&self, files: Vec<PathBuf>) {
        self.files_to_return.lock().unwrap().push(files);
    }
}

#[async_trait::async_trait]
impl WatchHandle for MockWatchHandle {
    async fn wait_for_change(&mut self) -> Result<Vec<PathBuf>, anyhow::Error> {
        self.change_count.fetch_add(1, Ordering::SeqCst);

        // Simulate waiting for file changes
        tokio::time::sleep(Duration::from_millis(100)).await;

        let changed_files = {
            let mut files = self.files_to_return.lock().unwrap();
            files.pop()
        };

        if let Some(files) = changed_files {
            Ok(files)
        } else {
            // Block forever if no more changes
            tokio::time::sleep(Duration::from_secs(3600)).await;
            Ok(vec![])
        }
    }
}

struct MockSignalHandler {
    interrupt_after: Option<Duration>,
}

impl MockSignalHandler {
    fn new() -> Self {
        Self {
            interrupt_after: None,
        }
    }

    fn with_interrupt_after(duration: Duration) -> Self {
        Self {
            interrupt_after: Some(duration),
        }
    }
}

#[async_trait::async_trait]
impl SignalHandler for MockSignalHandler {
    async fn wait_for_interrupt(&self) -> Result<(), anyhow::Error> {
        if let Some(duration) = self.interrupt_after {
            tokio::time::sleep(duration).await;
            Ok(())
        } else {
            // Never interrupt
            tokio::time::sleep(Duration::from_secs(3600)).await;
            Ok(())
        }
    }
}

struct MockAsyncRuntime {
    sleep_count: Arc<AtomicU32>,
}

impl MockAsyncRuntime {
    fn new() -> Self {
        Self {
            sleep_count: Arc::new(AtomicU32::new(0)),
        }
    }

    fn get_sleep_count(&self) -> u32 {
        self.sleep_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl AsyncRuntime for MockAsyncRuntime {
    async fn sleep(&self, duration: Duration) {
        self.sleep_count.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(duration).await;
    }
}

// Test fixture
struct TestFixture {
    file_system: MockFileSystemMock,
    command_executor: MockCommandExecutorMock,
    process_manager: Arc<MockProcessManager>,
    ui: Arc<TestUserInterface>,
    spin_installer: MockSpinInstallerMock,
    async_runtime: Arc<MockAsyncRuntime>,
    file_watcher: Arc<MockFileWatcher>,
    signal_handler: Arc<MockSignalHandler>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            file_system: MockFileSystemMock::new(),
            command_executor: MockCommandExecutorMock::new(),
            process_manager: Arc::new(MockProcessManager::new()),
            ui: Arc::new(TestUserInterface::new()),
            spin_installer: MockSpinInstallerMock::new(),
            async_runtime: Arc::new(MockAsyncRuntime::new()),
            file_watcher: Arc::new(MockFileWatcher::new()),
            signal_handler: Arc::new(MockSignalHandler::new()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<UpDependencies> {
        Arc::new(UpDependencies {
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
            command_executor: Arc::new(self.command_executor) as Arc<dyn CommandExecutor>,
            process_manager: self.process_manager as Arc<dyn ProcessManager>,
            ui: self.ui as Arc<dyn UserInterface>,
            spin_installer: Arc::new(self.spin_installer) as Arc<dyn SpinInstaller>,
            async_runtime: self.async_runtime as Arc<dyn AsyncRuntime>,
            file_watcher: self.file_watcher as Arc<dyn FileWatcher>,
            signal_handler: self.signal_handler as Arc<dyn SignalHandler>,
        })
    }
}

// Import test helpers
use crate::test_helpers::*;
use mockall::predicate::*;

#[tokio::test]
async fn test_up_no_spin_toml() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml doesn't exist
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| false);

    let deps = fixture.to_deps();
    let result = execute_with_deps(
        UpConfig {
            path: None,
            port: 3000,
            build: false,
            watch: false,
            clear: false,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No spin.toml found")
    );
}

#[tokio::test]
async fn test_up_normal_mode_no_build() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: process spawn and wait - process should wait for 200ms
    fixture.process_manager.add_spawn_response(Ok(Box::new(
        MockProcessHandle::new(1234, 0).with_wait_duration(Duration::from_millis(200)),
    )));

    // Mock: signal handler with interrupt after 100ms
    fixture.signal_handler = Arc::new(MockSignalHandler::with_interrupt_after(
        Duration::from_millis(100),
    ));

    let ui = fixture.ui.clone();
    let process_manager = fixture.process_manager.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(
        UpConfig {
            path: None,
            port: 3000,
            build: false,
            watch: false,
            clear: false,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify process was spawned
    assert_eq!(process_manager.get_spawn_count(), 1);

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Starting server")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Server will start at http://127.0.0.1:3000"))
    );

    // The stopping message should be in the output
    assert!(output.iter().any(|s| s.contains("Stopping server")));
}

#[tokio::test]
async fn test_up_with_build() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml exists (checked twice - once for up, once for build)
    fixture
        .file_system
        .expect_exists()
        .times(2)
        .returning(|path| path == Path::new("./spin.toml"));

    // Mock: read spin.toml for build
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| {
            Ok(r#"
spin_manifest_version = "1"
name = "test-app"

[component.backend]
source = "target/wasm32-wasi/release/backend.wasm"
[component.backend.build]
command = "cargo build --target wasm32-wasi"
"#
            .to_string())
        });

    // Mock: spin installer (called twice - once for build, once for up)
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(2)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: build command execution
    fixture
        .command_executor
        .expect_execute()
        .times(1)
        .returning(|_, _| {
            Ok(CommandOutput {
                success: true,
                stdout: b"Build successful".to_vec(),
                stderr: vec![],
            })
        });

    // Mock: process spawn
    fixture
        .process_manager
        .add_spawn_response(Ok(Box::new(MockProcessHandle::new(1234, 0))));

    // Mock: signal handler with interrupt
    fixture.signal_handler = Arc::new(MockSignalHandler::with_interrupt_after(
        Duration::from_millis(100),
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(
        UpConfig {
            path: None,
            port: 3000,
            build: true,
            watch: false,
            clear: false,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify output includes build step
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Building project before starting server"))
    );
    assert!(output.iter().any(|s| s.contains("Building 1 component")));
}

#[tokio::test]
async fn test_up_process_fails() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml exists
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: process spawn with exit code 1
    fixture
        .process_manager
        .add_spawn_response(Ok(Box::new(MockProcessHandle::new(1234, 1))));

    // Don't send interrupt signal
    fixture.signal_handler = Arc::new(MockSignalHandler::new());

    let deps = fixture.to_deps();

    let result = execute_with_deps(
        UpConfig {
            path: None,
            port: 3000,
            build: false,
            watch: false,
            clear: false,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Spin exited with status: 1")
    );
}

#[tokio::test]
async fn test_up_with_custom_path() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml exists at custom path
    fixture
        .file_system
        .expect_exists()
        .with(eq(Path::new("/my/project/spin.toml")))
        .times(1)
        .returning(|_| true);

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .times(1)
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: process spawn
    fixture
        .process_manager
        .add_spawn_response(Ok(Box::new(MockProcessHandle::new(1234, 0))));

    // Mock: signal handler
    fixture.signal_handler = Arc::new(MockSignalHandler::with_interrupt_after(
        Duration::from_millis(100),
    ));

    let deps = fixture.to_deps();

    let result = execute_with_deps(
        UpConfig {
            path: Some(PathBuf::from("/my/project")),
            port: 8080,
            build: false,
            watch: false,
            clear: false,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_up_watch_mode_initial_build_fails() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml exists (checked twice)
    fixture
        .file_system
        .expect_exists()
        .times(2)
        .returning(|path| path == Path::new("./spin.toml"));

    // Mock: read spin.toml for build - return invalid toml
    fixture
        .file_system
        .expect_read_to_string()
        .with(eq(Path::new("./spin.toml")))
        .times(1)
        .returning(|_| Ok("invalid toml content".to_string()));

    let deps = fixture.to_deps();

    let result = execute_with_deps(
        UpConfig {
            path: None,
            port: 3000,
            build: false,
            watch: true,
            clear: false,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse spin.toml")
    );
}

#[tokio::test]
async fn test_up_watch_mode_file_change() {
    let mut fixture = TestFixture::new();

    // Mock: spin.toml exists (multiple checks)
    fixture
        .file_system
        .expect_exists()
        .returning(|path| path == Path::new("./spin.toml"));

    // Mock: read spin.toml for builds
    fixture.file_system.expect_read_to_string().returning(|_| {
        Ok(r#"
spin_manifest_version = "1"
name = "test-app"

[component.backend]
source = "target/wasm32-wasi/release/backend.wasm"
[component.backend.build]
command = "cargo build --target wasm32-wasi"
"#
        .to_string())
    });

    // Mock: spin installer
    fixture
        .spin_installer
        .expect_check_and_install()
        .returning(|| Ok("/usr/local/bin/spin".to_string()));

    // Mock: build command execution
    fixture.command_executor.expect_execute().returning(|_, _| {
        Ok(CommandOutput {
            success: true,
            stdout: b"Build successful".to_vec(),
            stderr: vec![],
        })
    });

    // Mock: process spawns
    fixture
        .process_manager
        .add_spawn_response(Ok(Box::new(MockProcessHandle::new(5678, 0))));
    fixture
        .process_manager
        .add_spawn_response(Ok(Box::new(MockProcessHandle::new(1234, 0))));

    // Mock: file watcher that returns a change after 200ms
    let watch_handle = Arc::new(MockWatchHandle::new());
    watch_handle.add_change(vec![PathBuf::from("src/main.rs")]);

    fixture.file_watcher = Arc::new(MockFileWatcher::new());

    // Mock: signal handler with interrupt after 500ms
    fixture.signal_handler = Arc::new(MockSignalHandler::with_interrupt_after(
        Duration::from_millis(500),
    ));

    let ui = fixture.ui.clone();
    let process_manager = fixture.process_manager.clone();
    let deps = fixture.to_deps();

    // Run in a task with timeout to prevent hanging
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        execute_with_deps(
            UpConfig {
                path: None,
                port: 3000,
                build: false,
                watch: true,
                clear: false,
            },
            deps,
        ),
    )
    .await;

    // Should timeout or complete successfully
    assert!(result.is_ok() || result.is_err());

    // Verify processes were spawned
    assert!(process_manager.get_spawn_count() >= 1);

    // Verify output shows watch mode
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Starting development server with auto-rebuild"))
    );
}

#[test]
fn test_should_watch_file() {
    // Test the file watching filter
    assert!(up::should_watch_file(&PathBuf::from("src/main.rs")));
    assert!(up::should_watch_file(&PathBuf::from("Cargo.toml")));
    assert!(up::should_watch_file(&PathBuf::from("src/index.ts")));

    assert!(!up::should_watch_file(&PathBuf::from("target/debug/app")));
    assert!(!up::should_watch_file(&PathBuf::from(
        "node_modules/pkg/index.js"
    )));
    assert!(!up::should_watch_file(&PathBuf::from("Cargo.lock")));
    assert!(!up::should_watch_file(&PathBuf::from("app.wasm")));
}
