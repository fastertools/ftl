use std::{
    path::Path,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};

use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tracing::debug;

/// Check if a file system event should trigger a rebuild
pub fn should_rebuild(event: &Event) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            event.paths.iter().any(|path| {
                if let Some(ext) = path.extension() {
                    matches!(ext.to_str(), Some("rs") | Some("toml") | Some("lock"))
                } else {
                    false
                }
            })
        }
        _ => false,
    }
}

/// Set up file watcher for a tool directory
pub fn setup_file_watcher<P: AsRef<Path>>(
    tool_path: P,
    tx: Sender<Event>,
) -> Result<notify::RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res
            && should_rebuild(&event)
        {
            debug!("File change detected: {:?}", event.paths);
            let _ = tx.send(event);
        }
    })?;

    let tool_path = tool_path.as_ref();

    // Watch source directory
    let src_dir = tool_path.join("src");
    if src_dir.exists() {
        watcher.watch(&src_dir, RecursiveMode::Recursive)?;
    }

    // Watch Cargo files
    let cargo_toml = tool_path.join("Cargo.toml");
    if cargo_toml.exists() {
        watcher.watch(&cargo_toml, RecursiveMode::NonRecursive)?;
    }

    let cargo_lock = tool_path.join("Cargo.lock");
    if cargo_lock.exists() {
        watcher.watch(&cargo_lock, RecursiveMode::NonRecursive)?;
    }

    // Watch ftl.toml
    let ftl_toml = tool_path.join("ftl.toml");
    if ftl_toml.exists() {
        watcher.watch(&ftl_toml, RecursiveMode::NonRecursive)?;
    }

    Ok(watcher)
}

/// Debouncer to prevent rapid rebuilds
pub struct Debouncer {
    last_trigger: Instant,
    duration: Duration,
}

impl Debouncer {
    pub fn new(duration: Duration) -> Self {
        Self {
            last_trigger: Instant::now()
                .checked_sub(duration)
                .unwrap_or(Instant::now()),
            duration,
        }
    }

    /// Check if enough time has passed since last trigger
    pub fn should_trigger(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_trigger) >= self.duration {
            self.last_trigger = now;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use notify::event::ModifyKind;

    use super::*;

    #[test]
    fn test_should_rebuild_rust_files() {
        let event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(PathBuf::from("src/main.rs"));
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_rebuild_toml_files() {
        let event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(PathBuf::from("Cargo.toml"));
        assert!(should_rebuild(&event));
    }

    #[test]
    fn test_should_not_rebuild_other_files() {
        let event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(PathBuf::from("README.md"));
        assert!(!should_rebuild(&event));
    }

    #[test]
    fn test_debouncer() {
        let mut debouncer = Debouncer::new(Duration::from_millis(100));

        // First trigger should succeed
        assert!(debouncer.should_trigger());

        // Immediate second trigger should fail
        assert!(!debouncer.should_trigger());

        // After waiting, should succeed
        std::thread::sleep(Duration::from_millis(150));
        assert!(debouncer.should_trigger());
    }
}
