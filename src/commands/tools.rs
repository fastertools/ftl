use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table};

use crate::config::FtlConfig;
use crate::registry::get_registry_adapter;

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

pub async fn handle_command(cmd: ToolsCommand) -> Result<()> {
    match cmd {
        ToolsCommand::List { category, filter, registry, verbose, all } => {
            list_tools(category, filter, registry, verbose, all).await
        }
        ToolsCommand::Add { tools, registry, yes } => {
            add_tools(tools, registry, yes).await
        }
    }
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
            .or_else(|| {
                // Fallback to old behavior for backward compatibility
                match reg_name.as_str() {
                    "ghcr" | "docker" | "ecr" => Some(&config.registries[0]), // Use any registry as placeholder
                    _ => None
                }
            })
            .context(format!("Registry '{}' not found", reg_name))?;
        vec![(reg.name.clone(), reg)]
    } else {
        // List from default registry
        let default_reg = config.get_registry(&config.default_registry)
            .context(format!("Default registry '{}' not found", config.default_registry))?;
        vec![(default_reg.name.clone(), default_reg)]
    };

    // Apply filters
    let filtered_tools: Vec<&Tool> = manifest.tools.iter()
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

    if filtered_tools.is_empty() {
        println!("No tools found matching your criteria");
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

    // Group tools by category
    let mut categories: HashMap<&str, Vec<&Tool>> = HashMap::new();
    for tool in &filtered_tools {
        categories.entry(&tool.category).or_default().push(tool);
    }

    // Sort categories
    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by_key(|(cat, _)| *cat);

    // Display tools grouped by category
    for (category, mut tools) in sorted_categories {
        println!("[{}]", style(category).bold().cyan());
        
        // Sort tools by name within category
        tools.sort_by_key(|t| &t.name);
        
        for tool in tools {
            // Show tool for each registry if listing all
            if all && registries_to_query.len() > 1 {
                for (reg_name, _reg_config) in &registries_to_query {
                    let adapter = get_registry_adapter(Some(&reg_name))?;
                    let url = adapter.get_registry_url(&tool.image_name);
                    
                    println!("  {} ({}) - {}", 
                        style(&tool.name).green(), 
                        style(reg_name).dim(),
                        tool.description
                    );
                    
                    if verbose {
                        println!("    Registry: {}", style(&url).dim());
                        if !tool.tags.is_empty() {
                            println!("    Tags: {}", tool.tags.join(", "));
                        }
                    }
                }
            } else {
                // Single registry display
                let (reg_name, _) = &registries_to_query[0];
                let adapter = get_registry_adapter(Some(reg_name))?;
                let url = adapter.get_registry_url(&tool.image_name);
                
                println!("  {} - {}", style(&tool.name).green(), tool.description);
                
                if verbose {
                    println!("    Registry: {}", style(&url).dim());
                    if !tool.tags.is_empty() {
                        println!("    Tags: {}", tool.tags.join(", "));
                    }
                }
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

async fn add_tools(tools: Vec<String>, registry: Option<String>, yes: bool) -> Result<()> {
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
        let (reg_name, tool_name) = if let Some((reg, name)) = tool_spec.split_once(':') {
            // Registry prefix provided (e.g., docker:tool-name)
            (reg.to_string(), name.to_string())
        } else if let Some(reg) = &registry {
            // Registry flag provided
            (reg.clone(), tool_spec.clone())
        } else {
            // Use default registry
            (config.default_registry.clone(), tool_spec.clone())
        };
        
        // Verify registry exists
        if !config.registries.iter().any(|r| r.name == reg_name) {
            // Fallback for backward compatibility
            if !matches!(reg_name.as_str(), "ghcr" | "docker" | "ecr") {
                anyhow::bail!("Registry '{}' not found", reg_name);
            }
        }
        
        // Find tool in manifest
        let tool = manifest.tools.iter()
            .find(|t| t.name == tool_name)
            .context(format!("Tool '{}' not found in manifest", tool_name))?;
        
        tools_to_add.push((reg_name, tool.clone()));
    }

    // Show what will be added
    println!("{} Adding the following tools:", style("→").cyan());
    println!();
    for (reg, tool) in &tools_to_add {
        println!("  {} {} ({}) - {}", 
            style("•").green(), 
            style(&tool.name).bold(),
            style(reg).dim(),
            tool.description
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
    for (reg_name, tool) in &tools_to_add {
        let adapter = get_registry_adapter(Some(reg_name))?;
        let registry_url = adapter.get_registry_url(&tool.image_name);
        
        add_tool_to_spin(&mut doc, &tool.name, &registry_url)?;
    }

    // Write back to spin.toml
    fs::write("spin.toml", doc.to_string())
        .context("Failed to write spin.toml")?;

    println!();
    println!("{} Successfully added tools to spin.toml", style("✓").green());
    println!("{} Remember to run 'ftl build' to pull the components", style("→").cyan());

    Ok(())
}

fn add_tool_to_spin(doc: &mut DocumentMut, tool_name: &str, registry_url: &str) -> Result<()> {
    // Create component entry
    let mut component = Table::new();
    
    let mut source = Table::new();
    source["registry"] = toml_edit::value(registry_url);
    component["source"] = Item::Table(source);
    
    // Add component to document
    doc[&format!("component.{}", tool_name)] = Item::Table(component);
    
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