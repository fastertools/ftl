use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use crate::common::spin_installer::check_and_install_spin;

#[derive(Debug)]
struct ComponentBuildInfo {
    name: String,
    build_command: Option<String>,
    workdir: Option<String>,
}

pub async fn execute(path: Option<PathBuf>, release: bool) -> Result<()> {
    let working_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Check if we're in a project directory (has spin.toml)
    let spin_toml_path = working_path.join("spin.toml");
    if !spin_toml_path.exists() {
        anyhow::bail!(
            "No spin.toml found. Run 'ftl build' from a project directory or use 'ftl init' to create a new project."
        );
    }

    // Parse spin.toml to find components with build commands
    let components = parse_component_builds(&spin_toml_path)?;

    if components.is_empty() {
        println!(
            "{} No components with build commands found in spin.toml",
            style("→").cyan()
        );
        return Ok(());
    }

    println!(
        "{} Building {} component{} in parallel",
        style("→").cyan(),
        style(components.len()).bold(),
        if components.len() > 1 { "s" } else { "" }
    );
    println!();

    // Check if spin is installed (in case we need it for fallback)
    let _spin_path = check_and_install_spin().await?;

    // Build all components in parallel
    build_components_parallel(components, &working_path, release).await?;

    println!();
    println!("{} All components built successfully!", style("✓").green());
    Ok(())
}

fn parse_component_builds(spin_toml_path: &Path) -> Result<Vec<ComponentBuildInfo>> {
    let content = std::fs::read_to_string(spin_toml_path).context("Failed to read spin.toml")?;
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
) -> Result<()> {
    let multi_progress = MultiProgress::new();
    let mut tasks = JoinSet::new();

    // Track errors across all tasks
    let error_flag = Arc::new(Mutex::new(None::<String>));

    // Limit concurrent builds to avoid overwhelming the system
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));

    for component in components {
        let pb = multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {prefix:.bold} {msg}")
                .unwrap(),
        );
        pb.set_prefix(format!("[{}]", component.name));
        pb.set_message("Starting build...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let working_path = working_path.to_path_buf();
        let error_flag = Arc::clone(&error_flag);
        let semaphore = Arc::clone(&semaphore);

        tasks.spawn(async move {
            // Acquire permit to limit concurrency
            let _permit = semaphore.acquire().await.unwrap();

            // Check if another task has already failed
            if error_flag.lock().await.is_some() {
                pb.finish_with_message(style("Skipped due to error").red().to_string());
                return Ok(());
            }

            let start = Instant::now();
            let result = build_single_component(&component, &working_path, release, &pb).await;

            match result {
                Ok(_) => {
                    let duration = start.elapsed();
                    pb.finish_with_message(
                        style(format!(
                            "✓ Built successfully in {:.1}s",
                            duration.as_secs_f64()
                        ))
                        .green()
                        .to_string(),
                    );
                    Ok(())
                }
                Err(e) => {
                    pb.finish_with_message(style(format!("✗ Build failed: {e}")).red().to_string());

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
    pb: &ProgressBar,
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
                build_command.clone()
            } else {
                // For other commands, just use as-is
                build_command.clone()
            }
        } else {
            build_command.clone()
        };

        // Execute the build command using shell to handle complex commands with operators
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .current_dir(&build_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .context(format!(
                    "Failed to execute build command for {}",
                    component.name
                ))?
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .current_dir(&build_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .context(format!(
                    "Failed to execute build command for {}",
                    component.name
                ))?
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Build failed:\n{}", stderr));
        }
    }

    Ok(())
}
