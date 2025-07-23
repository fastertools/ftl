//! Refactored up command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result};

use ftl_common::SpinInstaller;
use ftl_core::deps::{
    AsyncRuntime, CommandExecutor, FileSystem, MessageStyle, ProcessManager, UserInterface,
};

/// File watcher trait for testability
#[async_trait::async_trait]
pub trait FileWatcher: Send + Sync {
    /// Start watching a path for changes
    async fn watch(&self, path: &Path, recursive: bool) -> Result<Box<dyn WatchHandle>>;
}

/// Watch handle trait
#[async_trait::async_trait]
pub trait WatchHandle: Send + Sync {
    /// Wait for file changes and return changed paths
    async fn wait_for_change(&mut self) -> Result<Vec<PathBuf>>;
}

/// Signal handler trait
#[async_trait::async_trait]
pub trait SignalHandler: Send + Sync {
    /// Wait for interrupt signal (Ctrl+C)
    async fn wait_for_interrupt(&self) -> Result<()>;
}

/// Up command configuration
pub struct UpConfig {
    /// Path to the Spin application
    pub path: Option<PathBuf>,
    /// Port to listen on
    pub port: u16,
    /// Build before starting
    pub build: bool,
    /// Watch for changes and restart
    pub watch: bool,
    /// Clear terminal on rebuild
    pub clear: bool,
}

/// Dependencies for the up command
pub struct UpDependencies {
    /// File system operations
    pub file_system: Arc<dyn FileSystem>,
    /// Command execution
    pub command_executor: Arc<dyn CommandExecutor>,
    /// Process management
    pub process_manager: Arc<dyn ProcessManager>,
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// Spin CLI installer
    pub spin_installer: Arc<dyn SpinInstaller>,
    /// Async runtime for delays
    pub async_runtime: Arc<dyn AsyncRuntime>,
    /// File watcher for watch mode
    pub file_watcher: Arc<dyn FileWatcher>,
    /// Signal handler for graceful shutdown
    pub signal_handler: Arc<dyn SignalHandler>,
}

/// Execute the up command with injected dependencies
pub async fn execute_with_deps(config: UpConfig, deps: Arc<UpDependencies>) -> Result<()> {
    let project_path = config.path.unwrap_or_else(|| PathBuf::from("."));

    // Validate project directory exists
    if !deps.file_system.exists(&project_path.join("spin.toml")) {
        anyhow::bail!(
            "No spin.toml found. Not in a project directory? Run 'ftl init' to create a new project."
        );
    }

    if config.watch {
        run_with_watch(project_path, config.port, config.clear, &deps).await
    } else {
        run_normal(project_path, config.port, config.build, &deps).await
    }
}

async fn run_normal(
    project_path: PathBuf,
    port: u16,
    build: bool,
    deps: &Arc<UpDependencies>,
) -> Result<()> {
    // Get spin path
    let spin_path = deps.spin_installer.check_and_install().await?;

    // If build flag is set, run our parallel build first
    if build {
        deps.ui.print(&format!(
            "{} Building project before starting server...",
            "→"
        ));
        deps.ui.print("");

        // Run build command
        run_build_command(&project_path, deps).await?;
        deps.ui.print("");
    }

    // Build command args for spin up (without --build since we already built)
    let mut args = vec!["up"];
    let listen_addr = format!("127.0.0.1:{port}");
    args.extend(["--listen", &listen_addr]);

    deps.ui.print(&format!("{} Starting server...", "→"));
    deps.ui.print("");
    deps.ui
        .print(&format!("🌐 Server will start at http://{listen_addr}"));
    deps.ui.print("⏹ Press Ctrl+C to stop");
    deps.ui.print("");

    // Start the server process
    let mut process = deps
        .process_manager
        .spawn(&spin_path, &args, Some(&project_path))
        .await
        .context("Failed to start spin up")?;

    // Create a flag to track if Ctrl+C was pressed
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let ctrlc_pressed_clone = ctrlc_pressed.clone();
    let signal_handler = deps.signal_handler.clone();

    // Set up Ctrl+C handler
    tokio::spawn(async move {
        let _ = signal_handler.wait_for_interrupt().await;
        ctrlc_pressed_clone.store(true, Ordering::SeqCst);
    });

    // Wait for the process to exit
    let exit_status = process.wait().await?;

    // Check if we should print the stopping message
    if ctrlc_pressed.load(Ordering::SeqCst) {
        deps.ui.print("");
        deps.ui
            .print_styled("■ Stopping server...", MessageStyle::Red);
    } else if !exit_status.success() {
        anyhow::bail!(
            "Spin exited with status: {}",
            exit_status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

async fn run_with_watch(
    project_path: PathBuf,
    port: u16,
    clear: bool,
    deps: &Arc<UpDependencies>,
) -> Result<()> {
    deps.ui.print(&format!(
        "{} Starting development server with auto-rebuild...",
        "→"
    ));
    deps.ui.print("");
    deps.ui.print("👀 Watching for file changes");
    deps.ui
        .print(&format!("🌐 Server will start at http://127.0.0.1:{port}"));
    deps.ui.print("⏹ Press Ctrl+C to stop");
    deps.ui.print("");

    // Initial build
    deps.ui.print(&format!("{} Running initial build...", "→"));
    run_build_command(&project_path, deps).await?;
    deps.ui.print("");

    // Start the server
    let spin_path = deps.spin_installer.check_and_install().await?;
    let listen_addr = format!("127.0.0.1:{port}");
    let args = vec!["up", "--listen", &listen_addr];

    let mut server_process = deps
        .process_manager
        .spawn(&spin_path, &args, Some(&project_path))
        .await
        .context("Failed to start spin up")?;

    // Set up file watcher
    let mut watch_handle = deps.file_watcher.watch(&project_path, true).await?;

    // Set up Ctrl+C handler
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let ctrlc_pressed_clone = ctrlc_pressed.clone();
    let signal_handler = deps.signal_handler.clone();

    tokio::spawn(async move {
        let _ = signal_handler.wait_for_interrupt().await;
        ctrlc_pressed_clone.store(true, Ordering::SeqCst);
    });

    // Main watch loop
    loop {
        tokio::select! {
            // File change detected
            Ok(changed_files) = watch_handle.wait_for_change() => {
                // Check if any of the changed files should trigger a rebuild
                let should_rebuild = changed_files.iter().any(|p| should_watch_file(p));

                if should_rebuild {
                    // Debounce - wait a bit for multiple file changes to settle
                    deps.async_runtime.sleep(Duration::from_millis(200)).await;

                    if clear {
                        // Clear screen
                        deps.ui.clear_screen();
                    }

                    deps.ui.print_styled("🔄 File change detected, rebuilding...", MessageStyle::Yellow);
                    deps.ui.print("");

                    // Kill the current server
                    server_process.shutdown().await?;

                    // Give the OS a moment to fully release the port
                    deps.async_runtime.sleep(Duration::from_secs(1)).await;

                    // Rebuild
                    match run_build_command(&project_path, deps).await {
                        Ok(()) => {
                            deps.ui.print("");
                            deps.ui.print(&format!("{} Restarting server...", "→"));

                            // Start new server
                            server_process = deps.process_manager
                                .spawn(&spin_path, &args, Some(&project_path))
                                .await
                                .context("Failed to restart spin up")?;
                        }
                        Err(e) => {
                            deps.ui.print("");
                            deps.ui.print_styled(&format!("✗ Build failed: {e}"), MessageStyle::Red);
                            deps.ui.print_styled("⏸ Waiting for file changes...", MessageStyle::Yellow);
                        }
                    }
                }
            }

            // Check for Ctrl+C periodically
            () = deps.async_runtime.sleep(Duration::from_millis(100)) => {
                if ctrlc_pressed.load(Ordering::SeqCst) {
                    deps.ui.print("");
                    deps.ui.print_styled("■ Stopping development server...", MessageStyle::Red);

                    // Shutdown the server (terminate and wait for exit)
                    server_process.shutdown().await?;
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Check if a file should trigger a rebuild when changed
pub fn should_watch_file(path: &Path) -> bool {
    // Skip if path contains common build/output directories
    let path_str = path.to_string_lossy();

    // Check for excluded directories (with or without leading separator)
    if path_str.contains("target/")
        || path_str.contains("target\\")
        || path_str.contains("dist/")
        || path_str.contains("dist\\")
        || path_str.contains("build/")
        || path_str.contains("build\\")
        || path_str.contains(".spin/")
        || path_str.contains(".spin\\")
        || path_str.contains("node_modules/")
        || path_str.contains("node_modules\\")
        || path_str.ends_with(".wasm")
        || path_str.ends_with(".wat")
        || path_str.ends_with("package-lock.json")
        || path_str.ends_with("yarn.lock")
        || path_str.ends_with("pnpm-lock.yaml")
        || path_str.ends_with("Cargo.lock")
    {
        return false;
    }

    // Only watch source files
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy();
        matches!(
            ext_str.as_ref(),
            "rs" | "toml" | "js" | "ts" | "jsx" | "tsx" | "json" | "go" | "py" | "c" | "cpp" | "h"
        ) && !matches!(ext_str.as_ref(), "wasm" | "wat")
    } else {
        false
    }
}

async fn run_build_command(project_path: &Path, deps: &Arc<UpDependencies>) -> Result<()> {
    // For now, we'll use the build command directly
    // In a real implementation, we'd refactor build command to be callable
    use crate::commands::build::{
        BuildConfig, BuildDependencies, execute_with_deps as build_execute,
    };

    let build_deps = Arc::new(BuildDependencies {
        file_system: deps.file_system.clone(),
        command_executor: deps.command_executor.clone(),
        ui: deps.ui.clone(),
        spin_installer: deps.spin_installer.clone(),
    });

    build_execute(
        BuildConfig {
            path: Some(project_path.to_path_buf()),
            release: false,
        },
        build_deps,
    )
    .await
}

/// Up command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct UpArgs {
    /// Path to the Spin application
    pub path: Option<PathBuf>,
    /// Port to listen on
    pub port: Option<u16>,
    /// Build before starting
    pub build: bool,
    /// Watch files and rebuild automatically
    pub watch: bool,
    /// Clear screen on rebuild (only with --watch)
    pub clear: bool,
}

// File watcher implementation using notify
struct RealFileWatcher;

#[async_trait::async_trait]
impl FileWatcher for RealFileWatcher {
    async fn watch(&self, path: &Path, recursive: bool) -> Result<Box<dyn WatchHandle>> {
        use notify::{RecursiveMode, Watcher};
        use tokio::sync::mpsc;

        let (tx, rx) = mpsc::unbounded_channel();
        let path = path.to_path_buf();

        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        for path in event.paths {
                            let _ = tx.send(path);
                        }
                    }
                }
            })?;

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        watcher.watch(&path, mode)?;

        Ok(Box::new(RealWatchHandle {
            _watcher: Box::new(watcher),
            rx,
        }))
    }
}

struct RealWatchHandle {
    _watcher: Box<dyn notify::Watcher + Send + Sync>,
    rx: tokio::sync::mpsc::UnboundedReceiver<PathBuf>,
}

#[async_trait::async_trait]
impl WatchHandle for RealWatchHandle {
    async fn wait_for_change(&mut self) -> Result<Vec<PathBuf>> {
        let mut changes = Vec::new();

        // Wait for first change
        if let Some(path) = self.rx.recv().await {
            changes.push(path);
        }

        // Collect any additional changes that arrive quickly
        while let Ok(path) = self.rx.try_recv() {
            changes.push(path);
        }

        if changes.is_empty() {
            anyhow::bail!("Watcher closed unexpectedly");
        }

        Ok(changes)
    }
}

// Signal handler implementation
struct RealSignalHandler;

#[async_trait::async_trait]
impl SignalHandler for RealSignalHandler {
    async fn wait_for_interrupt(&self) -> Result<()> {
        tokio::signal::ctrl_c().await?;
        Ok(())
    }
}

// Spin installer wrapper
struct SpinInstallerWrapper;

#[async_trait::async_trait]
impl SpinInstaller for SpinInstallerWrapper {
    async fn check_and_install(&self) -> Result<String> {
        let path = ftl_common::check_and_install_spin().await?;
        Ok(path.to_string_lossy().to_string())
    }
}

/// Execute the up command with default dependencies
pub async fn execute(args: UpArgs) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_core::deps::{
        RealAsyncRuntime, RealCommandExecutor, RealFileSystem, RealProcessManager,
    };

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(UpDependencies {
        file_system: Arc::new(RealFileSystem),
        command_executor: Arc::new(RealCommandExecutor),
        process_manager: Arc::new(RealProcessManager),
        ui: ui.clone(),
        spin_installer: Arc::new(SpinInstallerWrapper),
        async_runtime: Arc::new(RealAsyncRuntime),
        file_watcher: Arc::new(RealFileWatcher),
        signal_handler: Arc::new(RealSignalHandler),
    });

    let config = UpConfig {
        path: args.path,
        port: args.port.unwrap_or(3000),
        build: args.build,
        watch: args.watch,
        clear: args.clear,
    };

    execute_with_deps(config, deps).await
}

#[cfg(test)]
#[path = "up_tests.rs"]
mod tests;
