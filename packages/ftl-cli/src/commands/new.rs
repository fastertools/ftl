use std::path::PathBuf;

use anyhow::Result;
use console::style;
use dialoguer::{Input, Select, theme::ColorfulTheme};

use crate::{language::{Language, get_language_support}, templates};

pub async fn execute(name: String, description: Option<String>, language: Option<String>) -> Result<()> {
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

    // Determine language
    let selected_language = match language {
        Some(lang_str) => {
            Language::from_str(&lang_str)
                .ok_or_else(|| anyhow::anyhow!("Invalid language: {}. Valid options are: rust, javascript", lang_str))?
        }
        None => {
            // Interactive language selection
            let languages = vec!["rust", "javascript"];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select programming language")
                .items(&languages)
                .default(0)
                .interact()?;
            
            Language::from_str(languages[selection]).unwrap()
        }
    };

    // Determine target directory
    let target_dir = PathBuf::from(&name);
    if target_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create tool using language-specific support
    let language_support = get_language_support(selected_language);
    
    // Use templates for Rust (existing), or language-specific for others
    match selected_language {
        Language::Rust => {
            templates::create_tool(&name, &description, &target_dir)?;
        }
        Language::JavaScript => {
            language_support.new_project(&name, &description, "default", &target_dir)?;
        }
    }

    // Success message based on language
    let main_file = match selected_language {
        Language::Rust => "src/lib.rs",
        Language::JavaScript => "src/index.js",
    };
    
    println!(
        r#"
{} {} tool created successfully!

Next steps:
  1. cd {}
  2. ftl build      # Build your tool
  3. ftl test       # Run the included tests
  4. ftl serve      # Start development server

Then edit {} to implement your tool logic!

Other commands:
  ftl deploy      # Deploy to FTL Edge
  ftl validate    # Validate tool configuration
  ftl size        # Show binary size details"#,
        style("✓").green(),
        selected_language,
        name,
        main_file
    );

    Ok(())
}
