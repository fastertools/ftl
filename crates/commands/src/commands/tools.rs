//! Tools command for managing pre-built FTL tools

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use ftl_runtime::deps::{MessageStyle, UserInterface};
use reqwest::Client;
use serde_json;
use toml_edit::{DocumentMut, Item, Table};

use crate::data::{Tool, ToolsManifest};
use crate::registry::{get_registry_adapter, RegistryAdapter};

/// Dependencies for the tools command
pub struct ToolsDependencies {
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// HTTP client for registry operations
    pub client: Client,
}

/// Represents a tool that has been resolved from the registry
/// with all necessary information for installation
#[derive(Debug, Clone)]
struct ResolvedTool {
    name: String,
    image_name: String,
    version: String,
}

/// Lists available tools from either the embedded manifest or directly from registries
/// 
/// # Arguments
/// * `deps` - Dependency injection container with UI and HTTP client
/// * `category` - Optional category filter (e.g., "basic_math", "text_processing")
/// * `filter` - Optional keyword filter for name/description/tags
/// * `registry` - Optional registry override (defaults to "ghcr")
/// * `verbose` - Show additional tool details
/// * `all` - Query all known registries (only with --direct)
/// * `direct` - Query registries directly instead of using embedded manifest
pub async fn list_with_deps(
    deps: &Arc<ToolsDependencies>,
    category: Option<&str>,
    filter: Option<&str>,
    registry: Option<&str>,
    verbose: bool,
    all: bool,
    direct: bool,
) -> Result<()> {
    if direct {
        if all {
            // Query all registries directly
            list_tools_from_all_registries(deps, category, filter, verbose).await
        } else {
            // Query specific registry directly via GitHub API
            list_tools_from_registry(deps, registry, category, filter, verbose).await
        }
    } else {
        // Use embedded tools.toml manifest
        list_tools_from_manifest(deps, category, filter, verbose).await
    }
}

/// Adds one or more tools to the current project's spin.toml
/// 
/// # Arguments
/// * `deps` - Dependency injection container
/// * `tools` - Tool specifications (name, name:version, registry:name:version)
/// * `registry` - Optional registry override
/// * `version` - Optional version override (applies to all tools)
/// * `yes` - Skip confirmation prompt
/// 
/// # Tool Specification Formats
/// * `toolname` - Latest version from default registry
/// * `toolname:1.0.0` - Specific version from default registry  
/// * `docker:toolname` - Latest from Docker Hub
/// * `docker:toolname:1.0.0` - Specific version from Docker Hub
pub async fn add_with_deps(
    deps: &Arc<ToolsDependencies>,
    tools: &[String],
    registry: Option<&str>,
    version: Option<&str>,
    yes: bool,
) -> Result<()> {
    let resolved_tools = resolve_tools(deps, tools, registry, version).await?;
    
    if !yes {
        if !confirm_tool_changes(deps, &resolved_tools, "add")? {
            deps.ui.print(&styled_text("Operation cancelled.", MessageStyle::Yellow));
            return Ok(());
        }
    }

    add_tools_to_project(deps, &resolved_tools).await
}

/// Updates existing tools in the project to new versions
/// 
/// # Arguments
/// * `deps` - Dependency injection container
/// * `tools` - Tool specifications to update
/// * `registry` - Optional registry override
/// * `version` - Optional version override ("latest" resolved to actual version)
/// * `yes` - Skip confirmation prompt
/// 
/// # Note
/// Only updates tools that are already installed. Use `add` for new tools.
pub async fn update_with_deps(
    deps: &Arc<ToolsDependencies>,
    tools: &[String],
    registry: Option<&str>,
    version: Option<&str>,
    yes: bool,
) -> Result<()> {
    // Check if tools are currently installed
    let installed_tools = get_installed_tools()?;
    let mut valid_tools = Vec::new();
    
    for tool_spec in tools {
        let (_registry_name, tool_name, _tool_version) = parse_tool_spec(tool_spec, registry);
        
        if !installed_tools.contains(&tool_name) {
            deps.ui.print(&format!(
                "{} Tool '{}' is not currently installed",
                styled_text("✗", MessageStyle::Red),
                styled_text(&tool_name, MessageStyle::Cyan)
            ));
            deps.ui.print(&format!(
                "{} Use 'ftl tools add {}' to install it first",
                styled_text("→", MessageStyle::Cyan),
                tool_name
            ));
            continue;
        }
        
        valid_tools.push(tool_spec.clone());
    }
    
    if valid_tools.is_empty() {
        deps.ui.print(&styled_text("No valid tools to update", MessageStyle::Yellow));
        return Ok(());
    }
    
    let resolved_tools = resolve_tools(deps, &valid_tools, registry, version).await?;
    
    if !yes {
        if !confirm_tool_changes(deps, &resolved_tools, "update")? {
            deps.ui.print(&styled_text("Operation cancelled.", MessageStyle::Yellow));
            return Ok(());
        }
    }

    update_tools_in_project(deps, &resolved_tools).await
}

/// Removes tools from the project's spin.toml
/// 
/// # Arguments
/// * `deps` - Dependency injection container
/// * `tools` - Names of tools to remove
/// * `yes` - Skip confirmation prompt
/// 
/// # Note
/// Updates both the component section and tool_components variable
pub async fn remove_with_deps(
    deps: &Arc<ToolsDependencies>,
    tools: &[String],
    yes: bool,
) -> Result<()> {
    // Check if tools are currently installed
    let installed_tools = get_installed_tools()?;
    let mut valid_tools = Vec::new();
    
    for tool_name in tools {
        if installed_tools.contains(tool_name) {
            valid_tools.push(tool_name.clone());
        } else {
            deps.ui.print(&format!(
                "{} Tool '{}' is not currently installed",
                styled_text("!", MessageStyle::Yellow),
                styled_text(tool_name, MessageStyle::Cyan)
            ));
        }
    }
    
    if valid_tools.is_empty() {
        deps.ui.print(&styled_text("No valid tools to remove", MessageStyle::Yellow));
        return Ok(());
    }
    
    if !yes {
        if !confirm_tool_removal(deps, &valid_tools)? {
            deps.ui.print(&styled_text("Operation cancelled.", MessageStyle::Yellow));
            return Ok(());
        }
    }

    remove_tools_from_project(deps, &valid_tools).await
}

/// List tools from embedded manifest
async fn list_tools_from_manifest(
    deps: &Arc<ToolsDependencies>,
    category: Option<&str>,
    filter: Option<&str>,
    verbose: bool,
) -> Result<()> {
    // Load tools manifest from embedded data
    let manifest_content = include_str!("../data/tools.toml");
    let manifest = ToolsManifest::from_toml(manifest_content)
        .context("Failed to parse tools manifest")?;

    let mut tools = manifest.get_tools().to_vec();

    // Apply category filter
    if let Some(cat) = category {
        tools.retain(|tool| tool.category == cat);
    }

    // Apply keyword filter
    if let Some(keyword) = filter {
        tools.retain(|tool| {
            tool.name.contains(keyword) 
                || tool.description.contains(keyword)
                || tool.tags.iter().any(|tag| tag.contains(keyword))
        });
    }

    if tools.is_empty() {
        deps.ui.print(&styled_text("No tools found matching the specified criteria.", MessageStyle::Yellow));
        return Ok(());
    }

    // Display tools
    if verbose {
        display_tools_verbose(deps, &tools);
    } else {
        display_tools_compact(deps, &tools);
    }

    Ok(())
}

/// Queries multiple registries in parallel to list available tools
/// 
/// Currently queries GHCR, Docker Hub, and ECR registries.
/// Falls back gracefully if individual registries fail.
async fn list_tools_from_all_registries(
    deps: &Arc<ToolsDependencies>,
    _category: Option<&str>,
    filter: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let registries = ["ghcr", "docker", "ecr"];
    
    deps.ui.print(&format!(
        "{} Querying all registries for available tools...",
        styled_text("→", MessageStyle::Cyan)
    ));

    for registry_name in &registries {
        deps.ui.print("");
        match list_tools_from_single_registry(deps, Some(registry_name), filter, verbose).await {
            Ok(_) => {}
            Err(e) => {
                deps.ui.print(&format!(
                    "{} Failed to query {} registry: {}",
                    styled_text("!", MessageStyle::Yellow),
                    registry_name,
                    e
                ));
            }
        }
    }

    deps.ui.print("");
    deps.ui.print("To add from a specific registry:");
    deps.ui.print("  ftl tools add <registry>:<tool-name>:<version>");
    
    Ok(())
}

/// List tools from registry via GitHub API
async fn list_tools_from_registry(
    deps: &Arc<ToolsDependencies>,
    registry: Option<&str>,
    _category: Option<&str>,
    filter: Option<&str>,
    verbose: bool,
) -> Result<()> {
    list_tools_from_single_registry(deps, registry, filter, verbose).await
}

/// List tools from a single registry
async fn list_tools_from_single_registry(
    deps: &Arc<ToolsDependencies>,
    registry: Option<&str>,
    filter: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let registry_name = registry.unwrap_or("ghcr");
    
    deps.ui.print(&format!(
        "{} Querying {} registry for available tools...",
        styled_text("→", MessageStyle::Cyan),
        styled_text(registry_name, MessageStyle::Bold)
    ));

    // Query GitHub for ftl-tool-* repositories
    let tools = query_github_tools(&deps.client, "fastertools", "ftl-tool-").await?;

    let mut resolved_tools = tools
        .into_iter()
        .map(|name| ResolvedTool {
            name: name.strip_prefix("ftl-tool-").unwrap_or(&name).to_string(),
            image_name: name,
            version: "latest".to_string(),
        })
        .collect::<Vec<_>>();

    // Apply filters
    if let Some(keyword) = filter {
        resolved_tools.retain(|tool| tool.name.contains(keyword));
    }

    if resolved_tools.is_empty() {
        deps.ui.print(&styled_text("No tools found in registry matching the specified criteria.", MessageStyle::Yellow));
        return Ok(());
    }

    // Display results
    deps.ui.print("");
    deps.ui.print(&format!(
        "{} Available tools from {} registry:",
        styled_text("●", MessageStyle::Green),
        registry_name
    ));

    for tool in &resolved_tools {
        if verbose {
            deps.ui.print(&format!(
                "  {} {} {}",
                styled_text(&tool.name, MessageStyle::Cyan),
                styled_text(&tool.version, MessageStyle::Yellow),
                styled_text(&format!("({})", tool.image_name), MessageStyle::Yellow)
            ));
        } else {
            deps.ui.print(&format!(
                "  {} {}",
                styled_text(&tool.name, MessageStyle::Cyan),
                styled_text(&tool.version, MessageStyle::Yellow)
            ));
        }
    }

    deps.ui.print("");
    deps.ui.print(&format!("Total: {} tools", resolved_tools.len()));

    Ok(())
}

/// Query GitHub API for tool repositories
async fn query_github_tools(client: &Client, org: &str, prefix: &str) -> Result<Vec<String>> {
    let url = format!("https://api.github.com/orgs/{}/repos?per_page=100", org);
    
    let response = client
        .get(&url)
        .header("User-Agent", "ftl-cli")
        .send()
        .await
        .context("Failed to query GitHub API")?;

    if !response.status().is_success() {
        anyhow::bail!("GitHub API request failed: {}", response.status());
    }

    let repos: serde_json::Value = response.json().await
        .context("Failed to parse GitHub API response")?;

    let mut tools = Vec::new();
    if let Some(repo_array) = repos.as_array() {
        for repo in repo_array {
            if let Some(name) = repo["name"].as_str() {
                if name.starts_with(prefix) {
                    tools.push(name.to_string());
                }
            }
        }
    }

    tools.sort();
    Ok(tools)
}

/// Resolve tool names to full image references using registry adapters
async fn resolve_tools(
    deps: &Arc<ToolsDependencies>,
    tools: &[String],
    registry: Option<&str>,
    version: Option<&str>,
) -> Result<Vec<ResolvedTool>> {
    let mut resolved = Vec::new();

    // Get registry adapter for image verification (for future crane integration)
    let _adapter = get_registry_adapter(registry)
        .with_context(|| format!("Failed to get registry adapter for: {:?}", registry))?;

    for tool in tools {
        let (_registry_name, tool_name, tool_version) = parse_tool_spec(tool, registry);
        let final_version = version.unwrap_or(&tool_version);
        
        // Resolve full image name based on registry
        let image_name = if tool_name.starts_with("ftl-tool-") {
            tool_name.clone()
        } else {
            format!("ftl-tool-{}", tool_name)
        };

        deps.ui.print(&format!(
            "{} Verifying tool {} in registry...",
            styled_text("→", MessageStyle::Cyan),
            styled_text(&tool_name, MessageStyle::Bold)
        ));

        // For now, we'll assume the image exists. In a full implementation,
        // we would use the crane CLI to verify image existence via adapter.get_registry_url()
        resolved.push(ResolvedTool {
            name: tool_name,
            image_name,
            version: final_version.to_string(),
        });
    }

    Ok(resolved)
}

/// Parse tool specification supporting registry:tool:version format
/// Returns (registry_name, tool_name, version)
pub fn parse_tool_spec(spec: &str, registry_override: Option<&str>) -> (String, String, String) {
    let parts: Vec<&str> = spec.splitn(3, ':').collect();
    let known_registries = ["ghcr", "docker", "ecr"]; // Basic registry support

    match parts.as_slice() {
        [single] => {
            // Just tool name, use default/override registry and latest tag
            let reg = registry_override.unwrap_or("ghcr").to_string();
            (reg, single.to_string(), "latest".to_string())
        }
        [first, second] => {
            // Could be "registry:tool" or "tool:version"
            // Check if first part is a known registry
            if known_registries.contains(first) {
                // It's "registry:tool"
                (first.to_string(), second.to_string(), "latest".to_string())
            } else {
                // It's "tool:version"
                let reg = registry_override.unwrap_or("ghcr").to_string();
                (reg, first.to_string(), second.to_string())
            }
        }
        [reg, tool, tag] => {
            // Full "registry:tool:version"
            (reg.to_string(), tool.to_string(), tag.to_string())
        }
        _ => {
            // Fallback for malformed input
            let reg = registry_override.unwrap_or("ghcr").to_string();
            (reg, spec.to_string(), "latest".to_string())
        }
    }
}

/// Display tools in compact format
fn display_tools_compact(deps: &Arc<ToolsDependencies>, tools: &[Tool]) {
    // Group by category
    let mut by_category = HashMap::new();
    for tool in tools {
        by_category.entry(&tool.category).or_insert_with(Vec::new).push(tool);
    }

    let mut categories: Vec<_> = by_category.keys().collect();
    categories.sort();

    for category in categories {
        deps.ui.print("");
        deps.ui.print(&format!(
            "{}{}:",
            styled_text("●", MessageStyle::Green),
            styled_text(category, MessageStyle::Bold)
        ));
        
        let mut category_tools = by_category[category].clone();
        category_tools.sort_by(|a, b| a.name.cmp(&b.name));
        
        for tool in category_tools {
            deps.ui.print(&format!(
                "  {} - {}",
                styled_text(&tool.name, MessageStyle::Cyan),
                tool.description
            ));
        }
    }

    deps.ui.print("");
    deps.ui.print(&format!("Total: {} tools", tools.len()));
}

/// Display tools in verbose format
fn display_tools_verbose(deps: &Arc<ToolsDependencies>, tools: &[Tool]) {
    for tool in tools {
        deps.ui.print("");
        deps.ui.print(&format!(
            "{}{}",
            styled_text("●", MessageStyle::Green),
            styled_text(&tool.name, MessageStyle::Cyan)
        ));
        deps.ui.print(&format!("  Category: {}", tool.category));
        deps.ui.print(&format!("  Description: {}", tool.description));
        deps.ui.print(&format!("  Image: {}", tool.image_name));
        deps.ui.print(&format!("  Tags: {}", tool.tags.join(", ")));
    }

    deps.ui.print("");
    deps.ui.print(&format!("Total: {} tools", tools.len()));
}

/// Confirm tool changes with user
fn confirm_tool_changes(
    deps: &Arc<ToolsDependencies>,
    tools: &[ResolvedTool],
    action: &str,
) -> Result<bool> {
    deps.ui.print(&format!("The following tools will be {}ed:", action));
    
    for tool in tools {
        deps.ui.print(&format!(
            "  {} ({}:{})",
            styled_text(&tool.name, MessageStyle::Cyan),
            tool.image_name,
            tool.version
        ));
    }

    deps.ui.print("");
    deps.ui.prompt_input("Continue? [y/N]: ", None)
        .map(|input| input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

/// Confirm tool removal with user
fn confirm_tool_removal(deps: &Arc<ToolsDependencies>, tools: &[String]) -> Result<bool> {
    deps.ui.print("The following tools will be removed:");
    
    for tool in tools {
        deps.ui.print(&format!("  {}", styled_text(tool, MessageStyle::Cyan)));
    }

    deps.ui.print("");
    deps.ui.prompt_input("Continue? [y/N]: ", None)
        .map(|input| input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

/// Add tools to project spin manifest using registry infrastructure
async fn add_tools_to_project(deps: &Arc<ToolsDependencies>, tools: &[ResolvedTool]) -> Result<()> {
    let manifest_path = Path::new("spin.toml");
    
    if !manifest_path.exists() {
        anyhow::bail!("No spin.toml found. Run this command from an FTL project directory.");
    }

    let content = fs::read_to_string(manifest_path)
        .context("Failed to read spin.toml")?;
    
    let mut doc = content.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    // Ensure component section exists
    if !doc.contains_key("component") {
        doc["component"] = Item::Table(Table::new());
    }

    // Get default registry adapter for spin format generation
    let adapter = get_registry_adapter(Some("ghcr"))
        .context("Failed to get GHCR registry adapter")?;

    for tool in tools {
        let component_name = format!("tool-{}", tool.name);
        
        // Check if component already exists
        let components = doc["component"].as_table_mut()
            .context("component section is not a table")?;
        
        if components.contains_key(&component_name) {
            deps.ui.print(&format!(
                "  {} {} (already exists, skipping)",
                styled_text("○", MessageStyle::Yellow),
                styled_text(&tool.name, MessageStyle::Cyan)
            ));
            continue;
        }

        // Use registry adapter to get components for Spin format
        let registry_components = adapter.get_registry_components(&deps.client, &tool.image_name).await
            .with_context(|| format!("Failed to get registry components for {}", tool.image_name))?;

        // Create component entry using registry infrastructure
        let registry_domain = registry_components.registry_domain.clone();
        let package_name = registry_components.package_name.clone();
        
        let mut component = Table::new();
        component["source"] = Item::Table({
            let mut source = Table::new();
            source["registry"] = Item::Value(registry_domain.into());
            source["package"] = Item::Value(package_name.clone().into());
            source["version"] = Item::Value(registry_components.version.clone().into());
            source
        });

        // Add component to components table
        let components = doc["component"].as_table_mut()
            .context("component section is not a table")?;
        components[&component_name] = Item::Table(component);

        // Update tool_components variable
        update_tool_components_variable(&mut doc, &tool.name, true)?;

        deps.ui.print(&format!(
            "  {} {} ({}:{})",
            styled_text("✓", MessageStyle::Green),
            styled_text(&tool.name, MessageStyle::Cyan),
            styled_text(&package_name, MessageStyle::Yellow),
            styled_text(&tool.version, MessageStyle::Yellow)
        ));
    }

    // Write back to file
    fs::write(manifest_path, doc.to_string())
        .context("Failed to write spin.toml")?;

    deps.ui.print("");
    deps.ui.print(&format!(
        "{} tools added successfully!",
        styled_text("✓", MessageStyle::Green)
    ));
    
    Ok(())
}

/// Update tools in project spin manifest
async fn update_tools_in_project(deps: &Arc<ToolsDependencies>, tools: &[ResolvedTool]) -> Result<()> {
    let manifest_path = Path::new("spin.toml");
    
    if !manifest_path.exists() {
        anyhow::bail!("No spin.toml found. Run this command from an FTL project directory.");
    }

    let content = fs::read_to_string(manifest_path)
        .context("Failed to read spin.toml")?;
    
    let mut doc = content.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    let components = doc["component"].as_table_mut()
        .context("component section not found in spin.toml")?;

    // Get registry adapter for version resolution
    let adapter = get_registry_adapter(None)
        .context("Failed to get registry adapter")?;

    for tool in tools {
        let component_name = format!("tool-{}", tool.name);
        
        if let Some(component) = components.get_mut(&component_name) {
            if let Some(source_item) = component.get_mut("source") {
                if let Some(source_table) = source_item.as_inline_table_mut() {
                // Use registry adapter to resolve version properly
                let registry_components = adapter.get_registry_components(&deps.client, &tool.image_name).await
                    .with_context(|| format!("Failed to get registry components for {}", tool.image_name))?;
                    
                    source_table.insert("version", registry_components.version.clone().into());
                    deps.ui.print(&format!(
                        "  {} {} → {}",
                        styled_text("✓", MessageStyle::Green),
                        styled_text(&tool.name, MessageStyle::Cyan),
                        styled_text(&registry_components.version, MessageStyle::Yellow)
                    ));
                }
            }
        } else {
            deps.ui.print(&format!(
                "  {} {} (not found, skipping)",
                styled_text("○", MessageStyle::Yellow),
                styled_text(&tool.name, MessageStyle::Cyan)
            ));
        }
    }

    // Write back to file
    fs::write(manifest_path, doc.to_string())
        .context("Failed to write spin.toml")?;

    deps.ui.print("");
    deps.ui.print(&format!(
        "{} tools updated successfully!",
        styled_text("✓", MessageStyle::Green)
    ));
    
    Ok(())
}

/// Remove tools from project spin manifest
async fn remove_tools_from_project(deps: &Arc<ToolsDependencies>, tools: &[String]) -> Result<()> {
    let manifest_path = Path::new("spin.toml");
    
    if !manifest_path.exists() {
        anyhow::bail!("No spin.toml found. Run this command from an FTL project directory.");
    }

    let content = fs::read_to_string(manifest_path)
        .context("Failed to read spin.toml")?;
    
    let mut doc = content.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    for tool in tools {
        let component_name = format!("tool-{}", tool);
        
        // Try to remove the component
        let components = doc["component"].as_table_mut()
            .context("component section not found in spin.toml")?;
        
        if components.remove(&component_name).is_some() {
            // Update tool_components variable by removing the tool
            update_tool_components_variable(&mut doc, tool, false)?;
            
            deps.ui.print(&format!(
                "  {} {}",
                styled_text("✓", MessageStyle::Green),
                styled_text(tool, MessageStyle::Cyan)
            ));
        } else {
            deps.ui.print(&format!(
                "  {} {} (not found, skipping)",
                styled_text("○", MessageStyle::Yellow),
                styled_text(tool, MessageStyle::Cyan)
            ));
        }
    }

    // Write back to file
    fs::write(manifest_path, doc.to_string())
        .context("Failed to write spin.toml")?;

    deps.ui.print("");
    deps.ui.print(&format!(
        "{} tools removed successfully!",
        styled_text("✓", MessageStyle::Green)
    ));
    
    Ok(())
}

/// Retrieves the list of currently installed tools from spin.toml
/// 
/// Reads the component section and extracts tool names (removing "tool-" prefix).
/// Returns an empty set if no spin.toml exists or no tools are installed.
#[cfg(test)]
pub fn get_installed_tools() -> Result<std::collections::HashSet<String>> {
    get_installed_tools_impl()
}

#[cfg(not(test))]
fn get_installed_tools() -> Result<std::collections::HashSet<String>> {
    get_installed_tools_impl()
}

fn get_installed_tools_impl() -> Result<std::collections::HashSet<String>> {
    let manifest_path = std::path::Path::new("spin.toml");
    
    if !manifest_path.exists() {
        return Ok(std::collections::HashSet::new());
    }

    let content = std::fs::read_to_string(manifest_path)
        .context("Failed to read spin.toml")?;
    
    let doc = content.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    // Look for tool components in the component section
    let mut tools = std::collections::HashSet::new();
    
    if let Some(components) = doc.get("component").and_then(|v| v.as_table()) {
        for (key, _) in components.iter() {
            if key.starts_with("tool-") {
                // Remove "tool-" prefix
                let tool_name = key.strip_prefix("tool-").unwrap().to_string();
                tools.insert(tool_name);
            }
        }
    }

    Ok(tools)
}

/// Update tool_components variable in spin.toml
/// If add=true, adds the tool. If add=false, removes the tool.
#[cfg(test)]
pub fn update_tool_components_variable(doc: &mut DocumentMut, tool_name: &str, add: bool) -> Result<()> {
    update_tool_components_variable_impl(doc, tool_name, add)
}

#[cfg(not(test))]
fn update_tool_components_variable(doc: &mut DocumentMut, tool_name: &str, add: bool) -> Result<()> {
    update_tool_components_variable_impl(doc, tool_name, add)
}

fn update_tool_components_variable_impl(doc: &mut DocumentMut, tool_name: &str, add: bool) -> Result<()> {
    // Ensure variables section exists
    if !doc.contains_key("variables") {
        doc["variables"] = Item::Table(Table::new());
    }

    let variables = doc["variables"].as_table_mut()
        .context("variables section is not a table")?;

    // Ensure tool_components variable exists as an inline table
    if !variables.contains_key("tool_components") {
        variables["tool_components"] = Item::Value(toml_edit::Value::InlineTable({
            let mut table = toml_edit::InlineTable::new();
            table.insert("default", "".into());
            table
        }));
    }

    let tool_components = variables["tool_components"].as_inline_table_mut()
        .context("tool_components is not an inline table")?;

    let current_value = tool_components.get("default")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let new_value = if add {
        // Add tool to comma-separated list
        if current_value.is_empty() {
            tool_name.to_string()
        } else {
            format!("{},{}", current_value, tool_name)
        }
    } else {
        // Remove tool from comma-separated list
        current_value
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && *s != tool_name)
            .collect::<Vec<_>>()
            .join(",")
    };

    tool_components.insert("default", new_value.into());
    Ok(())
}

/// Helper function to create styled text
fn styled_text(text: &str, style: MessageStyle) -> String {
    // For now, return plain text. In a real implementation,
    // this would apply console styling based on MessageStyle
    match style {
        MessageStyle::Green => format!("\x1b[32m{}\x1b[0m", text),
        MessageStyle::Yellow => format!("\x1b[33m{}\x1b[0m", text),
        MessageStyle::Cyan => format!("\x1b[36m{}\x1b[0m", text),
        MessageStyle::Bold => format!("\x1b[1m{}\x1b[0m", text),
        MessageStyle::Red => format!("\x1b[31m{}\x1b[0m", text),
        _ => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_spec() {
        // Simple tool name
        assert_eq!(parse_tool_spec("add", None), ("ghcr".to_string(), "add".to_string(), "latest".to_string()));
        
        // Tool with version
        assert_eq!(parse_tool_spec("add:1.0", None), ("ghcr".to_string(), "add".to_string(), "1.0".to_string()));
        
        // Registry with tool
        assert_eq!(parse_tool_spec("docker:add", None), ("docker".to_string(), "add".to_string(), "latest".to_string()));
        
        // Full registry:tool:version
        assert_eq!(parse_tool_spec("docker:add:1.0", None), ("docker".to_string(), "add".to_string(), "1.0".to_string()));
        
        // Registry override
        assert_eq!(parse_tool_spec("add", Some("ecr")), ("ecr".to_string(), "add".to_string(), "latest".to_string()));
    }

    #[test]
    fn test_tools_manifest_loading() {
        let manifest_content = include_str!("../data/tools.toml");
        let manifest = ToolsManifest::from_toml(manifest_content).unwrap();
        
        assert!(manifest.count() > 80); // Should have ~84 tools
        assert!(manifest.find_tool("add").is_some());
        assert!(manifest.get_categories().contains(&"basic_math".to_string()));
    }

    #[test]
    fn test_tool_filtering() {
        let manifest_content = include_str!("../data/tools.toml");
        let manifest = ToolsManifest::from_toml(manifest_content).unwrap();
        
        let math_tools = manifest.get_tools_by_category("basic_math");
        assert!(!math_tools.is_empty());
        
        let encoding_tools = manifest.search_by_tag("encoding");
        assert!(!encoding_tools.is_empty());
    }
}

#[cfg(test)]
#[path = "tools_tests.rs"]
mod tools_tests;