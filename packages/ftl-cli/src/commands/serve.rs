use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::Result;
use console::style;
use tokio::{signal, time::sleep};
use tracing::{debug, warn};

use crate::{
    common::{
        manifest_utils::load_manifest_and_name,
        spin_installer::check_and_install_spin,
        spin_utils::start_spin_server_with_path,
        tool_paths::{
            ensure_ftl_dir, get_profile_dir, get_spin_toml_path, get_wasm_path,
            validate_tool_exists,
        },
        watch_utils::{Debouncer, setup_file_watcher},
    },
    language::Language,
    spin_generator,
};

pub async fn execute(tool_path: String, port: u16, build_first: bool) -> Result<()> {
    println!(
        "{} Serving tool: {} on port {}",
        style("→").cyan(),
        style(&tool_path).bold(),
        style(port).yellow()
    );

    // Validate tool exists and load manifest
    validate_tool_exists(&tool_path)?;
    let (manifest, tool_name) = load_manifest_and_name(&tool_path)?;

    // Build if requested
    if build_first {
        println!("{} Building tool first...", style("→").cyan());
        crate::commands::build::execute(Some(tool_path.clone()), None).await?;
    }

    // Check WASM binary exists and determine spin.toml path
    let (_wasm_path, spin_toml_path) = match manifest.tool.language {
        Language::Rust => {
            let wasm = get_wasm_path(&tool_path, &tool_name, &manifest.build.profile);
            if !wasm.exists() {
                anyhow::bail!(
                    "WASM binary not found at: {}. Run 'ftl build {}' first.",
                    wasm.display(),
                    tool_path
                );
            }

            // Ensure .ftl directory and spin.toml exist for Rust
            ensure_ftl_dir(&tool_path)?;
            let spin_path = get_spin_toml_path(&tool_path);

            if !spin_path.exists() {
                // Generate development spin.toml if it doesn't exist
                let profile_dir = get_profile_dir(&manifest.build.profile);
                let wasm_filename = format!("{}.wasm", tool_name.replace('-', "_"));
                let relative_wasm_path = PathBuf::from("..")
                    .join("target")
                    .join("wasm32-wasip1")
                    .join(profile_dir)
                    .join(&wasm_filename);

                let spin_content = spin_generator::generate_development_config(
                    &tool_name,
                    port,
                    &relative_wasm_path,
                )?;
                std::fs::write(&spin_path, spin_content)?;
            }

            (wasm, spin_path)
        }
        Language::JavaScript | Language::TypeScript => {
            // For JS/TS, use Spin's generated paths
            let wasm = PathBuf::from(&tool_path)
                .join("dist")
                .join(format!("{tool_name}.wasm"));
            if !wasm.exists() {
                anyhow::bail!(
                    "WASM binary not found at: {}. Run 'ftl build {}' first.",
                    wasm.display(),
                    tool_path
                );
            }

            // Use spin.toml from .ftl directory
            let spin_path = get_spin_toml_path(&tool_path);
            if !spin_path.exists() {
                anyhow::bail!(
                    "spin.toml not found in .ftl directory. Run 'ftl build {}' first.",
                    tool_path
                );
            }

            (wasm, spin_path)
        }
    };

    // Check spin is installed and get the path
    let spin_path = check_and_install_spin().await?;

    // Set up hot reload
    let should_rebuild = Arc::new(AtomicBool::new(false));
    let rebuild_flag = should_rebuild.clone();

    // Set up file watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let _watcher = setup_file_watcher(&tool_path, tx)?;

    let tool_path_clone = tool_path.clone();
    let watcher_task = tokio::task::spawn_blocking(move || {
        let mut debouncer = Debouncer::new(Duration::from_millis(500));

        while let Ok(event) = rx.recv() {
            if debouncer.should_trigger() {
                // Set rebuild flag
                rebuild_flag.store(true, Ordering::Relaxed);

                // Display changed files
                for path in &event.paths {
                    if let Ok(rel_path) = path.strip_prefix(&tool_path_clone) {
                        println!("\n📝 Changed: {}", rel_path.display());
                    }
                }

                println!("🔄 Reloading...");
            }
        }
    });

    // Start initial server
    println!();
    println!(
        "{} Starting development server with hot reload...",
        style("▶").green()
    );
    println!();
    println!("  Tool: {tool_path}");
    println!("  URL: http://localhost:{port}/mcp");
    println!("  Watching for changes in src/");
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    let mut server =
        start_spin_server_with_path(&spin_path, &tool_path, port, Some(&spin_toml_path))?;

    // Main server loop with rebuild handling
    let rebuild_check = should_rebuild.clone();
    let mut rebuild_interval = tokio::time::interval(Duration::from_millis(250));

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!();
                println!("{} Stopping server...", style("■").red());
                break;
            }
            _ = rebuild_interval.tick() => {
                if rebuild_check.load(Ordering::Relaxed) {
                    rebuild_check.store(false, Ordering::Relaxed);

                    // Stop current server
                    if let Err(e) = server.kill() {
                        warn!("Failed to stop server: {}", e);
                    }
                    let _ = server.wait();

                    // Rebuild
                    match crate::commands::build::execute(Some(tool_path.clone()), None).await {
                        Ok(_) => {
                            println!("✅ Build successful, restarting server...");

                            // Small delay to ensure port is released
                            sleep(Duration::from_millis(100)).await;

                            // Restart server
                            match start_spin_server_with_path(&spin_path, &tool_path, port, Some(&spin_toml_path)) {
                                Ok(new_server) => {
                                    server = new_server;
                                }
                                Err(e) => {
                                    println!("❌ Failed to restart server: {e}");
                                    println!("   Fix the issue and save to retry");
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Build failed: {e}");
                            println!("   Fix the error and save to retry");

                            // Restart server anyway (will serve last good build)
                            if let Ok(new_server) = start_spin_server_with_path(&spin_path, &tool_path, port, Some(&spin_toml_path)) {
                                server = new_server;
                            }
                        }
                    }
                }
            }
        }
    }

    // Cleanup
    drop(watcher_task);
    if let Err(e) = server.kill() {
        debug!("Failed to stop server during cleanup: {}", e);
    }

    Ok(())
}
