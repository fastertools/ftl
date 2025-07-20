//! Refactored add command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::deps::{
    CommandExecutor, FileSystem, UserInterface, SpinInstaller, MessageStyle
};
use crate::language::Language;

/// Add command configuration
#[derive(Debug, Clone)]
pub struct AddConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub dir: Option<PathBuf>,
    pub tar: Option<String>,
}

/// Dependencies for the add command
pub struct AddDependencies {
    pub file_system: Arc<dyn FileSystem>,
    pub command_executor: Arc<dyn CommandExecutor>,
    pub ui: Arc<dyn UserInterface>,
    pub spin_installer: Arc<dyn SpinInstaller>,
}

/// Execute the add command with injected dependencies
pub async fn execute_with_deps(
    config: AddConfig,
    deps: Arc<AddDependencies>,
) -> Result<()> {
    // Check if we're in a Spin project directory
    if !deps.file_system.exists(Path::new("spin.toml")) {
        anyhow::bail!("No spin.toml found. Not in a Spin project directory? Run 'ftl init' first.");
    }

    // Get component name interactively if not provided
    let component_name = match config.name {
        Some(n) => n,
        None => deps.ui.prompt_input("Tool name", None)?,
    };

    deps.ui.print(&format!("→ Adding tool: {}", component_name));

    // Validate component name
    validate_component_name(&component_name)?;

    // Get description interactively if not provided
    let description = match config.description {
        Some(d) => d,
        None => {
            if deps.ui.is_interactive() {
                deps.ui.prompt_input("Tool description", None)?
            } else {
                // Non-interactive mode - use default
                format!(
                    "MCP tool written in {}",
                    config.language.as_ref().unwrap_or(&"Rust".to_string())
                )
            }
        }
    };

    // Determine language
    let selected_language = determine_language(&config.language, &deps.ui)?;

    // Get spin path
    let spin_path = deps.spin_installer.check_and_install().await?;
    deps.ui.print(&format!("Using Spin at: {}", spin_path));

    // Use spin add with the appropriate ftl-mcp template
    let template_id = match selected_language {
        Language::Rust => "ftl-mcp-rust",
        Language::TypeScript | Language::JavaScript => "ftl-mcp-ts",
    };

    // Check if custom template source is provided
    let using_custom_template = config.git.is_some() || config.dir.is_some() || config.tar.is_some();

    // Build spin add command
    let mut args = vec!["add"];
    
    // Add template source options
    if let Some(git_url) = &config.git {
        args.push("--git");
        args.push(git_url);
        if let Some(branch_name) = &config.branch {
            args.push("--branch");
            args.push(branch_name);
        }
    } else if let Some(dir_path) = &config.dir {
        args.push("--dir");
        args.push(dir_path.to_str().unwrap());
    } else if let Some(tar_path) = &config.tar {
        args.push("--tar");
        args.push(tar_path);
    } else {
        // Use default template
        args.push("-t");
        args.push(template_id);
    }

    args.push("--accept-defaults");
    args.push("--value");
    let desc_value = format!("tool-description={}", description);
    args.push(&desc_value);
    args.push(&component_name);

    // Execute spin add
    let output = deps.command_executor.execute(&spin_path, &args).await
        .context("Failed to run spin add")?;

    if !output.success {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check if templates need to be installed (only for default templates)
        if !using_custom_template
            && (stderr.contains("no such template") || stderr.contains("template not found"))
        {
            deps.ui.print("");
            deps.ui.print_styled("✗ ftl-mcp templates not found.", MessageStyle::Error);
            deps.ui.print("");
            deps.ui.print("Please install the ftl-mcp templates by running:");
            deps.ui.print("  ftl setup templates");
            deps.ui.print("");
            anyhow::bail!("ftl-mcp templates not installed");
        } else {
            anyhow::bail!("Failed to add tool:\n{}", stderr);
        }
    }

    // Update spin.toml to add the component to tool_components variable
    update_tool_components(&deps.file_system, &component_name)?;

    // Success message
    print_success_message(&deps.ui, &component_name, selected_language);

    Ok(())
}

/// Validate component name
fn validate_component_name(name: &str) -> Result<()> {
    if !name
        .chars()
        .all(|c| c.is_lowercase() || c == '-' || c == '_' || c.is_numeric())
    {
        anyhow::bail!(
            "Tool name must be lowercase with hyphens or underscores (e.g., my-tool, my_tool)"
        );
    }

    // Don't allow leading or trailing hyphens/underscores, or double hyphens/underscores
    if name.starts_with('-')
        || name.starts_with('_')
        || name.ends_with('-')
        || name.ends_with('_')
        || name.contains("--")
        || name.contains("__")
    {
        anyhow::bail!(
            "Tool name cannot start or end with hyphens/underscores, or contain double hyphens/underscores"
        );
    }

    Ok(())
}

/// Determine the language to use
fn determine_language(
    language: &Option<String>,
    ui: &Arc<dyn UserInterface>,
) -> Result<Language> {
    match language {
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
                    "Invalid language: {}. Valid options are: rust, typescript, javascript",
                    lang_str
                )
            })
        }
        None => {
            // Interactive language selection
            let languages = vec!["rust", "typescript"];
            let selection = ui.prompt_select("Select programming language", &languages, 0)?;
            Ok(Language::from_str(languages[selection]).unwrap())
        }
    }
}

/// Update the tool_components variable in spin.toml to include the new component
fn update_tool_components(fs: &Arc<dyn FileSystem>, component_name: &str) -> Result<()> {
    use toml_edit::{DocumentMut, InlineTable};

    // Read the spin.toml file
    let content = fs.read_to_string(Path::new("spin.toml"))
        .context("Failed to read spin.toml")?;

    // Parse as TOML document (preserves formatting)
    let mut doc = content
        .parse::<DocumentMut>()
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
    fs.write_string(Path::new("spin.toml"), &updated_content)
        .context("Failed to write updated spin.toml")?;

    Ok(())
}

/// Helper function to update component list in either table type
fn update_component_list_in_table<T>(table: &mut T, component_name: &str) -> Result<()>
where
    T: toml_edit::TableLike,
{
    // Get current value of "default"
    let current_value = table.get("default").and_then(|v| v.as_str()).unwrap_or("");

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

/// Print success message
fn print_success_message(
    ui: &Arc<dyn UserInterface>,
    component_name: &str,
    language: Language,
) {
    let route = format!("/{}", component_name.replace('_', "-"));
    let main_file = match language {
        Language::Rust => format!("{}/src/lib.rs", component_name),
        Language::JavaScript | Language::TypeScript => format!("{}/src/index.ts", component_name),
    };

    ui.print("");
    ui.print_styled(&format!("✓ {} tool added successfully!", language), MessageStyle::Success);
    ui.print("");
    ui.print("📁 Tool location:");
    ui.print(&format!("  └── {}/         # Tool source code", component_name));
    ui.print("");
    ui.print(&format!("💡 Edit {} to implement your tool logic", main_file));
    ui.print("");
    ui.print("🔨 Build and run:");
    ui.print("  ftl build       # Build all tools");
    ui.print("  ftl up          # Start the MCP server");
    ui.print("");
    ui.print(&format!("🚀 Your tool will be available at route: {}", route));
    ui.print("");
}

#[cfg(test)]
#[path = "add_tests.rs"]
mod tests;