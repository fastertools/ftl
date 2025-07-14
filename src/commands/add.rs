use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use atty;
use console::style;
use dialoguer::{Input, Select, theme::ColorfulTheme};

use crate::{common::spin_installer::check_and_install_spin, language::Language};

pub struct AddOptions {
    pub name: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub dir: Option<PathBuf>,
    pub tar: Option<String>,
}

pub async fn execute(options: AddOptions) -> Result<()> {
    let AddOptions {
        name,
        description,
        language,
        git,
        branch,
        dir,
        tar,
    } = options;
    
    // Check if we're in a Spin project directory
    if !PathBuf::from("spin.toml").exists() {
        anyhow::bail!("No spin.toml found. Not in a Spin project directory? Run 'ftl init' first.");
    }

    // Get component name interactively if not provided
    let component_name = match name {
        Some(n) => n,
        None => Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Tool name")
            .interact_text()?,
    };

    println!(
        "{} Adding tool: {}",
        style("→").cyan(),
        style(&component_name).bold()
    );

    // Validate component name
    if !component_name
        .chars()
        .all(|c| c.is_lowercase() || c == '-' || c == '_' || c.is_numeric())
    {
        anyhow::bail!("Tool name must be lowercase with hyphens or underscores (e.g., my-tool, my_tool)");
    }

    // Don't allow leading or trailing hyphens/underscores, or double hyphens/underscores
    if component_name.starts_with('-')
        || component_name.starts_with('_')
        || component_name.ends_with('-')
        || component_name.ends_with('_')
        || component_name.contains("--")
        || component_name.contains("__")
    {
        anyhow::bail!("Tool name cannot start or end with hyphens/underscores, or contain double hyphens/underscores");
    }

    // Get description interactively if not provided
    let description = match description {
        Some(d) => d,
        None => {
            if atty::is(atty::Stream::Stdin) {
                Input::<String>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Tool description")
                    .interact_text()?
            } else {
                // Non-interactive mode - use default
                format!("MCP tool written in {}", language.as_ref().unwrap_or(&"Rust".to_string()))
            }
        }
    };

    // Determine language
    let selected_language = match language {
        Some(lang_str) => {
            let lang_lower = lang_str.to_lowercase();
            // Map javascript to typescript
            let mapped_lang = if lang_lower == "javascript" || lang_lower == "js" {
                "typescript"
            } else {
                &lang_lower
            };
            
            Language::from_str(mapped_lang).ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid language: {lang_str}. Valid options are: rust, typescript, javascript"
                )
            })?
        }
        None => {
            // Interactive language selection
            let languages = vec!["rust", "typescript"];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select programming language")
                .items(&languages)
                .default(0)
                .interact()?;

            Language::from_str(languages[selection]).unwrap()
        }
    };

    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // Use spin add with the appropriate ftl-mcp template
    let template_id = match selected_language {
        Language::Rust => "ftl-mcp-rust",
        Language::TypeScript | Language::JavaScript => "ftl-mcp-ts",
    };

    // Check if custom template source is provided
    let using_custom_template = git.is_some() || dir.is_some() || tar.is_some();

    let mut spin_cmd = Command::new(&spin_path);
    spin_cmd.args(["add"]);

    // Add template source options
    if let Some(git_url) = &git {
        spin_cmd.args(["--git", git_url]);
        if let Some(branch_name) = &branch {
            spin_cmd.args(["--branch", branch_name]);
        }
    } else if let Some(dir_path) = &dir {
        spin_cmd.args(["--dir", dir_path.to_str().unwrap()]);
    } else if let Some(tar_path) = &tar {
        spin_cmd.args(["--tar", tar_path]);
    } else {
        // Use default template
        spin_cmd.args(["-t", template_id]);
    }

    // The route is now automatically the tool name (no /mcp suffix)
    let route = format!("/{}", component_name.replace('_', "-"));

    spin_cmd.args([
        "--accept-defaults",
        "--value",
        &format!("tool-description={description}"),
        &component_name,
    ]);

    let output = spin_cmd.output().context("Failed to run spin add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check if templates need to be installed (only for default templates)
        if !using_custom_template
            && (stderr.contains("no such template") || stderr.contains("template not found"))
        {
            eprintln!();
            eprintln!("{} ftl-mcp templates not found.", style("✗").red());
            eprintln!();
            eprintln!("Please install the ftl-mcp templates by running:");
            eprintln!("  ftl setup templates");
            eprintln!();
            anyhow::bail!("ftl-mcp templates not installed");
        } else {
            anyhow::bail!("Failed to add tool:\n{}", stderr);
        }
    }

    // Update spin.toml to add the component to tool_components variable
    update_tool_components(&component_name)?;

    // Success message based on language
    let main_file = match selected_language {
        Language::Rust => format!("{component_name}/src/lib.rs"),
        Language::JavaScript | Language::TypeScript => format!("{component_name}/src/index.ts"),
    };

    println!(
        r#"
{} {} tool added successfully!

{} Tool location:
  └── {}/         # Tool source code

{} Edit {} to implement your tool logic

{} Build and run:
  ftl build       # Build all tools
  ftl up          # Start the MCP server

{} Your tool will be available at route: {}
"#,
        style("✓").green(),
        selected_language,
        style("📁").blue(),
        component_name,
        style("💡").bright(),
        style(main_file).cyan(),
        style("🔨").bright(),
        style("🚀").yellow(),
        style(route).cyan(),
    );

    Ok(())
}

/// Update the tool_components variable in spin.toml to include the new component
fn update_tool_components(component_name: &str) -> Result<()> {
    use toml_edit::{DocumentMut, InlineTable};
    
    // Read the spin.toml file
    let spin_toml_path = PathBuf::from("spin.toml");
    let content = std::fs::read_to_string(&spin_toml_path)
        .context("Failed to read spin.toml")?;
    
    // Parse as TOML document (preserves formatting)
    let mut doc = content.parse::<DocumentMut>()
        .context("Failed to parse spin.toml")?;
    
    // Navigate to variables.tool_components.default
    let variables = doc
        .get_mut("variables")
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| anyhow::anyhow!("No [variables] section found in spin.toml"))?;
    
    // Ensure tool_components exists
    if !variables.contains_key("tool_components") {
        let mut inline_table = InlineTable::new();
        inline_table.insert("default", "".into());
        variables["tool_components"] = toml_edit::Item::Value(inline_table.into());
    }
    
    // Get tool_components table
    let tool_components = variables
        .get_mut("tool_components")
        .ok_or_else(|| anyhow::anyhow!("Failed to get tool_components"))?;
    
    // Handle both inline table and regular table formats
    match tool_components {
        toml_edit::Item::Value(val) => {
            if let Some(table) = val.as_inline_table_mut() {
                update_component_list_in_table(table, component_name)?;
            } else {
                anyhow::bail!("tool_components is not a table");
            }
        }
        toml_edit::Item::Table(table) => {
            update_component_list_in_table(table, component_name)?;
        }
        _ => anyhow::bail!("tool_components has unexpected type"),
    }
    
    // Write back to file
    let updated_content = doc.to_string();
    std::fs::write(&spin_toml_path, updated_content)
        .context("Failed to write updated spin.toml")?;
    
    println!(
        "{} Updated tool_components in spin.toml",
        style("✓").green()
    );
    
    Ok(())
}

// Helper function to update component list in either table type
fn update_component_list_in_table<T>(table: &mut T, component_name: &str) -> Result<()>
where
    T: toml_edit::TableLike,
{
    // Get current value of "default"
    let current_value = table
        .get("default")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    // Parse existing components
    let mut component_list: Vec<String> = if current_value.is_empty() {
        vec![]
    } else {
        current_value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    
    // Add new component if not already present
    if !component_list.contains(&component_name.to_string()) {
        component_list.push(component_name.to_string());
    }
    
    // Update the value
    table.insert("default", component_list.join(",").into());
    
    Ok(())
}