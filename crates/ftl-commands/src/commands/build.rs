//! Refactored build command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use ftl_common::SpinInstaller;
use ftl_core::deps::{
    CommandExecutor, CommandOutput, FileSystem, MessageStyle, ProgressIndicator, UserInterface,
};

/// Information about a component to build
#[derive(Debug, Clone)]
pub struct ComponentBuildInfo {
    /// Component name
    pub name: String,
    /// Build command to execute
    pub build_command: Option<String>,
    /// Working directory for the build
    pub workdir: Option<String>,
}

/// Build command configuration
pub struct BuildConfig {
    /// Path to the Spin application
    pub path: Option<PathBuf>,
    /// Build in release mode
    pub release: bool,
}

/// Dependencies for the build command
pub struct BuildDependencies {
    /// File system operations
    pub file_system: Arc<dyn FileSystem>,
    /// Command execution operations
    pub command_executor: Arc<dyn CommandExecutor>,
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// Spin CLI installer
    pub spin_installer: Arc<dyn SpinInstaller>,
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

/// Parse component build information from spin.toml
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
                        .map(std::string::ToString::to_string);

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
                Ok(()) => {
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
    build_dir: &Path,
) -> Result<CommandOutput> {
    // shell_args already contains ["-c", "command"], so we need to modify the command part
    let original_command = shell_args.get(1).unwrap_or(&"");
    let cd_and_run = format!("cd {} && {}", build_dir.display(), original_command);

    // Build the new command with the cd prefix
    let result = if shell_args.len() >= 2 {
        // For sh -c "command", replace with sh -c "cd dir && command"
        executor
            .execute(shell_cmd, &[shell_args[0], &cd_and_run])
            .await
    } else {
        // Fallback case - shouldn't happen in normal usage
        executor.execute(shell_cmd, &[&cd_and_run]).await
    };

    result.context("Failed to execute build command")
}

/// Build command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct BuildArgs {
    /// Path to the Spin application
    pub path: Option<PathBuf>,
    /// Build in release mode
    pub release: bool,
}

// Spin installer wrapper that adapts the common implementation
struct SpinInstallerWrapper;

#[async_trait::async_trait]
impl SpinInstaller for SpinInstallerWrapper {
    async fn check_and_install(&self) -> Result<String> {
        let path = ftl_common::check_and_install_spin().await?;
        Ok(path.to_string_lossy().to_string())
    }
}

/// Execute the build command with default dependencies
pub async fn execute(args: BuildArgs) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_core::deps::{RealCommandExecutor, RealFileSystem};

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(BuildDependencies {
        file_system: Arc::new(RealFileSystem),
        command_executor: Arc::new(RealCommandExecutor),
        ui: ui.clone(),
        spin_installer: Arc::new(SpinInstallerWrapper),
    });

    let config = BuildConfig {
        path: args.path,
        release: args.release,
    };

    execute_with_deps(config, deps).await
}

#[cfg(test)]
#[path = "build_tests.rs"]
mod tests;
