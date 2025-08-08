//! Component resolver for parallel downloading of registry components
//!
//! This module handles resolving registry references to local WASM files,
//! downloading components in parallel when needed with progress indicators.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use crate::config::ftl_config::FtlConfig;
use ftl_runtime::deps::{MessageStyle, UserInterface};

/// Strategy for resolving components based on command needs
#[derive(Debug, Clone)]
pub enum ComponentResolutionStrategy {
    /// Local execution (ftl up, ftl run)
    Local {
        /// Whether to include MCP system components
        include_mcp: bool,
        /// Whether to include user tool components
        include_user: bool,
    },
    /// Deployment (ftl deploy)
    Deploy {
        /// Only resolve user components for pushing (MCP provided by platform)
        push_user_only: bool,
    },
    /// No resolution needed
    None,
}

/// A component that needs to be resolved
#[derive(Debug, Clone)]
pub struct ComponentToResolve {
    /// Name of the component
    pub name: String,
    /// Registry reference or local path
    pub source: String,
    /// Whether this is an MCP system component
    pub is_mcp: bool,
}

/// Result of component resolution
#[derive(Debug, Clone)]
pub struct ResolvedComponents {
    /// Mapping from component name to local path
    pub mappings: HashMap<String, PathBuf>,
    /// Temporary directory containing downloaded components (must be kept alive)
    _temp_dir: Arc<TempDir>,
}

impl ResolvedComponents {
    /// Get the local path for a component
    pub fn get_path(&self, name: &str) -> Option<&PathBuf> {
        self.mappings.get(name)
    }

    /// Get all resolved mappings
    pub const fn mappings(&self) -> &HashMap<String, PathBuf> {
        &self.mappings
    }
}

/// Component resolver for parallel downloads with progress
pub struct ComponentResolver {
    /// Maximum concurrent downloads
    max_concurrent: usize,
    /// User interface for progress display
    ui: Arc<dyn UserInterface>,
}

impl ComponentResolver {
    /// Create a new component resolver with UI
    pub fn new(ui: Arc<dyn UserInterface>) -> Self {
        Self {
            max_concurrent: 4,
            ui,
        }
    }

    /// Create a resolver with custom concurrency limit
    pub fn with_concurrency(ui: Arc<dyn UserInterface>, max_concurrent: usize) -> Self {
        Self { max_concurrent, ui }
    }

    /// Resolve components based on strategy
    pub async fn resolve_components(
        &self,
        ftl_config: &FtlConfig,
        strategy: ComponentResolutionStrategy,
    ) -> Result<ResolvedComponents> {
        // Identify which components need resolving
        let components = Self::identify_components(ftl_config, &strategy);

        if components.is_empty() {
            // No components to resolve, return empty result with temp dir
            let temp_dir = Arc::new(
                tempfile::Builder::new()
                    .prefix("ftl-components-")
                    .tempdir()
                    .context("Failed to create temporary directory")?,
            );
            return Ok(ResolvedComponents {
                mappings: HashMap::new(),
                _temp_dir: temp_dir,
            });
        }

        // Create temporary directory for downloads
        let temp_dir = Arc::new(
            tempfile::Builder::new()
                .prefix("ftl-components-")
                .tempdir()
                .context("Failed to create temporary directory")?,
        );

        // Resolve components in parallel with progress
        let resolved = self
            .resolve_parallel_with_progress(components, temp_dir.clone())
            .await?;

        Ok(ResolvedComponents {
            mappings: resolved,
            _temp_dir: temp_dir,
        })
    }

    /// Identify components that need resolution based on strategy
    fn identify_components(
        ftl_config: &FtlConfig,
        strategy: &ComponentResolutionStrategy,
    ) -> Vec<ComponentToResolve> {
        let mut components = Vec::new();

        match strategy {
            ComponentResolutionStrategy::Local {
                include_mcp,
                include_user,
            } => {
                // Add MCP components if requested
                if *include_mcp {
                    // Only add if they're registry references (not local .wasm files)
                    if !ftl_config.mcp.gateway.to_lowercase().ends_with(".wasm") {
                        components.push(ComponentToResolve {
                            name: "mcp-gateway".to_string(),
                            source: ftl_config.mcp.gateway.clone(),
                            is_mcp: true,
                        });
                    }

                    // Add authorizer if auth is enabled
                    if ftl_config.is_auth_enabled()
                        && !ftl_config.mcp.authorizer.to_lowercase().ends_with(".wasm")
                    {
                        components.push(ComponentToResolve {
                            name: "mcp-authorizer".to_string(),
                            source: ftl_config.mcp.authorizer.clone(),
                            is_mcp: true,
                        });
                    }
                }

                // Add user components if requested
                if *include_user {
                    for (component_name, component_config) in &ftl_config.component {
                        if let Some(repo_ref) = &component_config.repo {
                            // This is a registry component
                            components.push(ComponentToResolve {
                                name: component_name.clone(),
                                source: repo_ref.clone(),
                                is_mcp: false,
                            });
                        }
                        // Skip local components (those with wasm field)
                    }
                }
            }
            ComponentResolutionStrategy::Deploy { push_user_only } => {
                if *push_user_only {
                    // Only resolve user registry components (not MCP, not local)
                    for (component_name, component_config) in &ftl_config.component {
                        if let Some(repo_ref) = &component_config.repo {
                            // This is a registry component that needs to be pushed
                            components.push(ComponentToResolve {
                                name: component_name.clone(),
                                source: repo_ref.clone(),
                                is_mcp: false,
                            });
                        }
                        // Skip local components - they don't need pulling for deploy
                    }
                }
            }
            ComponentResolutionStrategy::None => {
                // No components needed
            }
        }

        components
    }

    /// Resolve components in parallel with progress indicators
    async fn resolve_parallel_with_progress(
        &self,
        components: Vec<ComponentToResolve>,
        temp_dir: Arc<TempDir>,
    ) -> Result<HashMap<String, PathBuf>> {
        if components.is_empty() {
            return Ok(HashMap::new());
        }

        self.ui.print("");
        self.ui.print(&format!(
            "→ Pulling {} component{} in parallel",
            components.len(),
            if components.len() == 1 { "" } else { "s" }
        ));
        self.ui.print("");

        let multi_progress = self.ui.create_multi_progress();
        let mut tasks = JoinSet::new();
        let resolved_components = Arc::new(Mutex::new(HashMap::new()));

        // Track errors across all tasks
        let error_flag = Arc::new(Mutex::new(None::<String>));

        // Limit concurrent operations
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));

        for component in components {
            let pb = multi_progress.add_spinner();
            pb.set_prefix(format!("[{}]", component.name));
            pb.set_message("Starting pull...");
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let temp_dir = temp_dir.clone();
            let resolved_components = Arc::clone(&resolved_components);
            let error_flag = Arc::clone(&error_flag);
            let semaphore = Arc::clone(&semaphore);
            let default_registry = None; // TODO: Get from ftl_config if needed

            tasks.spawn(async move {
                // Acquire permit to limit concurrency
                let _permit = semaphore.acquire().await.unwrap();

                // Check if another task has already failed
                if error_flag.lock().await.is_some() {
                    pb.finish_with_message("Skipped due to error".to_string());
                    return Ok(());
                }

                let start = Instant::now();

                // Resolve the registry URL
                let resolved_url =
                    crate::registry::resolve_registry_url(&component.source, default_registry);

                // Determine output path
                let wasm_filename = format!("{}.wasm", component.name);
                let wasm_path = temp_dir.path().join(&wasm_filename);

                // Update progress
                pb.set_message(&format!("Pulling from {resolved_url}"));

                // Pull the component
                match pull_component_async(&resolved_url, &wasm_path).await {
                    Ok(()) => {
                        // Add to resolved components
                        let mut resolved = resolved_components.lock().await;
                        resolved.insert(component.name.clone(), wasm_path);

                        let duration = start.elapsed();
                        pb.finish_with_message(format!(
                            "✓ Pulled in {:.1}s",
                            duration.as_secs_f64()
                        ));
                        Ok(())
                    }
                    Err(e) => {
                        pb.finish_with_message(format!("✗ Pull failed: {e}"));

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
            if let Err(e) = result?
                && first_error.is_none()
            {
                first_error = Some(e);
            }
        }

        // If any component failed, return the first error
        if let Some(e) = first_error {
            return Err(e);
        }

        let mappings = Arc::try_unwrap(resolved_components).unwrap().into_inner();

        self.ui.print("");
        self.ui.print_styled(
            "✓ All components pulled successfully!",
            MessageStyle::Success,
        );

        Ok(mappings)
    }
}

/// Async wrapper for pulling a component
async fn pull_component_async(registry_url: &str, output_path: &Path) -> Result<()> {
    // For now, we'll use tokio::process to run wkg asynchronously
    // In the future, this could be replaced with a native async registry client

    use tokio::process::Command;

    // Check wkg is available
    crate::registry::check_wkg_available()?;

    let output = Command::new("wkg")
        .args([
            "oci",
            "pull",
            registry_url,
            "-o",
            &output_path.to_string_lossy(),
        ])
        .output()
        .await
        .context("Failed to execute wkg oci pull")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to pull component: {}", stderr);
    }

    Ok(())
}
