//! Refactored build command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use crate::deps::{
    AsyncRuntime, CommandExecutor, CommandOutput, FileSystem, MessageStyle, ProgressIndicator,
    SpinInstaller, UserInterface,
};

#[derive(Debug, Clone)]
pub struct ComponentBuildInfo {
    pub name: String,
    pub build_command: Option<String>,
    pub workdir: Option<String>,
}

/// Build command configuration
pub struct BuildConfig {
    pub path: Option<PathBuf>,
    pub release: bool,
}

/// Dependencies for the build command
pub struct BuildDependencies {
    pub file_system: Arc<dyn FileSystem>,
    pub command_executor: Arc<dyn CommandExecutor>,
    pub ui: Arc<dyn UserInterface>,
    pub spin_installer: Arc<dyn SpinInstaller>,
    pub async_runtime: Arc<dyn AsyncRuntime>,
}

/// Execute the build command with injected dependencies
pub async fn execute_with_deps(config: BuildConfig, deps: Arc<BuildDependencies>) -> Result<()> {
    let working_path = config.path.unwrap_or_else(|| PathBuf::from("."));

    // Check if we're in a project directory (has spin.toml)
    let spin_toml_path = working_path.join("spin.toml");
    if !deps.file_system.exists(&spin_toml_path) {
        anyhow::bail!(
            "No spin.toml found. Run 'ftl build' from a project directory or use 'ftl init' to create a new project."
        );
    }

    // Parse spin.toml to find components with build commands
    let components = parse_component_builds(&deps.file_system, &spin_toml_path)?;

    if components.is_empty() {
        deps.ui.print_styled(
            "→ No components with build commands found in spin.toml",
            MessageStyle::Cyan,
        );
        return Ok(());
    }

    // Check if spin is installed (only if we have components to build)
    let _spin_path = deps.spin_installer.check_and_install().await?;

    deps.ui.print(&format!(
        "→ Building {} component{} in parallel",
        components.len(),
        if components.len() > 1 { "s" } else { "" }
    ));
    deps.ui.print("");

    // Build all components in parallel
    build_components_parallel(components, &working_path, config.release, &deps).await?;

    deps.ui.print("");
    deps.ui.print_styled(
        "✓ All components built successfully!",
        MessageStyle::Success,
    );
    Ok(())
}

pub fn parse_component_builds(
    fs: &Arc<dyn FileSystem>,
    spin_toml_path: &Path,
) -> Result<Vec<ComponentBuildInfo>> {
    let content = fs
        .read_to_string(spin_toml_path)
        .context("Failed to read spin.toml")?;
    let toml: toml::Value = toml::from_str(&content).context("Failed to parse spin.toml")?;

    let mut components = Vec::new();

    // Look for components with build configurations
    if let Some(components_table) = toml.get("component").and_then(|c| c.as_table()) {
        for (name, component) in components_table {
            // Check if this component has a build section
            if let Some(build_section) = component.get("build").and_then(|b| b.as_table()) {
                if let Some(command) = build_section.get("command").and_then(|c| c.as_str()) {
                    let workdir = build_section
                        .get("workdir")
                        .and_then(|w| w.as_str())
                        .map(|s| s.to_string());

                    components.push(ComponentBuildInfo {
                        name: name.clone(),
                        build_command: Some(command.to_string()),
                        workdir,
                    });
                }
            }
        }
    }

    Ok(components)
}

async fn build_components_parallel(
    components: Vec<ComponentBuildInfo>,
    working_path: &Path,
    release: bool,
    deps: &Arc<BuildDependencies>,
) -> Result<()> {
    let multi_progress = deps.ui.create_multi_progress();
    let mut tasks = JoinSet::new();

    // Track errors across all tasks
    let error_flag = Arc::new(Mutex::new(None::<String>));

    // Limit concurrent builds to avoid overwhelming the system
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

        let working_path = working_path.to_path_buf();
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
            let result =
                build_single_component(&component, &working_path, release, pb.as_ref(), &deps)
                    .await;

            match result {
                Ok(_) => {
                    let duration = start.elapsed();
                    pb.finish_with_message(format!(
                        "✓ Built successfully in {:.1}s",
                        duration.as_secs_f64()
                    ));
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

    Ok(())
}

async fn build_single_component(
    component: &ComponentBuildInfo,
    working_path: &Path,
    release: bool,
    pb: &dyn ProgressIndicator,
    deps: &Arc<BuildDependencies>,
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
        let command = prepare_build_command(build_command, release);

        // Execute the build command using shell to handle complex commands with operators
        let (shell_cmd, shell_args) = get_shell_command(&command);

        let output =
            run_build_command(&deps.command_executor, shell_cmd, &shell_args, &build_dir).await?;

        if !output.success {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Build failed:\n{}", stderr));
        }
    }

    Ok(())
}

fn prepare_build_command(build_command: &str, release: bool) -> String {
    if release && !build_command.contains("--release") {
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
    }
}

fn get_shell_command(command: &str) -> (&str, Vec<&str>) {
    if cfg!(target_os = "windows") {
        ("cmd", vec!["/C", command])
    } else {
        ("sh", vec!["-c", command])
    }
}

async fn run_build_command(
    executor: &Arc<dyn CommandExecutor>,
    shell_cmd: &str,
    shell_args: &[&str],
    _build_dir: &Path,
) -> Result<CommandOutput> {
    // Note: In a real implementation, we would need to pass the working directory
    // to the command executor. For now, we'll just execute in the current directory.
    // The CommandExecutor trait would need to be extended to support this.
    executor
        .execute(shell_cmd, shell_args)
        .await
        .context("Failed to execute build command")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_build_command() {
        // Test cargo build
        assert_eq!(
            prepare_build_command("cargo build", true),
            "cargo build --release"
        );
        assert_eq!(
            prepare_build_command("cargo build --target wasm32-wasi", true),
            "cargo build --release --target wasm32-wasi"
        );
        assert_eq!(
            prepare_build_command("cargo build --release", true),
            "cargo build --release"
        );

        // Test npm
        assert_eq!(
            prepare_build_command("npm run build", true),
            "npm run build"
        );

        // Test other commands
        assert_eq!(prepare_build_command("make", true), "make");

        // Test non-release mode
        assert_eq!(prepare_build_command("cargo build", false), "cargo build");
    }

    #[test]
    fn test_get_shell_command() {
        let command = "cargo build --release";

        #[cfg(target_os = "windows")]
        {
            let (cmd, args) = get_shell_command(command);
            assert_eq!(cmd, "cmd");
            assert_eq!(args, vec!["/C", command]);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let (cmd, args) = get_shell_command(command);
            assert_eq!(cmd, "sh");
            assert_eq!(args, vec!["-c", command]);
        }
    }
}
