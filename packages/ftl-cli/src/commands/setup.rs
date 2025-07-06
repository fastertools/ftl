use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use console::style;

use crate::common::spin_installer::check_and_install_spin;

pub async fn templates(
    force: bool,
    git: Option<String>,
    branch: Option<String>,
    dir: Option<PathBuf>,
    tar: Option<String>,
) -> Result<()> {
    println!("{} Managing FTL templates", style("→").cyan());

    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // Check if templates are already installed
    if !force {
        let list_output = Command::new(&spin_path)
            .args(["templates", "list"])
            .output()
            .context("Failed to list templates")?;

        let output_str = String::from_utf8_lossy(&list_output.stdout);
        let has_ftl_templates = output_str.contains("ftl-rust")
            || output_str.contains("ftl-typescript")
            || output_str.contains("ftl-javascript");

        if has_ftl_templates {
            println!("{} FTL templates are already installed", style("✓").green());
            println!();
            println!("Use --force to reinstall/update them");
            return Ok(());
        }
    }

    // Build install command based on provided options
    let mut install_cmd = Command::new(&spin_path);
    install_cmd.args(["templates", "install"]);

    if let Some(git_url) = &git {
        println!(
            "{} Installing templates from Git: {}",
            style("→").dim(),
            style(git_url).dim()
        );
        install_cmd.args(["--git", git_url]);
        if let Some(branch_name) = &branch {
            install_cmd.args(["--branch", branch_name]);
        }
    } else if let Some(dir_path) = &dir {
        println!(
            "{} Installing templates from directory: {}",
            style("→").dim(),
            style(dir_path.display()).dim()
        );
        install_cmd.args(["--dir", dir_path.to_str().unwrap()]);
    } else if let Some(tar_path) = &tar {
        println!(
            "{} Installing templates from tarball: {}",
            style("→").dim(),
            style(tar_path).dim()
        );
        install_cmd.args(["--tar", tar_path]);
    } else {
        // Default: install from bundled templates
        let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

        println!(
            "{} Installing FTL templates from {}",
            style("→").dim(),
            style(template_dir.display()).dim()
        );

        install_cmd.args(["--dir", template_dir.to_str().unwrap()]);
    }

    install_cmd.arg("--upgrade");

    let install_output = install_cmd
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

    // List installed FTL templates
    let list_output = Command::new(&spin_path)
        .args(["templates", "list"])
        .output()
        .context("Failed to list templates")?;

    let output_str = String::from_utf8_lossy(&list_output.stdout);
    println!("Available FTL templates:");
    for line in output_str.lines() {
        if line.contains("ftl-") {
            println!("  {}", line.trim());
        }
    }

    Ok(())
}

pub async fn info() -> Result<()> {
    println!("{} FTL Configuration", style("→").cyan());
    println!();

    // Show version
    println!("FTL CLI version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    // Check spin installation
    match crate::common::spin_installer::get_spin_path() {
        Ok(spin_path) => {
            println!(
                "Spin: {} {}",
                style("✓").green(),
                style(spin_path.display()).dim()
            );

            // Get spin version
            if let Ok(output) = Command::new(&spin_path).arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("  Version: {}", version.trim());
            }
        }
        Err(_) => {
            println!("Spin: {} Not installed", style("✗").red());
            println!("  Run 'ftl setup templates' to install");
        }
    }
    println!();

    // Check templates
    if let Ok(spin_path) = crate::common::spin_installer::get_spin_path() {
        if let Ok(output) = Command::new(&spin_path)
            .args(["templates", "list"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let ftl_templates: Vec<&str> = output_str
                .lines()
                .filter(|line| line.contains("ftl-"))
                .collect();

            if ftl_templates.is_empty() {
                println!("FTL Templates: {} Not installed", style("✗").red());
                println!("  Run 'ftl setup templates' to install");
            } else {
                println!("FTL Templates: {} Installed", style("✓").green());
                for template in ftl_templates {
                    println!("  - {}", template.trim());
                }
            }
        }
    }
    println!();

    // Check for cargo-component
    match Command::new("cargo")
        .args(["component", "--version"])
        .output()
    {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("cargo-component: {} {}", style("✓").green(), version.trim());
        }
        Err(_) => {
            println!("cargo-component: {} Not installed", style("✗").red());
            println!("  Required for building Rust components");
            println!("  Will be installed automatically when building Rust components");
        }
    }
    println!();

    // Check for wkg
    match Command::new("wkg").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("wkg: {} {}", style("✓").green(), version.trim());
        }
        Err(_) => {
            println!("wkg: {} Not installed", style("✗").red());
            println!("  Required for 'ftl publish'");
            println!("  Install from: https://github.com/bytecodealliance/wasm-pkg-tools");
        }
    }

    Ok(())
}
