use std::time::Duration;

use anyhow::Result;
use tracing::{debug, info, warn};

use crate::{
    commands::build,
    common::{
        manifest_utils::validate_and_load_manifest,
        tool_paths::validate_tool_exists,
        watch_utils::{setup_file_watcher, Debouncer},
    },
};

pub async fn execute(tool_path: String) -> Result<()> {
    // Validate tool exists
    validate_tool_exists(&tool_path)?;
    let manifest = validate_and_load_manifest(&tool_path)?;

    info!("Watching tool: {} for changes...", manifest.tool.name);

    // Initial build
    println!("🔨 Initial build...");
    build::execute(Some(tool_path.clone()), None).await?;

    // Set up file watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let _watcher = setup_file_watcher(&tool_path, tx)?;

    println!("👀 Watching for changes... (Press Ctrl+C to stop)");
    println!("   Watching:");
    println!("   - {tool_path}/src/");
    println!("   - {tool_path}/Cargo.toml");
    println!("   - {tool_path}/ftl.toml");

    // Process file change events
    let mut debouncer = Debouncer::new(Duration::from_millis(500));

    loop {
        match rx.recv() {
            Ok(event) => {
                debug!("File change detected: {:?}", event.paths);

                // Debounce rapid changes
                if !debouncer.should_trigger() {
                    continue;
                }

                // Display changed files
                for path in &event.paths {
                    if let Ok(rel_path) = path.strip_prefix(&tool_path) {
                        println!("📝 Changed: {}", rel_path.display());
                    }
                }

                // Rebuild
                println!("🔨 Rebuilding...");
                match build::execute(Some(tool_path.clone()), None).await {
                    Ok(_) => println!("✅ Build successful"),
                    Err(e) => {
                        println!("❌ Build failed: {e}");
                        warn!("Build error: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Watch error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use notify::{event::ModifyKind, Event, EventKind};

    use crate::common::watch_utils::should_rebuild;

    #[test]
    fn test_should_rebuild() {
        // Should rebuild for Rust files
        let event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(PathBuf::from("src/main.rs"));
        assert!(should_rebuild(&event));

        // Should rebuild for TOML files
        let event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(PathBuf::from("Cargo.toml"));
        assert!(should_rebuild(&event));

        // Should not rebuild for non-code files
        let event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(PathBuf::from("README.md"));
        assert!(!should_rebuild(&event));
    }
}
