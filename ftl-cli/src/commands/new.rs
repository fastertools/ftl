use std::path::PathBuf;

use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};

use crate::templates;

pub async fn execute(name: String, description: Option<String>) -> Result<()> {
    println!(
        "{} Creating new tool: {}",
        style("→").cyan(),
        style(&name).bold()
    );

    // Validate tool name
    if !name
        .chars()
        .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
    {
        anyhow::bail!("Tool name must be lowercase with hyphens (e.g., my-tool)");
    }

    // Don't allow leading or trailing hyphens, or double hyphens
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        anyhow::bail!("Tool name cannot start or end with hyphens, or contain double hyphens");
    }

    // Get description interactively if not provided
    let description = match description {
        Some(d) => d,
        None => Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Tool description")
            .interact_text()?,
    };

    // Determine target directory
    let target_dir = PathBuf::from(&name);
    if target_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create tool from template
    templates::create_tool(&name, &description, &target_dir)?;

    // Success message
    println!(
        r#"
{} Tool created successfully!

Next steps:
  1. cd {}
  2. ftl build      # Build your tool
  3. ftl test       # Run the included tests
  4. ftl serve      # Start development server

Then edit src/lib.rs to implement your tool logic!

Other commands:
  ftl deploy      # Deploy to FTL Edge
  ftl validate    # Validate tool configuration
  ftl size        # Show binary size details"#,
        style("✓").green(),
        name
    );

    Ok(())
}
