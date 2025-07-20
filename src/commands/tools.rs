use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table};

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

        /// Registry to use (ghcr, ecr)
        #[arg(short, long, default_value = "ghcr")]
        registry: String,

        /// Show additional details
        #[arg(short, long)]
        verbose: bool,
    },

    /// Add pre-built tools to your project
    Add {
        /// Tool names to add
        tools: Vec<String>,

        /// Registry to use (ghcr, ecr)
        #[arg(short, long, default_value = "ghcr")]
        registry: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Debug, Deserialize)]
struct ToolsManifest {
    tools: Vec<Tool>,
}

#[derive(Debug, Deserialize)]
struct Tool {
    name: String,
    category: String,
    description: String,
    image_name: String,
    tags: Vec<String>,
}

pub async fn handle_command(cmd: ToolsCommand) -> Result<()> {
    match cmd {
        ToolsCommand::List { category, filter, registry, verbose } => {
            list_tools(category, filter, registry, verbose).await
        }
        ToolsCommand::Add { tools, registry, yes } => {
            add_tools(tools, registry, yes).await
        }
    }
}

async fn list_tools(category: Option<String>, filter: Option<String>, registry: String, verbose: bool) -> Result<()> {
    let manifest: ToolsManifest = toml::from_str(TOOLS_MANIFEST)
        .context("Failed to parse tools manifest")?;
    
    // Get the registry adapter
    let adapter = get_registry_adapter(Some(&registry))?;

    // Apply filters
    let mut tools: Vec<&Tool> = manifest.tools.iter()
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

    // Sort by category then name
    tools.sort_by(|a, b| {
        a.category.cmp(&b.category)
            .then(a.name.cmp(&b.name))
    });

    if tools.is_empty() {
        println!("{} No tools found matching your criteria", style("!").yellow());
        return Ok(());
    }

    println!("{} Available FTL Tools\n", style("📦").cyan());

    let mut current_category = "";
    for tool in tools {
        // Print category header when it changes
        if tool.category != current_category {
            current_category = &tool.category;
            println!("\n{}", style(format!("[{}]", current_category)).bold().cyan());
        }

        // Print tool info
        print!("  {} ", style(&tool.name).green().bold());
        println!("- {}", tool.description);

        if verbose {
            let registry_url = adapter.get_registry_url(&tool.image_name);
            println!("    Registry: {}", style(&registry_url).dim());
            println!("    Tags: {}", tool.tags.join(", "));
        }
    }

    println!("\n{} Use 'ftl tools add <tool>' to add tools to your project", 
             style("→").cyan());

    Ok(())
}

async fn add_tools(tool_names: Vec<String>, registry: String, skip_confirm: bool) -> Result<()> {
    // Load tools manifest
    let manifest: ToolsManifest = toml::from_str(TOOLS_MANIFEST)
        .context("Failed to parse tools manifest")?;
    
    // Get the registry adapter
    let adapter = get_registry_adapter(Some(&registry))?;

    // Find requested tools
    let mut tools_to_add = Vec::new();
    let mut not_found = Vec::new();

    for name in &tool_names {
        if let Some(tool) = manifest.tools.iter().find(|t| t.name == *name) {
            tools_to_add.push(tool);
        } else {
            not_found.push(name.clone());
        }
    }

    if !not_found.is_empty() {
        println!("{} Tools not found: {}", 
                 style("✗").red(), 
                 not_found.join(", "));
        return Ok(());
    }

    // Check if spin.toml exists
    if !Path::new("spin.toml").exists() {
        println!("{} No spin.toml found in current directory", style("✗").red());
        println!("{} Run this command from your FTL project root", style("→").cyan());
        return Ok(());
    }

    // Show what will be added
    println!("{} Adding the following tools:\n", style("→").cyan());
    for tool in &tools_to_add {
        println!("  {} {} - {}", 
                 style("•").green(), 
                 style(&tool.name).bold(),
                 tool.description);
    }

    // Confirm if needed
    if !skip_confirm {
        println!("\n{} This will modify your spin.toml file.", style("!").yellow());
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Continue?")
            .default(true)
            .interact()?;

        if !confirm {
            println!("{} Cancelled", style("✗").red());
            return Ok(());
        }
    }

    // Read and parse spin.toml
    let spin_content = fs::read_to_string("spin.toml")
        .context("Failed to read spin.toml")?;
    let mut doc = spin_content.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;

    // Add each tool as a component
    for tool in tools_to_add {
        add_tool_to_spin_toml(&mut doc, tool, adapter.as_ref())?;
    }

    // Write back to file
    fs::write("spin.toml", doc.to_string())
        .context("Failed to write spin.toml")?;

    println!("\n{} Successfully added tools to spin.toml", style("✓").green());
    println!("{} Remember to run 'ftl build' to pull the components", style("→").cyan());

    Ok(())
}

fn add_tool_to_spin_toml(doc: &mut DocumentMut, tool: &Tool, adapter: &dyn RegistryAdapter) -> Result<()> {
    // First, update the tool_components variable at the top level
    let variables_table = doc
        .entry("variables")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("Failed to create variables table")?;
    
    // Get or create tool_components entry
    let tool_components_entry = variables_table
        .entry("tool_components")
        .or_insert_with(|| {
            let mut inline_table = toml_edit::InlineTable::new();
            inline_table.insert("default", "".into());
            toml_edit::value(inline_table)
        });
    
    // Update the tool_components list
    if let Some(tc_table) = tool_components_entry.as_inline_table_mut() {
        if let Some(default_value) = tc_table.get_mut("default") {
            let current = default_value.as_str().unwrap_or("");
            let tools: Vec<&str> = if current.is_empty() {
                vec![]
            } else {
                current.split(',').collect()
            };
            
            if !tools.contains(&tool.name.as_str()) {
                let new_value = if current.is_empty() {
                    tool.name.clone()
                } else {
                    format!("{},{}", current, tool.name)
                };
                *default_value = new_value.into();
            }
        }
    }
    
    // Add the component definition
    let component_table = doc
        .entry("component")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("Failed to create component table")?;
    
    // Create the tool component section [component.tool-name]
    let mut tool_component = Table::new();
    
    // Create inline source table for registry reference
    let mut source = toml_edit::InlineTable::new();
    let registry_url = adapter.get_registry_url(&tool.image_name);
    source.insert("registry", registry_url.into());
    tool_component["source"] = toml_edit::value(source);
    
    // Add the component
    component_table[&tool.name] = Item::Table(tool_component);
    
    Ok(())
}