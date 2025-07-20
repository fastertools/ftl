use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table};

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

        /// Show additional details
        #[arg(short, long)]
        verbose: bool,
    },

    /// Add pre-built tools to your project
    Add {
        /// Tool names to add
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

#[derive(Debug, Deserialize)]
struct Tool {
    name: String,
    category: String,
    description: String,
    registry_url: String,
    tags: Vec<String>,
}

pub async fn handle_command(cmd: ToolsCommand) -> Result<()> {
    match cmd {
        ToolsCommand::List { category, filter, verbose } => {
            list_tools(category, filter, verbose).await
        }
        ToolsCommand::Add { tools, yes } => {
            add_tools(tools, yes).await
        }
    }
}

async fn list_tools(category: Option<String>, filter: Option<String>, verbose: bool) -> Result<()> {
    let manifest: ToolsManifest = toml::from_str(TOOLS_MANIFEST)
        .context("Failed to parse tools manifest")?;

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
            println!("    Registry: {}", style(&tool.registry_url).dim());
            println!("    Tags: {}", tool.tags.join(", "));
        }
    }

    println!("\n{} Use 'ftl tools add <tool>' to add tools to your project", 
             style("→").cyan());

    Ok(())
}

async fn add_tools(tool_names: Vec<String>, skip_confirm: bool) -> Result<()> {
    // Load tools manifest
    let manifest: ToolsManifest = toml::from_str(TOOLS_MANIFEST)
        .context("Failed to parse tools manifest")?;

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
        add_tool_to_spin_toml(&mut doc, tool)?;
    }

    // Write back to file
    fs::write("spin.toml", doc.to_string())
        .context("Failed to write spin.toml")?;

    println!("\n{} Successfully added tools to spin.toml", style("✓").green());
    println!("{} Remember to run 'ftl build' to pull the components", style("→").cyan());

    Ok(())
}

fn add_tool_to_spin_toml(doc: &mut DocumentMut, tool: &Tool) -> Result<()> {
    // Create component table
    let mut component = Table::new();
    component["id"] = toml_edit::value(&tool.name);
    
    // Create source table for registry reference
    let mut source = Table::new();
    source["registry"] = toml_edit::value(&tool.registry_url);
    component["source"] = Item::Table(source);
    
    // Set route - tools are accessed via internal routing
    component["route"] = toml_edit::value(format!("/{}...", &tool.name));

    // Add to components array
    if let Some(components) = doc.get_mut("component").and_then(|c| c.as_array_of_tables_mut()) {
        components.push(component);
    } else {
        // Create component array if it doesn't exist
        let mut array = toml_edit::ArrayOfTables::new();
        array.push(component);
        doc["component"] = Item::ArrayOfTables(array);
    }

    // Update tool_components variable in mcp-gateway
    if let Some(gateway) = doc.get_mut("component")
        .and_then(|c| c.as_table_mut())
        .and_then(|t| t.get_mut("mcp-gateway"))
        .and_then(|g| g.as_table_mut()) {
        
        if let Some(vars) = gateway.get_mut("variables").and_then(|v| v.as_table_mut()) {
            if let Some(tool_components) = vars.get_mut("tool_components").and_then(|tc| tc.as_array_mut()) {
                // Add to existing array
                tool_components.push(&tool.name);
            } else {
                // Create new array
                let mut array = toml_edit::Array::new();
                array.push(&tool.name);
                vars["tool_components"] = toml_edit::value(array);
            }
        }
    }

    Ok(())
}