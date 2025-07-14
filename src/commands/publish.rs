use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use console::style;

use crate::common::spin_installer::check_and_install_spin;

pub async fn execute(
    path: Option<PathBuf>,
    registry: Option<String>,
    tag: Option<String>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    println!("{} Publishing project", style("→").cyan());

    // Validate we're in a Spin project directory
    if !project_path.join("spin.toml").exists() {
        anyhow::bail!("No spin.toml found. Not in a project directory? Run 'ftl init' to create a new project.");
    }

    // For now, we'll use spin registry push to publish the entire application
    // In the future, we might want to support publishing individual tools
    
    let spin_path = check_and_install_spin().await?;
    
    // Build the project first
    println!("{} Building project...", style("→").dim());
    crate::commands::build::execute(Some(project_path.clone()), true).await?;
    
    // Prepare registry push arguments
    let mut args = vec!["registry", "push"];
    
    if let Some(registry_url) = registry.as_ref() {
        args.push("--registry");
        args.push(registry_url);
    }
    
    if let Some(version_tag) = tag.as_ref() {
        args.push("--tag");
        args.push(version_tag);
    }
    
    println!("{} Publishing to registry...", style("→").dim());
    
    let output = Command::new(&spin_path)
        .args(&args)
        .current_dir(&project_path)
        .output()
        .context("Failed to run spin registry push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        anyhow::bail!("Publishing failed:\n{}\n{}", stdout, stderr);
    }

    println!("{} Project published successfully!", style("✓").green());
    
    // Print any useful output from spin
    let output_str = String::from_utf8_lossy(&output.stdout);
    if !output_str.trim().is_empty() {
        println!("{}", output_str);
    }
    
    Ok(())
}