//! Refactored registry command with dependency injection for better testability

use std::sync::Arc;

use anyhow::Result;
use ftl_runtime::deps::{MessageStyle, UserInterface};
use reqwest::Client;

use crate::registry::{get_registry_adapter, RegistryAdapter};

/// Dependencies for the registry command
pub struct RegistryDependencies {
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// HTTP client for registry operations
    pub client: Client,
}

/// Execute the list subcommand with injected dependencies
pub async fn list_with_deps(registry: Option<&str>, deps: &Arc<RegistryDependencies>) -> Result<()> {
    let registry_name = registry.unwrap_or("ghcr");
    
    deps.ui.print(&format!(
        "{} Listing components from {}",
        styled_text("→", MessageStyle::Cyan),
        styled_text(registry_name, MessageStyle::Bold)
    ));
    deps.ui.print("");

    // Get the registry adapter
    match get_registry_adapter(registry) {
        Ok(adapter) => {
            deps.ui.print(&format!(
                "{} Using {} registry",
                styled_text("ℹ", MessageStyle::Cyan),
                adapter.name()
            ));
            deps.ui.print("");
            
            // For now, show guidance since crane ls needs specific image names
            deps.ui.print(&format!(
                "{} Registry listing requires crane CLI and specific image names",
                styled_text("!", MessageStyle::Yellow)
            ));
            deps.ui.print("");
            deps.ui.print("To list tags for a specific image, use:");
            deps.ui.print(&format!("  crane ls {}", adapter.get_registry_url("IMAGE_NAME")));
            deps.ui.print("");
            deps.ui.print("Browse components at:");
            
            match registry_name {
                "ghcr" => {
                    deps.ui.print("  - GitHub Container Registry: https://github.com/orgs/fastertools/packages");
                }
                "docker" => {
                    deps.ui.print("  - Docker Hub: https://hub.docker.com/");
                }
                _ => {
                    deps.ui.print(&format!("  - Registry: {}", registry_name));
                }
            }
        }
        Err(e) => {
            deps.ui.print(&format!(
                "{} Error: {}",
                styled_text("✗", MessageStyle::Red),
                e
            ));
            return Err(e);
        }
    }
    
    Ok(())
}

/// Execute the search subcommand with injected dependencies
pub async fn search_with_deps(query: &str, registry: Option<&str>, deps: &Arc<RegistryDependencies>) -> Result<()> {
    let registry_name = registry.unwrap_or("ghcr");

    deps.ui.print(&format!(
        "{} Searching for '{}' in {}",
        styled_text("→", MessageStyle::Cyan),
        styled_text(query, MessageStyle::Bold),
        registry_name
    ));
    deps.ui.print("");

    // Get the registry adapter
    match get_registry_adapter(registry) {
        Ok(adapter) => {
            deps.ui.print(&format!(
                "{} Using {} registry",
                styled_text("ℹ", MessageStyle::Cyan),
                adapter.name()
            ));
            deps.ui.print("");
            
            deps.ui.print(&format!(
                "{} Registry search not yet implemented via crane",
                styled_text("!", MessageStyle::Yellow)
            ));
            deps.ui.print("");
            deps.ui.print("For now, you can search at:");
            
            match registry_name {
                "ghcr" => {
                    deps.ui.print(&format!(
                        "  - GitHub Packages: https://github.com/search?q={}&type=registrypackages"
                    , query));
                }
                "docker" => {
                    deps.ui.print(&format!(
                        "  - Docker Hub: https://hub.docker.com/search?q={}"
                    , query));
                }
                _ => {
                    deps.ui.print(&format!("  - Search manually in {} registry", registry_name));
                }
            }
        }
        Err(e) => {
            deps.ui.print(&format!(
                "{} Error: {}",
                styled_text("✗", MessageStyle::Red),
                e
            ));
            return Err(e);
        }
    }
    
    Ok(())
}

/// Execute the info subcommand with injected dependencies
pub async fn info_with_deps(component: &str, deps: &Arc<RegistryDependencies>) -> Result<()> {
    deps.ui.print(&format!(
        "{} Getting info for component: {}",
        styled_text("→", MessageStyle::Cyan),
        styled_text(component, MessageStyle::Bold)
    ));
    deps.ui.print("");

    // Try to determine registry from component name or use default
    let registry = if component.contains("ghcr.io") {
        Some("ghcr")
    } else if component.contains("docker.io") {
        Some("docker")
    } else {
        None // Use default
    };

    match get_registry_adapter(registry) {
        Ok(adapter) => {
            deps.ui.print(&format!(
                "{} Using {} registry",
                styled_text("ℹ", MessageStyle::Cyan),
                adapter.name()
            ));
            deps.ui.print("");
            
            // Check if component exists
            deps.ui.print(&format!(
                "{} Checking if component exists...",
                styled_text("→", MessageStyle::Cyan)
            ));
            
            match adapter.verify_image_exists(&deps.client, component).await {
                Ok(true) => {
                    deps.ui.print(&format!(
                        "{} Component exists in registry",
                        styled_text("✓", MessageStyle::Green)
                    ));
                    
                    // Try to get registry components for more info
                    match adapter.get_registry_components(&deps.client, component).await {
                        Ok(components) => {
                            deps.ui.print("");
                            deps.ui.print("Component details:");
                            deps.ui.print(&format!("  Registry: {}", components.registry_domain));
                            deps.ui.print(&format!("  Package:  {}", components.package_name));
                            deps.ui.print(&format!("  Version:  {}", components.version));
                        }
                        Err(e) => {
                            deps.ui.print(&format!(
                                "{} Could not get component details: {}",
                                styled_text("!", MessageStyle::Yellow),
                                e
                            ));
                        }
                    }
                }
                Ok(false) => {
                    deps.ui.print(&format!(
                        "{} Component not found in registry",
                        styled_text("✗", MessageStyle::Red)
                    ));
                }
                Err(e) => {
                    deps.ui.print(&format!(
                        "{} Error checking component: {}",
                        styled_text("!", MessageStyle::Yellow),
                        e
                    ));
                }
            }
            
            deps.ui.print("");
            deps.ui.print("Component reference formats:");
            deps.ui.print("  - ghcr.io/username/component:version");
            deps.ui.print("  - docker.io/username/component:version");
            deps.ui.print("  - component-name (searches default registry)");
        }
        Err(e) => {
            deps.ui.print(&format!(
                "{} Error: {}",
                styled_text("✗", MessageStyle::Red),
                e
            ));
            return Err(e);
        }
    }
    
    Ok(())
}

// Helper function to format styled text (since we're not using console crate directly)
const fn styled_text(text: &str, _style: MessageStyle) -> &str {
    text
}

/// Registry command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct RegistryArgs {
    /// Subcommand
    pub command: RegistryCommand,
}

/// Registry subcommands
#[derive(Debug, Clone)]
pub enum RegistryCommand {
    /// List available components
    List {
        /// Registry URL
        registry: Option<String>,
    },
    /// Search for components
    Search {
        /// Search query
        query: String,
        /// Registry URL
        registry: Option<String>,
    },
    /// Get info about a component
    Info {
        /// Component name
        component: String,
    },
}

/// Execute the registry command with default dependencies
pub async fn execute(args: RegistryArgs) -> Result<()> {
    use ftl_common::RealUserInterface;

    let ui = Arc::new(RealUserInterface);
    let client = Client::new();
    let deps = Arc::new(RegistryDependencies { ui: ui.clone(), client });

    match args.command {
        RegistryCommand::List { registry } => {
            list_with_deps(registry.as_deref(), &deps).await
        }
        RegistryCommand::Search { query, registry } => {
            search_with_deps(&query, registry.as_deref(), &deps).await
        }
        RegistryCommand::Info { component } => {
            info_with_deps(&component, &deps).await
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
