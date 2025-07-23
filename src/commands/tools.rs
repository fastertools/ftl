use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table};
use reqwest::Client;


use crate::config::FtlConfig;
use crate::registry::{get_registry_adapter, RegistryAdapter};

// Embed the tools manifest at compile time
const TOOLS_MANIFEST: &str = include_str!("../data/tools.toml");

#[derive(Subcommand)]
pub enum ToolsCommand {
    /// List available pre-built tools
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Filter by keyword in name or description
        #[arg(short, long)]
        filter: Option<String>,

        /// Registry to use (overrides config)
        #[arg(short, long)]
        registry: Option<String>,

        /// Show additional details
        #[arg(short, long)]
        verbose: bool,

        /// List from all enabled registries
        #[arg(short, long)]
        all: bool,
    },

    /// Add pre-built tools to your project
    Add {
        /// Tool names to add (can include registry prefix like docker:tool-name)
        tools: Vec<String>,

        /// Registry to use (overrides config and tool prefix)
        #[arg(short, long)]
        registry: Option<String>,

        /// Version/tag to use (overrides tool:version syntax)
        #[arg(short, long)]
        version: Option<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Update existing tools in your project
    Update {
        /// Tool names to update (can include registry prefix like docker:tool-name)
        tools: Vec<String>,

        /// Registry to use (overrides config and tool prefix)
        #[arg(short, long)]
        registry: Option<String>,

        /// Version/tag to update to (overrides tool:version syntax)
        #[arg(short, long)]
        version: Option<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Remove tools from your project
    Remove {
        /// Tool names to remove
        tools: Vec<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Debug, Deserialize)]
struct ToolsManifest {
    tools: Vec<Tool>,
}

#[derive(Debug, Deserialize, Clone)]
struct Tool {
    name: String,
    category: String,
    description: String,
    image_name: String,
    tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct VerifiedTool {
    tool: Tool,
    available_registries: Vec<String>,
}

#[derive(Debug, Clone)]
struct ResolvedTool {
    name: String,
    description: String,
    image_name: String,
    #[allow(dead_code)]
    category: String,
    #[allow(dead_code)]
    tags: Vec<String>,
    from_manifest: bool,
    version: String,
}


pub async fn handle_command(cmd: ToolsCommand) -> Result<()> {
    match cmd {
        ToolsCommand::List { category, filter, registry, verbose, all } => {
            list_tools(category, filter, registry, verbose, all).await
        }
        ToolsCommand::Add { tools, registry, version, yes } => {
            add_tools(tools, registry, version, yes).await
        }
        ToolsCommand::Update { tools, registry, version, yes } => {
            update_tools(tools, registry, version, yes).await
        }
        ToolsCommand::Remove { tools, yes } => {
            remove_tools(tools, yes).await
        }
    }
}

async fn verify_tools_in_registries(
    tools: &[Tool],
    registries: &[(String, &crate::config::registry::RegistryConfig)],
    client: &Client,
) -> Result<Vec<VerifiedTool>> {
    let mut verified_tools = Vec::new();
    
    println!("{} Verifying tools in registries...", style("🔍").dim());
    
    for tool in tools {
        let mut available_registries = Vec::new();
        
        for (reg_name, _reg_config) in registries {
            if let Ok(adapter) = get_registry_adapter(Some(reg_name)) {
                match adapter.verify_image_exists(client, &tool.image_name).await {
                    Ok(true) => {
                        available_registries.push(reg_name.clone());
                    }
                    Ok(false) => {
                        // Tool doesn't exist in this registry - this is normal
                    }
                    Err(e) => {
                        // Check if it's a crane availability issue
                        if e.to_string().contains("crane") {
                            // Re-return the error to propagate it up
                            return Err(e);
                        }
                        // Other errors (rate limit, network, etc.) - skip this registry
                    }
                }
            }
        }
        
        // Only include tools that exist in at least one registry
        if !available_registries.is_empty() {
            verified_tools.push(VerifiedTool {
                tool: tool.clone(),
                available_registries,
            });
        }
    }
    
    Ok(verified_tools)
}

async fn list_tools(
    category: Option<String>, 
    filter: Option<String>, 
    registry: Option<String>, 
    verbose: bool,
    all: bool
) -> Result<()> {
    let manifest: ToolsManifest = toml::from_str(TOOLS_MANIFEST)
        .context("Failed to parse tools manifest")?;
    
    let config = FtlConfig::load()?;
    let client = Client::new();
    
    // Determine which registries to list from
    let registries_to_query = if all {
        // List from all enabled registries
        config.enabled_registries()
            .into_iter()
            .map(|r| (r.name.clone(), r))
            .collect::<Vec<_>>()
    } else if let Some(reg_name) = registry {
        // List from specific registry
        let reg = config.get_registry(&reg_name)
            .context(format!("Registry '{}' not found", reg_name))?;
        vec![(reg.name.clone(), reg)]
    } else {
        // List from default registry
        let default_reg = config.get_registry(&config.default_registry)
            .context(format!("Default registry '{}' not found", config.default_registry))?;
        vec![(default_reg.name.clone(), default_reg)]
    };

    // Apply initial filters to the manifest before verification
    let candidate_tools: Vec<&Tool> = manifest.tools.iter()
        .filter(|tool| {
            if let Some(cat) = &category {
                tool.category.to_lowercase() == cat.to_lowercase()
            } else {
                true
            }
        })
        .filter(|tool| {
            if let Some(f) = &filter {
                let f_lower = f.to_lowercase();
                tool.name.to_lowercase().contains(&f_lower)
                    || tool.description.to_lowercase().contains(&f_lower)
                    || tool.tags.iter().any(|tag| tag.to_lowercase().contains(&f_lower))
            } else {
                true
            }
        })
        .collect();

    if candidate_tools.is_empty() {
        println!("No tools found matching your criteria");
        return Ok(());
    }

    // Verify which tools actually exist in the registries
    let verified_tools = verify_tools_in_registries(&candidate_tools.into_iter().cloned().collect::<Vec<_>>(), &registries_to_query, &client).await?;

    if verified_tools.is_empty() {
        println!("No tools found in the specified registries");
        return Ok(());
    }

    println!("{} Available FTL Tools", style("📦").dim());
    
    if registries_to_query.len() > 1 {
        println!();
        println!("Showing tools from {} registries:", registries_to_query.len());
        for (name, _) in &registries_to_query {
            println!("  • {}", style(name).cyan());
        }
    }
    println!();

    // Group verified tools by category
    let mut categories: HashMap<&str, Vec<&VerifiedTool>> = HashMap::new();
    for verified_tool in &verified_tools {
        categories.entry(&verified_tool.tool.category).or_default().push(verified_tool);
    }

    // Sort categories
    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by_key(|(cat, _)| *cat);

    // Display tools grouped by category
    for (category, mut verified_tools) in sorted_categories {
        println!("[{}]", style(category).bold().cyan());
        
        // Sort tools by name within category
        verified_tools.sort_by_key(|vt| &vt.tool.name);
        
        for verified_tool in verified_tools {
            let tool = &verified_tool.tool;
            
            println!("  {} - {}", style(&tool.name).green(), tool.description);
            
            if verbose {
                println!("    Available in:");
                for reg_name in &verified_tool.available_registries {
                    let adapter = get_registry_adapter(Some(reg_name))?;
                    let url = adapter.get_registry_url(&tool.image_name);
                    println!("      {} ({})", style(reg_name).cyan(), style(&url).dim());
                }
                if !tool.tags.is_empty() {
                    println!("    Tags: {}", tool.tags.join(", "));
                }
            } else {
                println!("    Available in: {}", style(verified_tool.available_registries.join(", ")).dim());
            }
        }
        println!();
    }

    println!();
    println!("To add a tool to your project:");
    println!("  {} ftl tools add <tool-name>", style("$").dim());
    
    if all {
        println!();
        println!("To add from a specific registry:");
        println!("  {} ftl tools add <registry>:<tool-name>", style("$").dim());
    }

    Ok(())
}

async fn add_tools(tools: Vec<String>, registry: Option<String>, version: Option<String>, yes: bool) -> Result<()> {
    let manifest: ToolsManifest = toml::from_str(TOOLS_MANIFEST)
        .context("Failed to parse tools manifest")?;
    
    let config = FtlConfig::load()?;
    
    // Check if spin.toml exists
    if !Path::new("spin.toml").exists() {
        println!("{} No spin.toml found in current directory", style("✗").red());
        println!("{} Run this command from your FTL project root", style("→").cyan());
        return Ok(());
    }

    // Parse tools and their registries
    let mut tools_to_add = Vec::new();
    
    for tool_spec in &tools {
        // Parse tool specification: could be "tool", "tool:tag", or "registry:tool:tag"
        let parts: Vec<&str> = tool_spec.splitn(3, ':').collect();
        
        let (reg_name, tool_name, parsed_tag) = match parts.as_slice() {
            [single] => {
                // Just tool name, use default registry and tag
                let reg = registry.as_ref().unwrap_or(&config.default_registry).clone();
                (reg, single.to_string(), "latest".to_string())
            }
            [first, second] => {
                // Could be "registry:tool" or "tool:tag"
                // Check if first part is a known registry
                if config.registries.iter().any(|r| r.name == *first) {
                    // It's "registry:tool"
                    (first.to_string(), second.to_string(), "latest".to_string())
                } else {
                    // It's "tool:tag"
                    let reg = registry.as_ref().unwrap_or(&config.default_registry).clone();
                    (reg, first.to_string(), second.to_string())
                }
            }
            [reg, tool, tag] => {
                // Full "registry:tool:tag"
                (reg.to_string(), tool.to_string(), tag.to_string())
            }
            _ => anyhow::bail!("Invalid tool specification: {}", tool_spec),
        };
        
        // CLI version flag overrides any parsed version
        let tag = version.as_ref().unwrap_or(&parsed_tag).clone();
        
        // Verify registry exists
        let _registry_config = config.registries.iter()
            .find(|r| r.name == reg_name)
            .context(format!("Registry '{}' not found", reg_name))?;
        
        // Try to find tool in manifest first
        let resolved_tool = if let Some(manifest_tool) = manifest.tools.iter().find(|t| t.name == tool_name) {
            // Found in manifest - use manifest data, but allow version override
            ResolvedTool {
                name: manifest_tool.name.clone(),
                description: manifest_tool.description.clone(),
                image_name: manifest_tool.image_name.clone(),
                category: manifest_tool.category.clone(),
                tags: manifest_tool.tags.clone(),
                from_manifest: true,
                version: tag,
            }
        } else {
            // Not in manifest - use tool name as-is
            let image_name = tool_name.clone();
            ResolvedTool {
                name: tool_name.clone(),
                description: format!("Custom tool: {}", tool_name),
                image_name,
                category: "custom".to_string(),
                tags: vec!["user-defined".to_string()],
                from_manifest: false,
                version: tag,
            }
        };
        
        tools_to_add.push((reg_name, resolved_tool));
    }

    // Show what will be added
    println!("{} Adding the following tools:", style("→").cyan());
    println!();
    for (reg, resolved_tool) in &tools_to_add {
        let source_indicator = if resolved_tool.from_manifest { "" } else { " [custom]" };
        println!("  {} {} ({}{}) - {}", 
            style("•").green(), 
            style(&resolved_tool.name).bold(),
            style(reg).dim(),
            style(source_indicator).dim(),
            resolved_tool.description
        );
    }

    // Confirm unless -y flag is provided
    if !yes {
        use dialoguer::Confirm;
        let proceed = Confirm::new()
            .with_prompt("Do you want to add these tools?")
            .default(true)
            .interact()?;
        
        if !proceed {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    // Read and parse spin.toml
    let contents = fs::read_to_string("spin.toml")
        .context("Failed to read spin.toml")?;
    
    let mut doc = contents.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    // Add tools
    for (reg_name, resolved_tool) in &tools_to_add {
        let adapter = get_registry_adapter(Some(reg_name))?;
        let registry_url = adapter.get_registry_url(&resolved_tool.image_name);
        
        add_tool_to_spin(&mut doc, &resolved_tool.name, &registry_url, &resolved_tool.version)?;
    }

    // Write back to spin.toml
    fs::write("spin.toml", doc.to_string())
        .context("Failed to write spin.toml")?;

    println!();
    println!("{} Successfully added tools to spin.toml", style("✓").green());
    println!("{} Remember to run 'ftl build' to pull the components", style("→").cyan());

    Ok(())
}

fn add_tool_to_spin(doc: &mut DocumentMut, tool_name: &str, registry_url: &str, version: &str) -> Result<()> {
    // Get or create the component section
    if !doc.contains_key("component") {
        doc.insert("component", Item::Table(Table::new()));
    }
    
    let component_section = doc.get_mut("component")
        .and_then(|item| item.as_table_mut())
        .context("Failed to get component section")?;
    
    // Create the component table
    let mut component = Table::new();
    
    // Create source as inline table with correct Spin schema
    let mut source = toml_edit::InlineTable::new();
    
    // Extract registry base URL and organization from the full registry URL
    let (registry_base, org_and_tool) = if registry_url.starts_with("ghcr.io/") {
        ("ghcr.io", registry_url.strip_prefix("ghcr.io/").unwrap())
    } else if registry_url.starts_with("docker.io/") {
        ("docker.io", registry_url.strip_prefix("docker.io/").unwrap())  
    } else {
        // For other registries, split on first slash
        let parts: Vec<&str> = registry_url.splitn(2, '/').collect();
        (parts[0], parts.get(1).copied().unwrap_or(tool_name))
    };
    
    source.insert("registry", toml_edit::Value::String(toml_edit::Formatted::new(registry_base.to_string())));
    // Convert package from org/tool format to org:tool format for Spin
    let package = org_and_tool.replace('/', ":");
    source.insert("package", toml_edit::Value::String(toml_edit::Formatted::new(package)));
    source.insert("version", toml_edit::Value::String(toml_edit::Formatted::new(version.to_string())));
    
    component.insert("source", Item::Value(toml_edit::Value::InlineTable(source)));
    
    // Insert into the component section
    component_section.insert(tool_name, Item::Table(component));
    
    // Update tool_components variable
    let variables = doc.get_mut("variables")
        .and_then(|v| v.as_table_mut())
        .context("Missing [variables] section in spin.toml")?;
    
    let tool_components = variables.get_mut("tool_components")
        .and_then(|v| v.as_inline_table_mut())
        .context("Missing tool_components variable")?;
    
    let current_value = tool_components.get("default")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let new_value = if current_value.is_empty() {
        tool_name.to_string()
    } else {
        format!("{},{}", current_value, tool_name)
    };
    
    tool_components["default"] = toml_edit::Value::from(new_value);
    
    Ok(())
}

async fn update_tools(tools: Vec<String>, registry: Option<String>, version: Option<String>, yes: bool) -> Result<()> {
    let manifest: ToolsManifest = toml::from_str(TOOLS_MANIFEST)
        .context("Failed to parse tools manifest")?;
    
    let config = FtlConfig::load()?;
    
    // Check if spin.toml exists
    if !Path::new("spin.toml").exists() {
        println!("{} No spin.toml found in current directory", style("✗").red());
        println!("{} Run this command from your FTL project root", style("→").cyan());
        return Ok(());
    }

    // Read current spin.toml to check which tools are installed
    let contents = fs::read_to_string("spin.toml")
        .context("Failed to read spin.toml")?;
    
    let doc = contents.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    // Get current tool_components to verify which tools exist
    let current_components = doc.get("variables")
        .and_then(|v| v.as_table())
        .and_then(|vars| vars.get("tool_components"))
        .and_then(|tc| tc.as_inline_table())
        .and_then(|tc| tc.get("default"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<std::collections::HashSet<_>>();

    // Parse tools and validate they exist in project
    let mut tools_to_update = Vec::new();
    
    for tool_spec in &tools {
        // Parse tool specification: could be "tool", "tool:tag", or "registry:tool:tag"
        let parts: Vec<&str> = tool_spec.splitn(3, ':').collect();
        
        let (reg_name, tool_name, parsed_tag) = match parts.as_slice() {
            [single] => {
                // Just tool name, use default registry and tag
                let reg = registry.as_ref().unwrap_or(&config.default_registry).clone();
                (reg, single.to_string(), "latest".to_string())
            }
            [first, second] => {
                // Could be "registry:tool" or "tool:tag"
                // Check if first part is a known registry
                if config.registries.iter().any(|r| r.name == *first) {
                    // It's "registry:tool"
                    (first.to_string(), second.to_string(), "latest".to_string())
                } else {
                    // It's "tool:tag"
                    let reg = registry.as_ref().unwrap_or(&config.default_registry).clone();
                    (reg, first.to_string(), second.to_string())
                }
            }
            [reg, tool, tag] => {
                // Full "registry:tool:tag"
                (reg.to_string(), tool.to_string(), tag.to_string())
            }
            _ => anyhow::bail!("Invalid tool specification: {}", tool_spec),
        };
        
        // CLI version flag overrides any parsed version
        let tag = version.as_ref().unwrap_or(&parsed_tag).clone();
        
        // Verify tool is currently installed
        if !current_components.contains(tool_name.as_str()) {
            println!("{} Tool '{}' is not currently installed", style("✗").red(), tool_name);
            println!("{} Use 'ftl tools add {}' to install it first", style("→").cyan(), tool_name);
            continue;
        }
        
        // Verify registry exists
        let _registry_config = config.registries.iter()
            .find(|r| r.name == reg_name)
            .context(format!("Registry '{}' not found", reg_name))?;
        
        // Try to find tool in manifest first
        let resolved_tool = if let Some(manifest_tool) = manifest.tools.iter().find(|t| t.name == tool_name) {
            // Found in manifest - use manifest data, but allow version override
            ResolvedTool {
                name: manifest_tool.name.clone(),
                description: manifest_tool.description.clone(),
                image_name: manifest_tool.image_name.clone(),
                category: manifest_tool.category.clone(),
                tags: manifest_tool.tags.clone(),
                from_manifest: true,
                version: tag,
            }
        } else {
            // Not in manifest - use tool name as-is
            let image_name = tool_name.clone();
            ResolvedTool {
                name: tool_name.clone(),
                description: format!("Custom tool: {}", tool_name),
                image_name,
                category: "custom".to_string(),
                tags: vec!["user-defined".to_string()],
                from_manifest: false,
                version: tag,
            }
        };
        
        tools_to_update.push((reg_name, resolved_tool));
    }

    if tools_to_update.is_empty() {
        println!("{} No tools to update", style("!").yellow());
        return Ok(());
    }

    // Show what will be updated
    println!("{} Updating the following tools:\n", style("→").cyan());
    for (reg, resolved_tool) in &tools_to_update {
        let source_indicator = if resolved_tool.from_manifest { "" } else { " [custom]" };
        println!("  {} {} ({}{}) to version {} - {}", 
            style("•").green(), 
            style(&resolved_tool.name).bold(),
            style(reg).dim(),
            style(source_indicator).dim(),
            style(&resolved_tool.version).yellow(),
            resolved_tool.description
        );
    }

    // Confirm unless -y flag is provided
    if !yes {
        use dialoguer::Confirm;
        let proceed = Confirm::new()
            .with_prompt("Do you want to update these tools?")
            .default(true)
            .interact()?;
        
        if !proceed {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    // Parse spin.toml again for updating
    let mut doc = contents.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    // Update tools by replacing their component entries
    for (reg_name, resolved_tool) in &tools_to_update {
        let adapter = get_registry_adapter(Some(reg_name))?;
        let registry_url = adapter.get_registry_url(&resolved_tool.image_name);
        
        // Remove the old component entry and add the new one
        if let Some(component_section) = doc.get_mut("component").and_then(|item| item.as_table_mut()) {
            component_section.remove(&resolved_tool.name);
        }
        
        add_tool_to_spin(&mut doc, &resolved_tool.name, &registry_url, &resolved_tool.version)?;
    }

    // Write back to spin.toml
    fs::write("spin.toml", doc.to_string())
        .context("Failed to write spin.toml")?;

    println!();
    println!("{} Successfully updated tools in spin.toml", style("✓").green());
    println!("{} Remember to run 'ftl build' to pull the updated components", style("→").cyan());

    Ok(())
}

async fn remove_tools(tools: Vec<String>, yes: bool) -> Result<()> {
    // Check if spin.toml exists
    if !Path::new("spin.toml").exists() {
        println!("{} No spin.toml found in current directory", style("✗").red());
        println!("{} Run this command from your FTL project root", style("→").cyan());
        return Ok(());
    }

    // Read current spin.toml to check which tools are installed
    let contents = fs::read_to_string("spin.toml")
        .context("Failed to read spin.toml")?;
    
    let doc = contents.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    // Get current tool_components to verify which tools exist
    let current_components = doc.get("variables")
        .and_then(|v| v.as_table())
        .and_then(|vars| vars.get("tool_components"))
        .and_then(|tc| tc.as_inline_table())
        .and_then(|tc| tc.get("default"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<std::collections::HashSet<_>>();

    // Validate tools exist and collect valid ones
    let mut tools_to_remove = Vec::new();
    
    for tool_name in &tools {
        if current_components.contains(tool_name.as_str()) {
            tools_to_remove.push(tool_name.clone());
        } else {
            println!("{} Tool '{}' is not currently installed", style("!").yellow(), tool_name);
        }
    }

    if tools_to_remove.is_empty() {
        println!("{} No tools to remove", style("!").yellow());
        return Ok(());
    }

    // Show what will be removed
    println!("{} Removing the following tools:\n", style("→").cyan());
    for tool_name in &tools_to_remove {
        println!("  {} {}", style("•").red(), style(tool_name).bold());
    }

    // Confirm unless -y flag is provided
    if !yes {
        use dialoguer::Confirm;
        let proceed = Confirm::new()
            .with_prompt("Do you want to remove these tools?")
            .default(true)
            .interact()?;
        
        if !proceed {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    // Parse spin.toml again for modification
    let mut doc = contents.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    // Remove tools
    for tool_name in &tools_to_remove {
        remove_tool_from_spin(&mut doc, tool_name)?;
    }

    // Write back to spin.toml
    fs::write("spin.toml", doc.to_string())
        .context("Failed to write spin.toml")?;

    println!();
    println!("{} Successfully removed tools from spin.toml", style("✓").green());

    Ok(())
}

fn remove_tool_from_spin(doc: &mut DocumentMut, tool_name: &str) -> Result<()> {
    // Remove from component section
    if let Some(component_section) = doc.get_mut("component").and_then(|item| item.as_table_mut()) {
        component_section.remove(tool_name);
    }
    
    // Update tool_components variable by removing the tool
    let variables = doc.get_mut("variables")
        .and_then(|v| v.as_table_mut())
        .context("Missing [variables] section in spin.toml")?;
    
    let tool_components = variables.get_mut("tool_components")
        .and_then(|v| v.as_inline_table_mut())
        .context("Missing tool_components variable")?;
    
    let current_value = tool_components.get("default")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    // Remove the tool from the comma-separated list
    let new_value = current_value
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != tool_name)
        .collect::<Vec<_>>()
        .join(",");
    
    tool_components["default"] = toml_edit::Value::from(new_value);
    
    Ok(())
}