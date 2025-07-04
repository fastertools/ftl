use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use console::style;
use dialoguer::{Input, Select, theme::ColorfulTheme};

use crate::{
    language::Language,
    common::spin_installer::check_and_install_spin,
};

pub async fn execute(
    name: String,
    description: Option<String>,
    language: Option<String>,
) -> Result<()> {
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
        Some(lang_str) => Language::from_str(&lang_str).ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid language: {lang_str}. Valid options are: rust, javascript, typescript"
            )
        })?,
        None => {
            // Interactive language selection
            let languages = vec!["rust", "javascript", "typescript"];
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
        anyhow::bail!("Directory '{name}' already exists");
    }

    // Get spin path
    let spin_path = tokio::runtime::Handle::try_current()
        .ok()
        .and_then(|handle| {
            tokio::task::block_in_place(|| handle.block_on(check_and_install_spin()).ok())
        })
        .unwrap_or_else(|| {
            // If no runtime exists, create one
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(check_and_install_spin())
                .expect("Failed to install Spin")
        });

    // Use spin new with the appropriate template
    let template_id = match selected_language {
        Language::Rust => "ftl-rust",
        Language::TypeScript => "ftl-typescript",
        Language::JavaScript => "ftl-javascript",
    };

    let template_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("templates")
        .join(template_id);

    // If template doesn't exist in binary dir, use the source templates
    let template_path = if !template_path.exists() {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/templates")
            .join(template_id)
    } else {
        template_path
    };

    let output = Command::new(&spin_path)
        .args([
            "new",
            "-t",
            template_path.to_str().unwrap(),
            "-o",
            target_dir.to_str().unwrap(),
            "--accept-defaults",
            &name,
        ])
        .env("project-description", &description)
        .output()
        .context("Failed to run spin new")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create project with spin new:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Success message based on language
    let main_file = match selected_language {
        Language::Rust => "handler/src/lib.rs",
        Language::JavaScript => "handler/src/index.js",
        Language::TypeScript => "handler/src/index.ts",
    };

    println!(
        r#"
{} {selected_language} tool created successfully!

Next steps:
  1. cd {name}
  2. ftl build      # Build your tool
  3. ftl test       # Run the included tests
  4. ftl serve      # Start development server

Then edit {main_file} to implement your tool logic!

Other commands:
  ftl deploy      # Deploy to FTL Edge
  ftl validate    # Validate tool configuration
  ftl size        # Show binary size details"#,
        style("✓").green()
    );

    Ok(())
}
