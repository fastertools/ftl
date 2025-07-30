//! Refactored up command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use ftl_common::SpinInstaller;
use ftl_runtime::deps::{
    AsyncRuntime, CommandExecutor, FileSystem, MessageStyle, ProcessManager, UserInterface,
};

use crate::commands::build::parse_component_builds_from_content;

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
    /// Path to the toolbox
    pub path: Option<PathBuf>,
    /// Port to listen on
    pub port: u16,
    /// Build before starting
    pub build: bool,
    /// Watch for changes and restart
    pub watch: bool,
    /// Clear terminal on rebuild
    pub clear: bool,
    /// Directory for component logs
    pub log_dir: Option<PathBuf>,
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
    let project_path = config.path.clone().unwrap_or_else(|| PathBuf::from("."));

    // Generate temporary spin.toml from ftl.toml
    let temp_spin_toml =
        crate::config::transpiler::generate_temp_spin_toml(&deps.file_system, &project_path)?;

    // We must have a temp spin.toml since ftl.toml is required
    let manifest_path = temp_spin_toml.ok_or_else(|| {
        anyhow::anyhow!("No ftl.toml found. Not in an FTL project directory? Run 'ftl init' to create a new project.")
    })?;

    // Always pass true for is_temp_manifest since we always generate temp spin.toml
    if config.watch {
        run_with_watch(project_path, manifest_path.clone(), config, &deps, true).await
    } else {
        run_normal(project_path, manifest_path.clone(), config, &deps, true).await
    }
}

async fn run_normal(
    project_path: PathBuf,
    manifest_path: PathBuf,
    config: UpConfig,
    deps: &Arc<UpDependencies>,
    is_temp_manifest: bool,
) -> Result<()> {
    // Get spin path
    let spin_path = deps.spin_installer.check_and_install().await?;

    // If build flag is set, run our parallel build first
    if config.build {
        deps.ui.print(&format!(
            "{} Building project before starting server...",
            "→"
        ));
        deps.ui.print("");

        // Run build command with our manifest path if it's temporary
        if is_temp_manifest {
            run_build_command_with_manifest(&project_path, &manifest_path, deps).await?;
        } else {
            run_build_command(&project_path, deps).await?;
        }
        deps.ui.print("");
    }

    // Build command args for spin up (without --build since we already built)
    let mut args = vec!["up", "-f", manifest_path.to_str().unwrap()];
    let listen_addr = format!("127.0.0.1:{}", config.port);
    args.extend(["--listen", &listen_addr]);

    // Set log directory - use provided path or default to .ftl/logs in project
    let log_dir = config
        .log_dir
        .clone()
        .unwrap_or_else(|| project_path.join(".ftl").join("logs"));
    args.extend(["--log-dir", log_dir.to_str().unwrap()]);

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

    // Set up Ctrl+C handler that will terminate the process
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = signal_handler.wait_for_interrupt().await;
        ctrlc_pressed_clone.store(true, Ordering::SeqCst);
        let _ = shutdown_tx.send(());
    });

    // Wait for either the process to exit or shutdown signal
    let exit_status = tokio::select! {
        status = process.wait() => status?,
        _ = &mut shutdown_rx => {
            // Ctrl+C was pressed, terminate the process
            deps.ui.print("");
            deps.ui.print_styled("■ Stopping server...", MessageStyle::Red);
            process.shutdown().await?
        }
    };

    // Clean up temporary manifest if it was created
    if is_temp_manifest {
        let _ = std::fs::remove_file(&manifest_path);
    }

    // Check exit status
    if !ctrlc_pressed.load(Ordering::SeqCst) && !exit_status.success() {
        anyhow::bail!(
            "Spin exited with status: {}",
            exit_status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

async fn run_with_watch(
    project_path: PathBuf,
    manifest_path: PathBuf,
    config: UpConfig,
    deps: &Arc<UpDependencies>,
    is_temp_manifest: bool,
) -> Result<()> {
    deps.ui.print(&format!(
        "{} Starting development server with auto-rebuild...",
        "→"
    ));
    deps.ui.print("");
    deps.ui.print("👀 Watching for file changes");
    deps.ui.print(&format!(
        "🌐 Server will start at http://127.0.0.1:{}",
        config.port
    ));
    deps.ui.print("⏹ Press Ctrl+C to stop");
    deps.ui.print("");

    // Initial build
    deps.ui.print(&format!("{} Running initial build...", "→"));
    run_build_command(&project_path, deps).await?;
    deps.ui.print("");

    // Start the server
    let spin_path = deps.spin_installer.check_and_install().await?;
    let listen_addr = format!("127.0.0.1:{}", config.port);

    // Set log directory - use provided path or default to .ftl/logs in project
    let log_dir = config
        .log_dir
        .clone()
        .unwrap_or_else(|| project_path.join(".ftl").join("logs"));
    let args = vec![
        "up",
        "-f",
        manifest_path.to_str().unwrap(),
        "--listen",
        &listen_addr,
        "--log-dir",
        log_dir.to_str().unwrap(),
    ];

    let mut server_process = deps
        .process_manager
        .spawn(&spin_path, &args, Some(&project_path))
        .await
        .context("Failed to start spin up")?;

    // Set up file watcher
    let mut watch_handle = deps.file_watcher.watch(&project_path, true).await?;

    // Set up Ctrl+C handler
    let signal_handler = deps.signal_handler.clone();
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let _ = signal_handler.wait_for_interrupt().await;
        let _ = shutdown_tx.send(());
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

                    if config.clear {
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

            // Shutdown signal received
            _ = &mut shutdown_rx => {
                deps.ui.print("");
                deps.ui.print_styled("■ Stopping development server...", MessageStyle::Red);

                // Shutdown the server (terminate and wait for exit)
                server_process.shutdown().await?;
                break;
            }
        }
    }

    // Clean up temporary manifest if it was created
    if is_temp_manifest {
        let _ = std::fs::remove_file(&manifest_path);
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
            export: None,
            export_out: None,
        },
        build_deps,
    )
    .await
}

async fn run_build_command_with_manifest(
    project_path: &Path,
    manifest_path: &Path,
    deps: &Arc<UpDependencies>,
) -> Result<()> {
    // Read the manifest content to parse components
    let manifest_content =
        std::fs::read_to_string(manifest_path).context("Failed to read manifest file")?;

    // Parse component builds from the manifest content
    let components = parse_component_builds_from_content(&manifest_content)?;

    if components.is_empty() {
        deps.ui.print_styled(
            "→ No components with build commands found",
            MessageStyle::Cyan,
        );
        return Ok(());
    }

    // Check if spin is installed
    let _spin_path = deps.spin_installer.check_and_install().await?;

    deps.ui.print(&format!(
        "→ Building {} component{} in parallel",
        components.len(),
        if components.len() > 1 { "s" } else { "" }
    ));
    deps.ui.print("");

    // Build components using our existing parallel build infrastructure
    let multi_progress = deps.ui.create_multi_progress();
    let mut tasks = JoinSet::new();

    // Track errors across all tasks
    let error_flag = Arc::new(Mutex::new(None::<String>));

    // Limit concurrent builds
    let max_concurrent = std::env::var("FTL_MAX_CONCURRENT_BUILDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(num_cpus::get);

    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    for component in components {
        let pb = multi_progress.add_spinner();
        pb.set_prefix(format!("[{}]", component.name));
        pb.set_message("Starting build...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let project_path = project_path.to_path_buf();
        let error_flag = Arc::clone(&error_flag);
        let semaphore = Arc::clone(&semaphore);
        let deps = Arc::clone(deps);

        tasks.spawn(async move {
            // Acquire permit to limit concurrency
            let _permit = semaphore.acquire().await.unwrap();

            // Check if another task has already failed
            if error_flag.lock().await.is_some() {
                pb.finish_with_message("Skipped due to error".to_string());
                return Ok(());
            }

            let start = Instant::now();
            let result = build_single_component_with_deps(
                &component,
                &project_path,
                false, // release
                pb.as_ref(),
                &deps,
            )
            .await;

            match result {
                Ok(()) => {
                    let duration = start.elapsed();
                    pb.finish_with_message(format!("✓ Built in {:.1}s", duration.as_secs_f64()));
                    Ok(())
                }
                Err(e) => {
                    pb.finish_with_message(format!("✗ Build failed: {e}"));

                    // Set error flag to prevent new tasks from starting
                    let mut error_guard = error_flag.lock().await;
                    if error_guard.is_none() {
                        *error_guard =
                            Some(format!("Component '{}' failed: {}", component.name, e));
                    }

                    Err(e)
                }
            }
        });
    }

    // Wait for all tasks to complete
    let mut first_error = None;
    while let Some(result) = tasks.join_next().await {
        if let Err(e) = result? {
            if first_error.is_none() {
                first_error = Some(e);
            }
        }
    }

    // If any component failed, return the first error
    if let Some(e) = first_error {
        return Err(e);
    }

    deps.ui.print("");
    deps.ui.print_styled(
        "✓ All components built successfully!",
        MessageStyle::Success,
    );

    Ok(())
}

async fn build_single_component_with_deps(
    component: &crate::commands::build::ComponentBuildInfo,
    working_path: &Path,
    release: bool,
    pb: &dyn ftl_runtime::deps::ProgressIndicator,
    deps: &Arc<UpDependencies>,
) -> Result<()> {
    if let Some(build_command) = &component.build_command {
        pb.set_message("Building...");

        // Determine the working directory for the build
        let build_dir = if let Some(workdir) = &component.workdir {
            working_path.join(workdir)
        } else {
            working_path.to_path_buf()
        };

        // Replace --release flag in command if needed
        let command = if release && !build_command.contains("--release") {
            // For common build tools, add release flag
            if build_command.starts_with("cargo build") {
                build_command.replace("cargo build", "cargo build --release")
            } else if build_command.starts_with("npm run build") {
                // npm scripts typically handle this internally
                build_command.to_string()
            } else {
                // For other commands, just use as-is
                build_command.to_string()
            }
        } else {
            build_command.to_string()
        };

        // Execute the build command using shell
        let (shell_cmd, shell_args) = if cfg!(target_os = "windows") {
            ("cmd", vec!["/C", &command])
        } else {
            ("sh", vec!["-c", &command])
        };

        // Run the command in the correct directory
        let cd_and_run = format!("cd {} && {}", build_dir.display(), shell_args[1]);
        let output = deps
            .command_executor
            .execute(shell_cmd, &[shell_args[0], &cd_and_run])
            .await
            .context("Failed to execute build command")?;

        if !output.success {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Build failed:\n{}", stderr));
        }
    }

    Ok(())
}

/// Up command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct UpArgs {
    /// Path to the toolbox
    pub path: Option<PathBuf>,
    /// Port to listen on
    pub port: Option<u16>,
    /// Build before starting
    pub build: bool,
    /// Watch files and rebuild automatically
    pub watch: bool,
    /// Clear screen on rebuild (only with --watch)
    pub clear: bool,
    /// Directory for component logs
    pub log_dir: Option<PathBuf>,
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
    use ftl_runtime::deps::{
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
        log_dir: args.log_dir,
    };

    execute_with_deps(config, deps).await
}

#[cfg(test)]
#[path = "up_tests.rs"]
mod tests;
