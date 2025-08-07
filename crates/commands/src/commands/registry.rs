//! Registry command for managing container registries

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use ftl_runtime::deps::{FileSystem, UserInterface};
use toml_edit::{DocumentMut, Item, Table};

/// Dependencies for the registry command
pub struct RegistryDependencies {
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// File system for reading/writing config
    pub file_system: Arc<dyn FileSystem>,
}

/// List configured registries or show available registry types
pub fn list_with_deps(deps: &Arc<RegistryDependencies>) -> Result<()> {
    // Try to read ftl.toml to show current configuration
    let ftl_path = Path::new("ftl.toml");

    if deps.file_system.exists(ftl_path) {
        let content = deps.file_system.read_to_string(ftl_path)?;
        let doc: DocumentMut = content.parse()?;

        if let Some(project) = doc.get("project") {
            if let Some(default_registry) = project.get("default_registry") {
                deps.ui.print("Current registry configuration:");
                deps.ui.print("");
                // Extract the string value without quotes
                let registry_str = if let Some(s) = default_registry.as_str() {
                    s.to_string()
                } else {
                    default_registry.to_string().trim_matches('"').to_string()
                };
                deps.ui
                    .print(&format!("  Default registry: {registry_str}"));
                deps.ui.print("");
            } else {
                deps.ui.print("No default registry configured.");
                deps.ui.print("");
            }
        }
    } else {
        deps.ui.print("No ftl.toml found in current directory.");
        deps.ui.print("");
    }

    deps.ui.print("Available registry types:");
    deps.ui
        .print("  • ghcr.io         - GitHub Container Registry");
    deps.ui.print("  • docker.io       - Docker Hub");
    deps.ui
        .print("  • Custom URL      - Any OCI-compatible registry");
    deps.ui.print("");
    deps.ui.print("Authentication:");
    deps.ui
        .print("  Use 'docker login' to authenticate with any registry");

    Ok(())
}

/// Set the default registry in ftl.toml
pub fn set_default_registry(deps: &Arc<RegistryDependencies>, registry_url: &str) -> Result<()> {
    let ftl_path = Path::new("ftl.toml");

    if !deps.file_system.exists(ftl_path) {
        return Err(anyhow::anyhow!("No ftl.toml found. Run 'ftl init' first."));
    }

    // Read and parse the existing file
    let content = deps.file_system.read_to_string(ftl_path)?;
    let mut doc: DocumentMut = content.parse()?;

    // Ensure project section exists
    if doc.get("project").is_none() {
        doc["project"] = Item::Table(Table::default());
    }

    // Set the default_registry field
    doc["project"]["default_registry"] = Item::Value(registry_url.into());

    // Write back to file
    deps.file_system.write_string(ftl_path, &doc.to_string())?;

    deps.ui
        .print(&format!("✓ Default registry set to: {registry_url}"));
    deps.ui.print("");
    deps.ui.print("You can now use short component names:");
    deps.ui
        .print("  my-component:1.0.0  # Will resolve to {}/my-component:1.0.0");

    Ok(())
}

/// Remove the default registry from ftl.toml
pub fn remove_default_registry(deps: &Arc<RegistryDependencies>) -> Result<()> {
    let ftl_path = Path::new("ftl.toml");

    if !deps.file_system.exists(ftl_path) {
        return Err(anyhow::anyhow!("No ftl.toml found."));
    }

    // Read and parse the existing file
    let content = deps.file_system.read_to_string(ftl_path)?;
    let mut doc: DocumentMut = content.parse()?;

    // Remove the default_registry field if it exists
    if let Some(project) = doc.get_mut("project") {
        if let Some(table) = project.as_table_mut() {
            table.remove("default_registry");
        }
    }

    // Write back to file
    deps.file_system.write_string(ftl_path, &doc.to_string())?;

    deps.ui.print("✓ Default registry removed");
    deps.ui
        .print("  Components will now require full registry URLs");

    Ok(())
}

/// Execute registry command
pub fn execute(action: RegistryAction) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_runtime::deps::RealFileSystem;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(RegistryDependencies {
        ui: ui.clone(),
        file_system: Arc::new(RealFileSystem),
    });

    match action {
        RegistryAction::List => list_with_deps(&deps),
        RegistryAction::Set { url } => set_default_registry(&deps, &url),
        RegistryAction::Remove => remove_default_registry(&deps),
    }
}

/// Registry command actions
#[derive(Debug, Clone)]
pub enum RegistryAction {
    /// List registries or show current configuration
    List,
    /// Set the default registry
    Set {
        /// Registry URL (e.g., "ghcr.io/myorg")
        url: String,
    },
    /// Remove the default registry
    Remove,
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
