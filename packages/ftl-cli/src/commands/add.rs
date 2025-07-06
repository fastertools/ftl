use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use console::style;
use dialoguer::{Input, Select, theme::ColorfulTheme};

use crate::{common::spin_installer::check_and_install_spin, language::Language};

pub struct AddOptions {
    pub name: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub route: Option<String>,
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
        route,
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
            .with_prompt("Component name")
            .interact_text()?,
    };

    println!(
        "{} Adding component: {}",
        style("→").cyan(),
        style(&component_name).bold()
    );

    // Validate component name
    if !component_name
        .chars()
        .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
    {
        anyhow::bail!("Component name must be lowercase with hyphens (e.g., my-component)");
    }

    // Don't allow leading or trailing hyphens, or double hyphens
    if component_name.starts_with('-')
        || component_name.ends_with('-')
        || component_name.contains("--")
    {
        anyhow::bail!("Component name cannot start or end with hyphens, or contain double hyphens");
    }

    // Get description interactively if not provided
    let description = match description {
        Some(d) => d,
        None => Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Component description")
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

    // Get route interactively if not provided
    let route = match route {
        Some(r) => {
            // Ensure route ends with /mcp
            if r.ends_with("/mcp") {
                r
            } else if r.ends_with('/') {
                format!("{r}mcp")
            } else {
                format!("{r}/mcp")
            }
        }
        None => {
            // Convert component name to kebab-case for the route
            let kebab_name = component_name.replace('_', "-").to_lowercase();
            let default_route = format!("/{kebab_name}/mcp");
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("HTTP route")
                .default(default_route)
                .interact_text()?
        }
    };

    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // Use spin add with the appropriate template
    let template_id = match selected_language {
        Language::Rust => "ftl-rust",
        Language::TypeScript => "ftl-typescript",
        Language::JavaScript => "ftl-javascript",
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

    spin_cmd.args([
        "--accept-defaults",
        "--value",
        &format!("project-description={description}"),
        "--value",
        &format!("route={route}"),
        &component_name,
    ]);

    let output = spin_cmd.output().context("Failed to run spin add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check if templates need to be installed (only for default templates)
        if !using_custom_template
            && (stderr.contains("no such template") || stderr.contains("template not found"))
        {
            println!();
            println!(
                "{} FTL templates not found. Installing...",
                style("→").yellow()
            );

            // Install templates
            let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

            let install_output = Command::new(&spin_path)
                .args([
                    "templates",
                    "install",
                    "--dir",
                    template_dir.to_str().unwrap(),
                    "--upgrade",
                ])
                .output()
                .context("Failed to install templates")?;

            if !install_output.status.success() {
                anyhow::bail!(
                    "Failed to install templates:\n{}",
                    String::from_utf8_lossy(&install_output.stderr)
                );
            }

            println!("{} Templates installed successfully!", style("✓").green());
            println!();

            // Retry spin add
            let retry_output = spin_cmd
                .output()
                .context("Failed to run spin add after template installation")?;

            if !retry_output.status.success() {
                anyhow::bail!(
                    "Failed to add component:\n{}",
                    String::from_utf8_lossy(&retry_output.stderr)
                );
            }
        } else {
            anyhow::bail!("Failed to add component:\n{}", stderr);
        }
    }

    // Success message based on language
    let main_file = match selected_language {
        Language::Rust => format!("{component_name}/src/lib.rs"),
        Language::JavaScript => format!("{component_name}/src/index.js"),
        Language::TypeScript => format!("{component_name}/src/index.ts"),
    };

    println!(
        r#"
{} {} component added successfully!

{} Component location:
  └── {}/         # Component source code

{} Edit {} to implement your MCP features

{} cd {} && make build # Build component
 
{} ftl up --build # Build all components and start development server"#,
        style("✓").green(),
        selected_language,
        style("📁").blue(),
        component_name,
        style("💡").bright(),
        style(main_file).cyan(),
        style("🔨").bright(),
        component_name,
        style("🚀").yellow(),
    );

    Ok(())
}
