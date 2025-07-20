//! Refactored up command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result};

use crate::deps::{
    AsyncRuntime, CommandExecutor, FileSystem, MessageStyle, ProcessManager, SpinInstaller,
    UserInterface,
};

/// File watcher trait for testability
#[async_trait::async_trait]
pub trait FileWatcher: Send + Sync {
    async fn watch(&self, path: &Path, recursive: bool) -> Result<Box<dyn WatchHandle>>;
}

/// Watch handle trait
#[async_trait::async_trait]
pub trait WatchHandle: Send + Sync {
    async fn wait_for_change(&mut self) -> Result<Vec<PathBuf>>;
}

/// Signal handler trait
#[async_trait::async_trait]
pub trait SignalHandler: Send + Sync {
    async fn wait_for_interrupt(&self) -> Result<()>;
}

/// Up command configuration
pub struct UpConfig {
    pub path: Option<PathBuf>,
    pub port: u16,
    pub build: bool,
    pub watch: bool,
    pub clear: bool,
}

/// Dependencies for the up command
pub struct UpDependencies {
    pub file_system: Arc<dyn FileSystem>,
    pub command_executor: Arc<dyn CommandExecutor>,
    pub process_manager: Arc<dyn ProcessManager>,
    pub ui: Arc<dyn UserInterface>,
    pub spin_installer: Arc<dyn SpinInstaller>,
    pub async_runtime: Arc<dyn AsyncRuntime>,
    pub file_watcher: Arc<dyn FileWatcher>,
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
                    server_process.terminate().await?;
                    server_process.wait().await?;

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

                    // Kill the server
                    server_process.terminate().await?;
                    server_process.wait().await?;
                    break;
                }
            }
        }
    }

    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_watch_file() {
        use std::path::PathBuf;

        // Should watch source files
        assert!(should_watch_file(&PathBuf::from("src/main.rs")));
        assert!(should_watch_file(&PathBuf::from("lib.rs")));
        assert!(should_watch_file(&PathBuf::from("src/index.ts")));
        assert!(should_watch_file(&PathBuf::from("app.js")));
        assert!(should_watch_file(&PathBuf::from("Cargo.toml")));
        assert!(should_watch_file(&PathBuf::from("package.json")));

        // Should not watch build outputs
        assert!(!should_watch_file(&PathBuf::from("target/debug/app")));
        assert!(!should_watch_file(&PathBuf::from("dist/bundle.js")));
        assert!(!should_watch_file(&PathBuf::from("build/output.wasm")));
        assert!(!should_watch_file(&PathBuf::from(".spin/config")));
        assert!(!should_watch_file(&PathBuf::from(
            "node_modules/package/index.js"
        )));

        // Should not watch lock files
        assert!(!should_watch_file(&PathBuf::from("Cargo.lock")));
        assert!(!should_watch_file(&PathBuf::from("package-lock.json")));
        assert!(!should_watch_file(&PathBuf::from("yarn.lock")));

        // Should not watch wasm files
        assert!(!should_watch_file(&PathBuf::from("module.wasm")));
        assert!(!should_watch_file(&PathBuf::from("module.wat")));
    }
}
