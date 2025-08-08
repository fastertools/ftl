//! Component management commands for publishing, pulling, and listing WASM components

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use ftl_runtime::deps::{CommandExecutor, FileSystem, MessageStyle, UserInterface};

/// Dependencies for component commands
pub struct ComponentDependencies {
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// File system operations
    pub file_system: Arc<dyn FileSystem>,
    /// Command executor for running wkg and crane
    pub command_executor: Arc<dyn CommandExecutor>,
}

/// Publish a WASM component to a registry
///
/// # Arguments
/// * `deps` - Dependency injection container
/// * `component_path` - Path to component directory or WASM file
/// * `registry` - Registry URL override (uses `default_registry` if not specified)
/// * `name` - Component name override (derives from path if not specified)
/// * `tag` - Version tag (defaults to "latest")
/// * `yes` - Skip confirmation prompt
pub async fn publish_with_deps(
    deps: &Arc<ComponentDependencies>,
    component_path: &Path,
    registry: Option<&str>,
    name: Option<&str>,
    tag: Option<&str>,
    yes: bool,
) -> Result<()> {
    deps.ui
        .print_styled("Publishing component", MessageStyle::Cyan);

    // Determine the WASM file to publish
    let wasm_path = resolve_wasm_path(deps, component_path)?;

    // Determine component name
    let component_name = if let Some(n) = name {
        n.to_string()
    } else {
        derive_component_name(component_path)?
    };

    // Get registry from ftl.toml or use override
    let registry_url = resolve_registry_url(deps, registry)?;

    // Determine version tag
    let version = tag.unwrap_or("latest");

    // Construct full registry reference
    let full_ref = format!("{registry_url}/{component_name}:{version}");

    // Confirm with user
    if !yes {
        deps.ui.print("");
        deps.ui.print(&format!("Component: {component_name}"));
        deps.ui
            .print(&format!("WASM file: {}", wasm_path.display()));
        deps.ui.print(&format!("Registry: {registry_url}"));
        deps.ui.print(&format!("Tag: {version}"));
        deps.ui.print(&format!("Full reference: {full_ref}"));
        deps.ui.print("");

        if !deps.ui.prompt_confirm("Continue?", true)? {
            deps.ui
                .print_styled("Publish cancelled", MessageStyle::Yellow);
            return Ok(());
        }
    }

    // Verify WASM file exists
    if !deps.file_system.exists(&wasm_path) {
        anyhow::bail!("WASM file not found: {}", wasm_path.display());
    }

    // Push to registry using wkg with spinner
    let spinner = deps.ui.create_spinner();
    spinner.set_message(&format!("Pushing to {full_ref}"));

    let wasm_path_str = wasm_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid WASM path"))?;

    let output = deps
        .command_executor
        .execute("wkg", &["oci", "push", &full_ref, wasm_path_str])
        .await?;

    if !output.success {
        spinner.finish_with_message(format!("✗ Failed to push to {full_ref}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to push component: {}", stderr);
    }

    spinner.finish_with_message(format!("✓ Pushed to {full_ref}"));

    deps.ui.print_styled(
        &format!("✓ Component published successfully to {full_ref}"),
        MessageStyle::Success,
    );

    deps.ui.print("");
    deps.ui.print("To use this component in a project:");
    deps.ui.print(&format!("  [component.{component_name}]"));
    deps.ui.print(&format!("  wasm = \"{full_ref}\""));

    Ok(())
}

/// Pull a component from a registry
///
/// # Arguments
/// * `deps` - Dependency injection container
/// * `component_ref` - Component reference (e.g., "mycomp:1.0.0" or "ghcr.io/org/comp:latest")
/// * `output` - Output path for the WASM file
/// * `force` - Overwrite existing file
pub async fn pull_with_deps(
    deps: &Arc<ComponentDependencies>,
    component_ref: &str,
    output: Option<&Path>,
    force: bool,
) -> Result<()> {
    deps.ui
        .print_styled("Pulling component from registry", MessageStyle::Cyan);

    // Resolve registry URL if using short reference
    let registry_url = get_default_registry(deps)?;
    let full_ref = if component_ref.contains('/')
        || component_ref.starts_with("ghcr.io")
        || component_ref.starts_with("docker.io")
    {
        component_ref.to_string()
    } else {
        // Apply default registry if available
        if let Some(default) = registry_url {
            format!("{default}/{component_ref}")
        } else {
            component_ref.to_string()
        }
    };

    deps.ui.print(&format!("Component: {full_ref}"));

    // Determine output path
    let output_path = if let Some(path) = output {
        path.to_path_buf()
    } else {
        // Default to component name with .wasm extension
        let name = extract_component_name(&full_ref);
        PathBuf::from(format!("{name}.wasm"))
    };

    // Check if file exists
    if deps.file_system.exists(&output_path) && !force {
        anyhow::bail!(
            "Output file already exists: {}. Use --force to overwrite.",
            output_path.display()
        );
    }

    // Pull from registry using wkg
    deps.ui
        .print(&format!("Saving to: {}", output_path.display()));

    let output_path_str = output_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid output path"))?;

    let output = deps
        .command_executor
        .execute("wkg", &["oci", "pull", &full_ref, "-o", output_path_str])
        .await?;

    if !output.success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to pull component: {}", stderr);
    }

    deps.ui.print_styled(
        &format!(
            "✓ Component pulled successfully to {}",
            output_path.display()
        ),
        MessageStyle::Success,
    );

    Ok(())
}

/// List components in a registry
///
/// # Arguments
/// * `deps` - Dependency injection container
/// * `repository` - Repository to list (e.g., "myorg/mycomponent")
/// * `registry` - Registry override
pub async fn list_with_deps(
    deps: &Arc<ComponentDependencies>,
    repository: &str,
    registry: Option<&str>,
) -> Result<()> {
    deps.ui
        .print_styled("Listing component versions", MessageStyle::Cyan);

    // Resolve registry URL
    let registry_url = if let Some(r) = registry {
        r.to_string()
    } else {
        get_default_registry(deps)?.unwrap_or_else(|| "ghcr.io".to_string())
    };

    // Construct full repository path
    let full_repo = if repository.contains('/') {
        format!("{registry_url}/{repository}")
    } else {
        // Assume it's under the default org
        format!("{registry_url}/{repository}")
    };

    deps.ui.print(&format!("Repository: {full_repo}"));
    deps.ui.print("");

    // List tags using crane
    let output = deps
        .command_executor
        .execute("crane", &["ls", &full_repo])
        .await?;

    if !output.success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list tags: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tags: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    if tags.is_empty() {
        deps.ui
            .print_styled("No versions found", MessageStyle::Yellow);
        return Ok(());
    }

    deps.ui.print("Available versions:");
    for tag in &tags {
        deps.ui.print(&format!("  - {tag}"));
    }

    deps.ui.print("");
    deps.ui.print(&format!("Total: {} versions", tags.len()));

    Ok(())
}

/// Inspect a component's metadata
///
/// # Arguments
/// * `deps` - Dependency injection container
/// * `component_ref` - Component reference to inspect
pub async fn inspect_with_deps(
    deps: &Arc<ComponentDependencies>,
    component_ref: &str,
) -> Result<()> {
    deps.ui
        .print_styled("Inspecting component", MessageStyle::Cyan);

    // Resolve registry URL if using short reference
    let registry_url = get_default_registry(deps)?;
    let full_ref = if component_ref.contains('/')
        || component_ref.starts_with("ghcr.io")
        || component_ref.starts_with("docker.io")
    {
        component_ref.to_string()
    } else {
        // Apply default registry if available
        if let Some(default) = registry_url {
            format!("{default}/{component_ref}")
        } else {
            component_ref.to_string()
        }
    };

    deps.ui.print(&format!("Component: {full_ref}"));
    deps.ui.print("");

    // Verify component exists using crane
    let output = deps
        .command_executor
        .execute("crane", &["manifest", &full_ref])
        .await?;

    if output.success {
        deps.ui
            .print_styled("✓ Component exists", MessageStyle::Success);
        deps.ui.print("");
        deps.ui.print("To pull this component:");
        deps.ui
            .print(&format!("  ftl component pull {component_ref}"));
    } else {
        deps.ui
            .print_styled("✗ Component not found", MessageStyle::Red);

        // Try to list available versions
        let repo = extract_repository(&full_ref);
        {
            let list_output = deps
                .command_executor
                .execute("crane", &["ls", &repo])
                .await?;

            if list_output.success {
                let stdout = String::from_utf8_lossy(&list_output.stdout);
                let tags: Vec<&str> = stdout
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .take(5)
                    .collect();

                if !tags.is_empty() {
                    deps.ui.print("");
                    deps.ui.print("Available versions:");
                    for tag in &tags {
                        deps.ui.print(&format!("  - {tag}"));
                    }

                    let total_lines = stdout.lines().filter(|l| !l.trim().is_empty()).count();
                    if total_lines > 5 {
                        deps.ui
                            .print(&format!("  ... and {} more", total_lines - 5));
                    }
                }
            }
        }
    }

    Ok(())
}

// Helper functions

fn resolve_wasm_path(deps: &Arc<ComponentDependencies>, component_path: &Path) -> Result<PathBuf> {
    if component_path.extension().and_then(|s| s.to_str()) == Some("wasm") {
        // Direct WASM file specified
        Ok(component_path.to_path_buf())
    } else {
        // It's a directory, first check ftl.toml for the wasm field
        let ftl_toml_path = PathBuf::from("ftl.toml");
        if deps.file_system.exists(&ftl_toml_path) {
            let ftl_content = deps.file_system.read_to_string(&ftl_toml_path)?;
            let ftl_config = crate::config::ftl_config::FtlConfig::parse(&ftl_content)?;

            // Find the tool that matches this component path
            let component_name = component_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            for (component_config_name, component_config) in &ftl_config.component {
                // Match by name or path
                if (component_config_name == component_name
                    || component_config.path.as_deref()
                        == Some(component_path.to_str().unwrap_or("")))
                    && let Some(wasm_path) = &component_config.wasm
                {
                    let full_wasm_path = PathBuf::from(wasm_path);
                    if deps.file_system.exists(&full_wasm_path) {
                        return Ok(full_wasm_path);
                    }
                }
            }
        }

        // Fallback to looking in common locations
        let candidates = vec![
            component_path.join("target/wasm32-wasip1/release"),
            component_path.join("target/wasm32-wasi/release"),
            component_path.join("build"),
            component_path.to_path_buf(),
        ];

        for candidate_dir in candidates {
            if deps.file_system.exists(&candidate_dir) {
                // Look for first .wasm file in directory
                let dir_name = component_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("component");

                let wasm_file = candidate_dir.join(format!("{}.wasm", dir_name.replace('-', "_")));
                if deps.file_system.exists(&wasm_file) {
                    return Ok(wasm_file);
                }

                // Also try without underscore conversion
                let wasm_file = candidate_dir.join(format!("{dir_name}.wasm"));
                if deps.file_system.exists(&wasm_file) {
                    return Ok(wasm_file);
                }
            }
        }

        anyhow::bail!(
            "No WASM file found for {}. Build the component first or specify the WASM file directly.",
            component_path.display()
        )
    }
}

fn derive_component_name(path: &Path) -> Result<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| anyhow::anyhow!("Cannot derive component name from path"))
}

fn resolve_registry_url(
    deps: &Arc<ComponentDependencies>,
    registry: Option<&str>,
) -> Result<String> {
    if let Some(r) = registry {
        Ok(r.to_string())
    } else if let Some(default) = get_default_registry(deps)? {
        Ok(default)
    } else {
        anyhow::bail!("No registry specified and no default_registry in ftl.toml")
    }
}

fn get_default_registry(deps: &Arc<ComponentDependencies>) -> Result<Option<String>> {
    let ftl_path = Path::new("ftl.toml");

    if !deps.file_system.exists(ftl_path) {
        return Ok(None);
    }

    let content = deps
        .file_system
        .read_to_string(ftl_path)
        .context("Failed to read ftl.toml")?;
    let config = crate::config::ftl_config::FtlConfig::parse(&content)
        .context("Failed to parse ftl.toml")?;

    Ok(config.project.default_registry)
}

fn extract_component_name(component_ref: &str) -> String {
    // Extract name from references like "ghcr.io/org/name:tag"
    let without_tag = component_ref.split(':').next().unwrap_or(component_ref);
    let name = without_tag.split('/').next_back().unwrap_or(without_tag);
    name.to_string()
}

fn extract_repository(component_ref: &str) -> String {
    // Extract repository from references like "ghcr.io/org/name:tag"
    if let Some(pos) = component_ref.rfind(':') {
        component_ref[..pos].to_string()
    } else {
        component_ref.to_string()
    }
}

// Command execution wrappers

/// Component command actions
#[derive(Debug, Clone)]
pub enum ComponentAction {
    /// Publish a component to a registry
    Publish {
        /// Path to component or WASM file
        path: PathBuf,
        /// Registry URL override
        registry: Option<String>,
        /// Component name override
        name: Option<String>,
        /// Version tag
        tag: Option<String>,
        /// Skip confirmation
        yes: bool,
    },
    /// Pull a component from a registry
    Pull {
        /// Component reference
        component: String,
        /// Output path
        output: Option<PathBuf>,
        /// Overwrite existing file
        force: bool,
    },
    /// List component versions in a registry
    List {
        /// Repository to list
        repository: String,
        /// Registry override
        registry: Option<String>,
    },
    /// Inspect a component
    Inspect {
        /// Component reference
        component: String,
    },
}

/// Execute component command
pub async fn execute(action: ComponentAction) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_runtime::deps::{RealCommandExecutor, RealFileSystem};

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(ComponentDependencies {
        ui: ui.clone(),
        file_system: Arc::new(RealFileSystem),
        command_executor: Arc::new(RealCommandExecutor),
    });

    match action {
        ComponentAction::Publish {
            path,
            registry,
            name,
            tag,
            yes,
        } => {
            publish_with_deps(
                &deps,
                &path,
                registry.as_deref(),
                name.as_deref(),
                tag.as_deref(),
                yes,
            )
            .await
        }
        ComponentAction::Pull {
            component,
            output,
            force,
        } => pull_with_deps(&deps, &component, output.as_deref(), force).await,
        ComponentAction::List {
            repository,
            registry,
        } => list_with_deps(&deps, &repository, registry.as_deref()).await,
        ComponentAction::Inspect { component } => inspect_with_deps(&deps, &component).await,
    }
}

#[cfg(test)]
#[path = "component_tests.rs"]
mod tests;
