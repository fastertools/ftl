//! Refactored add command with dependency injection for better testability

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};

use ftl_common::SpinInstaller;
use ftl_language::Language;
use ftl_runtime::deps::{CommandExecutor, FileSystem, MessageStyle, UserInterface};

/// Add command configuration
#[derive(Debug, Clone)]
pub struct AddConfig {
    /// Name of the component to add
    pub name: Option<String>,
    /// Language to use for the component
    pub language: Option<String>,
    /// Git repository URL for templates
    pub git: Option<String>,
    /// Git branch to use
    pub branch: Option<String>,
    /// Directory containing templates
    pub dir: Option<PathBuf>,
    /// Tar file URL for templates
    pub tar: Option<String>,
}

/// Dependencies for the add command
pub struct AddDependencies {
    /// File system operations
    pub file_system: Arc<dyn FileSystem>,
    /// Command executor for running external commands
    pub command_executor: Arc<dyn CommandExecutor>,
    /// User interface for interaction
    pub ui: Arc<dyn UserInterface>,
    /// Spin installer for ensuring Spin is available
    pub spin_installer: Arc<dyn SpinInstaller>,
}

/// Execute the add command with injected dependencies
pub async fn execute_with_deps(config: AddConfig, deps: Arc<AddDependencies>) -> Result<()> {
    // Check if we have ftl.toml first
    let has_ftl_toml = deps.file_system.exists(Path::new("ftl.toml"));

    // Check if we're in a project directory
    if !has_ftl_toml && !deps.file_system.exists(Path::new("spin.toml")) {
        anyhow::bail!(
            "No spin.toml or ftl.toml found. Not in a project directory? Run 'ftl init' first."
        );
    }

    // Get component name interactively if not provided
    let component_name = match config.name {
        Some(n) => n,
        None => deps.ui.prompt_input("Tool name", None)?,
    };

    deps.ui.print(&format!("→ Adding tool: {component_name}"));

    // Validate component name
    validate_component_name(&component_name)?;

    // Determine language
    let selected_language = determine_language(config.language.as_ref(), &deps.ui)?;

    // Get spin path
    let spin_path = deps.spin_installer.check_and_install().await?;

    // If using ftl.toml, we need to temporarily generate spin.toml for spin add
    let temp_spin_created = if has_ftl_toml {
        // Generate spin.toml in the project directory temporarily
        crate::config::transpiler::ensure_spin_toml(&deps.file_system, &PathBuf::from("."))?;
        true
    } else {
        false
    };

    // Use spin add with the appropriate ftl-mcp template
    let template_id = match selected_language {
        Language::Rust => "ftl-mcp-rust",
        Language::TypeScript | Language::JavaScript => "ftl-mcp-ts",
    };

    // Check if custom template source is provided
    let using_custom_template =
        config.git.is_some() || config.dir.is_some() || config.tar.is_some();

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
    args.push(&component_name);

    // Execute spin add
    let output = deps
        .command_executor
        .execute(&spin_path, &args)
        .await
        .context("Failed to run spin add")?;

    if !output.success {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check if templates need to be installed (only for default templates)
        if !using_custom_template
            && (stderr.contains("no such template") || stderr.contains("template not found"))
        {
            deps.ui.print("");
            deps.ui
                .print_styled("✗ ftl-mcp templates not found.", MessageStyle::Error);
            deps.ui.print("");
            deps.ui
                .print("Please install the ftl-mcp templates by running:");
            deps.ui.print("  ftl setup templates");
            deps.ui.print("");
            anyhow::bail!("ftl-mcp templates not installed");
        }
        anyhow::bail!("Failed to add tool:\n{}", stderr);
    }

    // Update configuration - ftl.toml if it exists, otherwise spin.toml
    if has_ftl_toml {
        update_ftl_toml(&deps.file_system, &component_name, selected_language)?;
    } else {
        // Update spin.toml to add the component to tool_components variable
        update_tool_components(&deps.file_system, &component_name)?;
    }

    // Success message
    print_success_message(&deps.ui, &component_name, selected_language);

    // Clean up temporary spin.toml if we created it
    if temp_spin_created {
        let _ = std::fs::remove_file("spin.toml");
    }

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
fn determine_language(language: Option<&String>, ui: &Arc<dyn UserInterface>) -> Result<Language> {
    if let Some(lang_str) = language {
        let lang_lower = lang_str.to_lowercase();
        // Map javascript to typescript
        let mapped_lang = if lang_lower == "javascript" || lang_lower == "js" {
            "typescript"
        } else {
            &lang_lower
        };

        Language::from_str(mapped_lang).map_err(|_| {
            anyhow::anyhow!(
                "Invalid language: {}. Valid options are: rust, typescript, javascript",
                lang_str
            )
        })
    } else {
        // Interactive language selection
        let languages = vec!["rust", "typescript"];
        let selection = ui.prompt_select("Select programming language", &languages, 0)?;
        Language::from_str(languages[selection])
            .map_err(|e| anyhow::anyhow!("Failed to parse language: {}", e))
    }
}

/// Update ftl.toml to add the new tool
fn update_ftl_toml(
    fs: &Arc<dyn FileSystem>,
    component_name: &str,
    language: Language,
) -> Result<()> {
    use crate::config::ftl_config::{BuildConfig, FtlConfig, ToolConfig};
    use std::collections::HashMap;

    // Read ftl.toml
    let content = fs
        .read_to_string(Path::new("ftl.toml"))
        .context("Failed to read ftl.toml")?;

    // Parse config
    let mut config = FtlConfig::parse(&content)?;

    // Create build configuration with explicit defaults based on language
    let build = match language {
        Language::Rust => BuildConfig {
            command: "cargo build --target wasm32-wasip1 --release".to_string(),
            workdir: None,
            watch: vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()],
            env: HashMap::new(),
        },
        Language::TypeScript | Language::JavaScript => BuildConfig {
            command: "npm install && npm run build".to_string(),
            workdir: None,
            watch: vec![
                "src/**/*.ts".to_string(),
                "src/**/*.js".to_string(),
                "package.json".to_string(),
                "tsconfig.json".to_string(),
            ],
            env: HashMap::new(),
        },
    };

    // Add the new tool
    config.tools.insert(
        component_name.to_string(),
        ToolConfig {
            path: component_name.to_string(),
            build,
            allowed_outbound_hosts: vec![],
            variables: HashMap::new(),
        },
    );

    // Write back
    let updated_content = config.to_toml_string()?;
    fs.write_string(Path::new("ftl.toml"), &updated_content)
        .context("Failed to write updated ftl.toml")?;

    Ok(())
}

/// Update the `tool_components` variable in spin.toml to include the new component
fn update_tool_components(fs: &Arc<dyn FileSystem>, component_name: &str) -> Result<()> {
    use toml_edit::{DocumentMut, InlineTable};

    // Read the spin.toml file
    let content = fs
        .read_to_string(Path::new("spin.toml"))
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
                update_component_list_in_table(table, component_name);
            } else {
                anyhow::bail!("tool_components is not a table");
            }
        }
        toml_edit::Item::Table(table) => {
            update_component_list_in_table(table, component_name);
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
fn update_component_list_in_table<T>(table: &mut T, component_name: &str)
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
}

/// Print success message
fn print_success_message(ui: &Arc<dyn UserInterface>, component_name: &str, language: Language) {
    let main_file = match language {
        Language::Rust => format!("{component_name}/src/lib.rs"),
        Language::JavaScript | Language::TypeScript => format!("{component_name}/src/index.ts"),
    };

    ui.print("");
    ui.print_styled(
        &format!("✓ {language} tool added successfully!"),
        MessageStyle::Success,
    );
    ui.print("");
    ui.print("📁 Tool location:");
    ui.print(&format!(
        "  └── {component_name}/         # Tool source code"
    ));
    ui.print("");
    ui.print(&format!("💡 Edit {main_file} to implement your tool logic"));
    ui.print("");
    ui.print("🔨 Build and run:");
    ui.print("  ftl build       # Build all tools");
    ui.print("  ftl up          # Start the MCP server");
    ui.print("");
}

/// Add command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct AddArgs {
    /// Name of the tool to add
    pub name: Option<String>,
    /// Programming language
    pub language: Option<String>,
    /// Git repository URL for custom template
    pub git: Option<String>,
    /// Git branch for custom template
    pub branch: Option<String>,
    /// Local directory path for custom template
    pub dir: Option<PathBuf>,
    /// Tarball path for custom template
    pub tar: Option<String>,
}

// Spin installer wrapper
struct SpinInstallerWrapper;

#[async_trait::async_trait]
impl SpinInstaller for SpinInstallerWrapper {
    async fn check_and_install(&self) -> Result<String> {
        let path = ftl_common::check_and_install_spin().await?;
        Ok(path.to_string_lossy().to_string())
    }
}

/// Execute the add command with default dependencies
pub async fn execute(args: AddArgs) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_runtime::deps::{RealCommandExecutor, RealFileSystem};

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(AddDependencies {
        file_system: Arc::new(RealFileSystem),
        command_executor: Arc::new(RealCommandExecutor),
        ui: ui.clone(),
        spin_installer: Arc::new(SpinInstallerWrapper),
    });

    let config = AddConfig {
        name: args.name,
        language: args.language,
        git: args.git,
        branch: args.branch,
        dir: args.dir,
        tar: args.tar,
    };

    execute_with_deps(config, deps).await
}

#[cfg(test)]
#[path = "add_tests.rs"]
mod tests;
