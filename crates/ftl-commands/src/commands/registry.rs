//! Refactored registry command with dependency injection for better testability

use std::sync::Arc;

use anyhow::Result;
use ftl_core::deps::{MessageStyle, UserInterface};

/// Dependencies for the registry command
pub struct RegistryDependencies {
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
}

/// Execute the list subcommand with injected dependencies
pub fn list_with_deps(registry: Option<&str>, deps: &Arc<RegistryDependencies>) {
    let registry_url = registry.unwrap_or("ghcr.io");

    deps.ui.print(&format!(
        "{} Listing components from {}",
        styled_text("→", MessageStyle::Cyan),
        styled_text(registry_url, MessageStyle::Bold)
    ));

    deps.ui.print("");
    deps.ui.print(&format!(
        "{} Registry listing not yet implemented",
        styled_text("!", MessageStyle::Yellow)
    ));
    deps.ui.print("");
    deps.ui.print("For now, you can browse components at:");
    deps.ui
        .print("  - GitHub Container Registry: https://github.com/orgs/YOUR_ORG/packages");
    deps.ui.print("  - Docker Hub: https://hub.docker.com/");
}

/// Execute the search subcommand with injected dependencies
pub fn search_with_deps(query: &str, registry: Option<&str>, deps: &Arc<RegistryDependencies>) {
    let registry_url = registry.unwrap_or("ghcr.io");

    deps.ui.print(&format!(
        "{} Searching for '{}' in {}",
        styled_text("→", MessageStyle::Cyan),
        styled_text(query, MessageStyle::Bold),
        registry_url
    ));

    deps.ui.print("");
    deps.ui.print(&format!(
        "{} Registry search not yet implemented",
        styled_text("!", MessageStyle::Yellow)
    ));
    deps.ui.print("");
    deps.ui.print("For now, you can search at:");
    deps.ui.print(&format!(
        "  - GitHub: https://github.com/search?q=mcp+{query}&type=registrypackages"
    ));
}

/// Execute the info subcommand with injected dependencies
pub fn info_with_deps(component: &str, deps: &Arc<RegistryDependencies>) {
    deps.ui.print(&format!(
        "{} Getting info for component: {}",
        styled_text("→", MessageStyle::Cyan),
        styled_text(component, MessageStyle::Bold)
    ));

    deps.ui.print("");
    deps.ui.print(&format!(
        "{} Registry info not yet implemented",
        styled_text("!", MessageStyle::Yellow)
    ));
    deps.ui.print("");
    deps.ui.print("Component reference formats:");
    deps.ui.print("  - ghcr.io/username/component:version");
    deps.ui.print("  - docker.io/username/component:version");
    deps.ui
        .print("  - component-name (searches default registry)");
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
#[allow(clippy::unused_async)]
pub async fn execute(args: RegistryArgs) -> Result<()> {
    use ftl_common::RealUserInterface;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(RegistryDependencies { ui: ui.clone() });

    match args.command {
        RegistryCommand::List { registry } => {
            list_with_deps(registry.as_deref(), &deps);
            Ok(())
        }
        RegistryCommand::Search { query, registry } => {
            search_with_deps(&query, registry.as_deref(), &deps);
            Ok(())
        }
        RegistryCommand::Info { component } => {
            info_with_deps(&component, &deps);
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
