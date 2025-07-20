//! Refactored registry command with dependency injection for better testability

use std::sync::Arc;

use anyhow::Result;

use crate::deps::{UserInterface, MessageStyle};

/// Dependencies for the registry command
pub struct RegistryDependencies {
    pub ui: Arc<dyn UserInterface>,
}

/// Execute the list subcommand with injected dependencies
pub async fn list_with_deps(
    registry: Option<String>,
    deps: Arc<RegistryDependencies>,
) -> Result<()> {
    let registry_url = registry.as_deref().unwrap_or("ghcr.io");

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
    deps.ui.print("  - GitHub Container Registry: https://github.com/orgs/YOUR_ORG/packages");
    deps.ui.print("  - Docker Hub: https://hub.docker.com/");

    Ok(())
}

/// Execute the search subcommand with injected dependencies
pub async fn search_with_deps(
    query: String,
    registry: Option<String>,
    deps: Arc<RegistryDependencies>,
) -> Result<()> {
    let registry_url = registry.as_deref().unwrap_or("ghcr.io");

    deps.ui.print(&format!(
        "{} Searching for '{}' in {}",
        styled_text("→", MessageStyle::Cyan),
        styled_text(&query, MessageStyle::Bold),
        registry_url
    ));

    deps.ui.print("");
    deps.ui.print(&format!(
        "{} Registry search not yet implemented",
        styled_text("!", MessageStyle::Yellow)
    ));
    deps.ui.print("");
    deps.ui.print("For now, you can search at:");
    deps.ui.print(&format!("  - GitHub: https://github.com/search?q=mcp+{query}&type=registrypackages"));

    Ok(())
}

/// Execute the info subcommand with injected dependencies
pub async fn info_with_deps(
    component: String,
    deps: Arc<RegistryDependencies>,
) -> Result<()> {
    deps.ui.print(&format!(
        "{} Getting info for component: {}",
        styled_text("→", MessageStyle::Cyan),
        styled_text(&component, MessageStyle::Bold)
    ));

    deps.ui.print("");
    deps.ui.print(&format!("{} Registry info not yet implemented", styled_text("!", MessageStyle::Yellow)));
    deps.ui.print("");
    deps.ui.print("Component reference formats:");
    deps.ui.print("  - ghcr.io/username/component:version");
    deps.ui.print("  - docker.io/username/component:version");
    deps.ui.print("  - component-name (searches default registry)");

    Ok(())
}

// Helper function to format styled text (since we're not using console crate directly)
fn styled_text(text: &str, _style: MessageStyle) -> &str {
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_text() {
        assert_eq!(styled_text("test", MessageStyle::Success), "test");
    }
}