use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table};
use reqwest::Client;
use serde_json;


use crate::config::FtlConfig;
use crate::registry::{get_registry_adapter, RegistryAdapter};

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
        
        /// Query registry directly, skip manifest
        #[arg(short, long)]
        direct: bool,
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

#[derive(Debug, Clone)]
struct ResolvedTool {
    name: String,
    image_name: String,
    version: String,
}


pub async fn handle_command(cmd: ToolsCommand) -> Result<()> {
    match cmd {
        ToolsCommand::List { category, filter, registry, verbose, all, direct } => {
            list_tools(category, filter, registry, verbose, all, direct).await
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


async fn query_ghcr_packages(organization: &str, prefix: &str) -> Result<Vec<String>> {
    // GitHub API to list packages
    let client = Client::new();
    
    // GitHub API endpoint for organization packages
    let url = format!("https://api.github.com/orgs/{}/packages?package_type=container&per_page=100", organization);
    
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "ftl-cli")
        .send()
        .await?;
    
    if !response.status().is_success() {
        // If GitHub API fails, fall back to using crane catalog if available
        use std::process::Command;
        
        println!("GitHub API failed, trying alternative method...");
        
        // Try using GitHub CLI if available
        let output = Command::new("gh")
            .args(&["api", &format!("/orgs/{}/packages?package_type=container&per_page=100", organization)])
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse JSON response
                if let Ok(packages) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                    let tools: Vec<String> = packages
                        .iter()
                        .filter_map(|p| p.get("name").and_then(|n| n.as_str()))
                        .filter(|name| name.starts_with(prefix))
                        .map(|s| s.to_string())
                        .collect();
                    return Ok(tools);
                }
            }
        }
        
        return Ok(Vec::new());
    }
    
    let packages: Vec<serde_json::Value> = response.json().await?;
    
    let tools: Vec<String> = packages
        .iter()
        .filter_map(|p| p.get("name").and_then(|n| n.as_str()))
        .filter(|name| name.starts_with(prefix))
        .map(|s| s.to_string())
        .collect();
    
    Ok(tools)
}

async fn list_tools(
    _category: Option<String>, 
    filter: Option<String>, 
    registry: Option<String>, 
    verbose: bool,
    all: bool,
    _direct: bool
) -> Result<()> {
    let config = FtlConfig::load()?;
    
    // Always use direct registry query mode now
    let registries_to_query = if all {
        config.enabled_registries()
            .into_iter()
            .map(|r| (r.name.clone(), r))
            .collect::<Vec<_>>()
    } else if let Some(reg_name) = registry {
        let reg = config.get_registry(&reg_name)
            .context(format!("Registry '{}' not found", reg_name))?;
        vec![(reg.name.clone(), reg)]
    } else {
        let default_reg = config.get_registry(&config.default_registry)
            .context(format!("Default registry '{}' not found", config.default_registry))?;
        vec![(default_reg.name.clone(), default_reg)]
    };
    
    println!("{} Querying registries for available tools", style("📦").cyan());
    
    for (reg_name, reg_config) in registries_to_query {
        if reg_config.registry_type == crate::config::registry::RegistryType::Ghcr {
            let org = reg_config.get_config_str("organization")
                .unwrap_or_else(|| "fastertools".to_string());
            
            println!("\nQuerying {} packages from {}...", style(&reg_name).yellow(), org);
            
            // Query GitHub API for all ftl-tool-* packages
            let mut tools = query_ghcr_packages(&org, "ftl-tool-").await?;
            
            // Apply filter if provided
            if let Some(f) = &filter {
                let f_lower = f.to_lowercase();
                tools.retain(|tool| tool.to_lowercase().contains(&f_lower));
            }
            
            tools.sort();
            
            if tools.is_empty() {
                println!("No tools found in {}", reg_name);
            } else {
                println!("\n{} tools found in {}:", tools.len(), style(&reg_name).green());
                for tool in tools {
                    let display_name = tool.strip_prefix("ftl-tool-").unwrap_or(&tool);
                    println!("  {} {}", style("•").green(), style(display_name).bold());
                    if verbose {
                        let adapter = get_registry_adapter(Some(&reg_name))?;
                        let url = adapter.get_registry_url(&tool);
                        println!("    Registry: {}", style(&url).dim());
                    }
                }
            }
        } else {
            println!("Direct query not implemented for {} registry type", reg_config.registry_type);
        }
    }
    
    println!("\nTo add a tool to your project:");
    println!("  {} ftl tools add <tool-name>:<version>", style("$").dim());
    
    if all {
        println!("To add from a specific registry:");
        println!("  {} ftl tools add <registry>:<tool-name>:<version>", style("$").dim());
    }

    Ok(())
}

async fn add_tools(tools: Vec<String>, registry: Option<String>, version: Option<String>, yes: bool) -> Result<()> {
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
        
        // Create resolved tool directly from user input
        let image_name = format!("{}:{}", tool_name, tag);
        let resolved_tool = ResolvedTool {
            name: tool_name.clone(),
            image_name,
            version: tag,
        };
        
        tools_to_add.push((reg_name, resolved_tool));
    }

    // Show what will be added
    println!("{} Adding the following tools:", style("→").cyan());
    println!();
    for (reg, resolved_tool) in &tools_to_add {
        println!("  {} {} ({})", 
            style("•").green(), 
            style(&resolved_tool.name).bold(),
            style(reg).dim()
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
    let client = reqwest::Client::new();
    for (reg_name, resolved_tool) in &tools_to_add {
        let adapter = get_registry_adapter(Some(reg_name))?;
        let registry_components = adapter.get_registry_components(&client, &resolved_tool.image_name).await?;
        
        add_tool_to_spin(&mut doc, &resolved_tool.name, &registry_components)?;
    }

    // Write back to spin.toml
    fs::write("spin.toml", doc.to_string())
        .context("Failed to write spin.toml")?;

    println!();
    println!("{} Successfully added tools to spin.toml", style("✓").green());
    println!("{} Remember to run 'ftl build' to pull the components", style("→").cyan());

    Ok(())
}

fn add_tool_to_spin(doc: &mut DocumentMut, tool_name: &str, registry_components: &crate::registry::RegistryComponents) -> Result<()> {
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
    
    // Use registry components directly - they're already in the correct format
    source.insert("registry", toml_edit::Value::String(toml_edit::Formatted::new(registry_components.registry_domain.clone())));
    source.insert("package", toml_edit::Value::String(toml_edit::Formatted::new(registry_components.package_name.clone())));
    source.insert("version", toml_edit::Value::String(toml_edit::Formatted::new(registry_components.version.clone())));
    
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
        
        // Create resolved tool directly from user input
        let image_name = format!("{}:{}", tool_name, tag);
        let resolved_tool = ResolvedTool {
            name: tool_name.clone(),
            image_name,
            version: tag,
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
        println!("  {} {} ({}) to version {}", 
            style("•").green(), 
            style(&resolved_tool.name).bold(),
            style(reg).dim(),
            style(&resolved_tool.version).yellow()
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
    let client = reqwest::Client::new();
    for (reg_name, resolved_tool) in &tools_to_update {
        let adapter = get_registry_adapter(Some(reg_name))?;
        let registry_components = adapter.get_registry_components(&client, &resolved_tool.image_name).await?;
        
        // Remove the old component entry and add the new one
        if let Some(component_section) = doc.get_mut("component").and_then(|item| item.as_table_mut()) {
            component_section.remove(&resolved_tool.name);
        }
        
        add_tool_to_spin(&mut doc, &resolved_tool.name, &registry_components)?;
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