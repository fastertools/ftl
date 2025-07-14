use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use console::style;
use dialoguer::{Input, theme::ColorfulTheme};

use crate::common::spin_installer::check_and_install_spin;

pub async fn execute(name: Option<String>, here: bool) -> Result<()> {
    // Get project name interactively if not provided
    let project_name = match name {
        Some(n) => n,
        None => Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Project name")
            .interact_text()?,
    };

    println!(
        "{} Initializing new MCP project: {}",
        style("→").cyan(),
        style(&project_name).bold()
    );

    // Validate project name
    if !project_name
        .chars()
        .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
    {
        anyhow::bail!("Project name must be lowercase with hyphens (e.g., my-project)");
    }

    // Don't allow leading or trailing hyphens, or double hyphens
    if project_name.starts_with('-') || project_name.ends_with('-') || project_name.contains("--") {
        anyhow::bail!("Project name cannot start or end with hyphens, or contain double hyphens");
    }

    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // Determine output directory
    let output_dir = if here {
        ".".to_string()
    } else {
        project_name.clone()
    };

    // Check if directory exists and is not empty (unless using --here)
    if !here && PathBuf::from(&output_dir).exists() {
        anyhow::bail!("Directory '{}' already exists", project_name);
    } else if here {
        let current_dir = std::env::current_dir()?;
        if current_dir.read_dir()?.next().is_some() {
            anyhow::bail!("Current directory is not empty");
        }
    }

    // First check if templates are installed
    let check_template_cmd = Command::new(&spin_path)
        .args(["templates", "list"])
        .output()
        .context("Failed to list templates")?;

    let templates_output = String::from_utf8_lossy(&check_template_cmd.stdout);
    let has_ftl_templates = templates_output.contains("ftl-mcp-server");

    if !has_ftl_templates {
        eprintln!();
        eprintln!("{} ftl-mcp templates not found.", style("✗").red());
        eprintln!();
        eprintln!("Please install the ftl-mcp templates by running:");
        eprintln!("  ftl setup templates");
        eprintln!();
        anyhow::bail!("ftl-mcp templates not installed");
    }

    // Use spin new with ftl-mcp-server template
    let mut spin_cmd = Command::new(&spin_path);
    spin_cmd.args([
        "new",
        "-t",
        "ftl-mcp-server",
        "-o",
        &output_dir,
        "--accept-defaults",
    ]);

    if !here {
        spin_cmd.arg(&project_name);
    }

    let output = spin_cmd.output().context("Failed to run spin new")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create project:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let cd_instruction = if here {
        ""
    } else {
        &format!("cd {project_name} && ")
    };

    println!(
        r#"
{} MCP project initialized!

{} Structure:
  └── spin.toml        # Spin configuration
  └── README.md        # Project documentation

{} MCP Gateway is pre-configured at route /mcp

{} Next steps:
  {}ftl add           # Add a tool to the project
  ftl build           # Build all tools
  ftl up              # Start the MCP server

{} Example:
  {}ftl add weather-api --language typescript
  {}ftl add calculator --language rust
  
{} Connect your MCP client to:
  http://localhost:3000/mcp"#,
        style("✓").green(),
        style("📁").blue(),
        style("🌐").cyan(),
        style("🚀").yellow(),
        cd_instruction,
        style("💡").bright(),
        cd_instruction,
        cd_instruction,
        style("🔗").magenta()
    );

    Ok(())
}