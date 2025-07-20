//! Refactored registry command with dependency injection for better testability

use std::sync::Arc;

use crate::deps::{MessageStyle, UserInterface};

/// Dependencies for the registry command
pub struct RegistryDependencies {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_text() {
        assert_eq!(styled_text("test", MessageStyle::Success), "test");
    }
}
