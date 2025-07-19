use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result};
use console::style;
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::commands::build;
use crate::common::spin_installer::check_and_install_spin;

pub async fn execute(
    path: Option<PathBuf>,
    port: u16,
    build: bool,
    watch: bool,
    clear: bool,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Validate project directory exists
    if !project_path.join("spin.toml").exists() {
        anyhow::bail!(
            "No spin.toml found. Not in a project directory? Run 'ftl init' to create a new project."
        );
    }

    if watch {
        run_with_watch(project_path, port, clear).await
    } else {
        run_normal(project_path, port, build).await
    }
}

async fn run_normal(project_path: PathBuf, port: u16, build: bool) -> Result<()> {
    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // If build flag is set, run our parallel build first
    if build {
        println!(
            "{} Building project before starting server...",
            style("→").cyan()
        );
        println!();

        // Use our parallel build command
        build::execute(Some(project_path.clone()), false).await?;

        println!();
    }

    // Build command args for spin up (without --build since we already built)
    let mut args = vec!["up"];
    let listen_addr = format!("127.0.0.1:{port}");
    args.extend(["--listen", &listen_addr]);

    println!("{} Starting server...", style("→").cyan());
    println!();
    println!(
        "{} Server will start at http://{}",
        style("🌐").blue(),
        listen_addr
    );
    println!("{} Press Ctrl+C to stop", style("⏹").yellow());
    println!();

    // Run spin up with inherited stdio so user can see logs
    let mut child = Command::new(&spin_path)
        .args(&args)
        .current_dir(&project_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start spin up")?;

    // Create a flag to track if Ctrl+C was pressed
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let ctrlc_pressed_clone = ctrlc_pressed.clone();

    // Set up Ctrl+C handler
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        ctrlc_pressed_clone.store(true, Ordering::SeqCst);
    });

    // Wait for the child process to exit
    let status = child.wait()?;

    // Check if we should print the stopping message
    if ctrlc_pressed.load(Ordering::SeqCst) {
        println!();
        println!("{} Stopping server...", style("■").red());
    } else if !status.success() {
        anyhow::bail!("Spin exited with status: {}", status);
    }

    Ok(())
}

async fn run_with_watch(project_path: PathBuf, port: u16, clear: bool) -> Result<()> {
    println!(
        "{} Starting development server with auto-rebuild...",
        style("→").cyan()
    );
    println!();
    println!("{} Watching for file changes", style("👀").dim());
    println!(
        "{} Server will start at http://127.0.0.1:{}",
        style("🌐").blue(),
        port
    );
    println!("{} Press Ctrl+C to stop", style("⏹").yellow());
    println!();

    // Initial build
    println!("{} Running initial build...", style("→").cyan());
    build::execute(Some(project_path.clone()), false).await?;
    println!();

    // Start the server
    let spin_path = check_and_install_spin().await?;
    let listen_addr = format!("127.0.0.1:{port}");
    let args = vec!["up", "--listen", &listen_addr];

    let mut server_process = Command::new(&spin_path)
        .args(&args)
        .current_dir(&project_path)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start spin up")?;

    // Set up file watcher
    let (tx, mut rx) = mpsc::channel(100);

    let mut watcher =
        notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            if let Ok(event) = event {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        // Filter out common non-source files and build outputs
                        let should_rebuild = event.paths.iter().any(|p| {
                            // Skip if path contains common build/output directories
                            let path_str = p.to_string_lossy();
                            if path_str.contains("/target/")
                                || path_str.contains("\\target\\")
                                || path_str.contains("/dist/")
                                || path_str.contains("\\dist\\")
                                || path_str.contains("/build/")
                                || path_str.contains("\\build\\")
                                || path_str.contains("/.spin/")
                                || path_str.contains("\\.spin\\")
                                || path_str.contains("/node_modules/")
                                || path_str.contains("\\node_modules\\")
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
                            if let Some(ext) = p.extension() {
                                let ext_str = ext.to_string_lossy();
                                matches!(
                                    ext_str.as_ref(),
                                    "rs" | "toml"
                                        | "js"
                                        | "ts"
                                        | "jsx"
                                        | "tsx"
                                        | "json"
                                        | "go"
                                        | "py"
                                        | "c"
                                        | "cpp"
                                        | "h"
                                ) && !matches!(ext_str.as_ref(), "wasm" | "wat")
                            } else {
                                false
                            }
                        });

                        if should_rebuild {
                            let _ = tx.blocking_send(());
                        }
                    }
                    _ => {}
                }
            }
        })?;

    // Watch the project directory
    watcher.watch(&project_path, RecursiveMode::Recursive)?;

    // Set up Ctrl+C handler
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let ctrlc_pressed_clone = ctrlc_pressed.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        ctrlc_pressed_clone.store(true, Ordering::SeqCst);
    });

    // Main watch loop
    loop {
        tokio::select! {
            // File change detected
            Some(_) = rx.recv() => {
                // Debounce - wait a bit for multiple file changes to settle
                tokio::time::sleep(Duration::from_millis(200)).await;

                // Drain any additional events that came in during the delay
                while rx.try_recv().is_ok() {}

                if clear {
                    // Clear screen
                    print!("\x1B[2J\x1B[1;1H");
                }

                println!("{} File change detected, rebuilding...", style("🔄").yellow());
                println!();

                // Kill the current server
                #[cfg(unix)]
                {
                    use nix::sys::signal::{self, Signal};
                    use nix::unistd::Pid;
                    let _ = signal::kill(Pid::from_raw(server_process.id() as i32), Signal::SIGTERM);
                }
                #[cfg(windows)]
                {
                    let _ = server_process.kill();
                }

                let _ = server_process.wait();

                // Rebuild
                match build::execute(Some(project_path.clone()), false).await {
                    Ok(_) => {
                        println!();
                        println!("{} Restarting server...", style("→").cyan());

                        // Start new server
                        server_process = Command::new(&spin_path)
                            .args(&args)
                            .current_dir(&project_path)
                            .stdin(Stdio::null())
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .spawn()
                            .context("Failed to restart spin up")?;
                    }
                    Err(e) => {
                        println!();
                        println!("{} Build failed: {}", style("✗").red(), e);
                        println!("{} Waiting for file changes...", style("⏸").yellow());
                    }
                }
            }

            // Ctrl+C pressed
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                if ctrlc_pressed.load(Ordering::SeqCst) {
                    println!();
                    println!("{} Stopping development server...", style("■").red());

                    // Kill the server
                    #[cfg(unix)]
                    {
                        use nix::sys::signal::{self, Signal};
                        use nix::unistd::Pid;
                        let _ = signal::kill(Pid::from_raw(server_process.id() as i32), Signal::SIGTERM);
                    }
                    #[cfg(windows)]
                    {
                        let _ = server_process.kill();
                    }

                    let _ = server_process.wait();
                    break;
                }
            }
        }
    }

    Ok(())
}
