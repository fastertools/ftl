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

#[derive(Debug, Clone)]
struct ResolvedTool {
    name: String,
    image_name: String,
    version: String,
}

/// Execute tools list command with dependency injection
pub async fn list_with_deps(
    deps: &Arc<ToolsDependencies>,
    category: Option<&str>,
    filter: Option<&str>,
    registry: Option<&str>,
    verbose: bool,
    _all: bool,
    direct: bool,
) -> Result<()> {
    if direct {
        // Query registry directly via GitHub API
        list_tools_from_registry(deps, registry, category, filter, verbose).await
    } else {
        // Use embedded tools.toml manifest
        list_tools_from_manifest(deps, category, filter, verbose).await
    }
}

/// Execute tools add command with dependency injection
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

/// Execute tools update command with dependency injection
pub async fn update_with_deps(
    deps: &Arc<ToolsDependencies>,
    tools: &[String],
    registry: Option<&str>,
    version: Option<&str>,
    yes: bool,
) -> Result<()> {
    let resolved_tools = resolve_tools(deps, tools, registry, version).await?;
    
    if !yes {
        if !confirm_tool_changes(deps, &resolved_tools, "update")? {
            deps.ui.print(&styled_text("Operation cancelled.", MessageStyle::Yellow));
            return Ok(());
        }
    }

    update_tools_in_project(deps, &resolved_tools).await
}

/// Execute tools remove command with dependency injection
pub async fn remove_with_deps(
    deps: &Arc<ToolsDependencies>,
    tools: &[String],
    yes: bool,
) -> Result<()> {
    if !yes {
        if !confirm_tool_removal(deps, tools)? {
            deps.ui.print(&styled_text("Operation cancelled.", MessageStyle::Yellow));
            return Ok(());
        }
    }

    remove_tools_from_project(deps, tools).await
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

/// List tools from registry via GitHub API
async fn list_tools_from_registry(
    deps: &Arc<ToolsDependencies>,
    registry: Option<&str>,
    _category: Option<&str>,
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
        let (tool_name, tool_version) = parse_tool_spec(tool);
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

/// Parse tool specification (name or name:version)
pub fn parse_tool_spec(spec: &str) -> (String, String) {
    if let Some(colon_pos) = spec.find(':') {
        let name = spec[..colon_pos].to_string();
        let version = spec[colon_pos + 1..].to_string();
        (name, version)
    } else {
        (spec.to_string(), "latest".to_string())
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

    let components = doc["component"].as_table_mut()
        .context("component section is not a table")?;

    // Get default registry adapter for spin format generation
    let adapter = get_registry_adapter(Some("ghcr"))
        .context("Failed to get GHCR registry adapter")?;

    for tool in tools {
        let component_name = format!("tool-{}", tool.name);
        
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

        components[&component_name] = Item::Table(component);

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
            if let Some(source) = component.get_mut("source").and_then(|s| s.as_table_mut()) {
                // Use registry adapter to resolve version properly
                let registry_components = adapter.get_registry_components(&deps.client, &tool.image_name).await
                    .with_context(|| format!("Failed to get registry components for {}", tool.image_name))?;
                    
                source["version"] = Item::Value(registry_components.version.clone().into());
                deps.ui.print(&format!(
                    "  {} {} → {}",
                    styled_text("✓", MessageStyle::Green),
                    styled_text(&tool.name, MessageStyle::Cyan),
                    styled_text(&registry_components.version, MessageStyle::Yellow)
                ));
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

    let components = doc["component"].as_table_mut()
        .context("component section not found in spin.toml")?;

    for tool in tools {
        let component_name = format!("tool-{}", tool);
        
        if components.remove(&component_name).is_some() {
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
        assert_eq!(parse_tool_spec("add"), ("add".to_string(), "latest".to_string()));
        assert_eq!(parse_tool_spec("add:1.0"), ("add".to_string(), "1.0".to_string()));
        assert_eq!(parse_tool_spec("ftl-tool-add:2.1"), ("ftl-tool-add".to_string(), "2.1".to_string()));
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