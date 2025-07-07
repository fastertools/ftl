use anyhow::{Context, Result};
use dialoguer::Confirm;
use std::process::Command;
use tracing::{debug, info};

/// Check if a specific Spin plugin is installed
pub async fn check_spin_plugin(plugin_name: &str) -> Result<bool> {
    let spin_path = which::which("spin")
        .context("Spin is not installed. Please install Spin from https://developer.fermyon.com/spin/install")?;
    
    let output = Command::new(&spin_path)
        .args(["plugins", "list"])
        .output()
        .context("Failed to list Spin plugins")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list plugins: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("Plugin list output: {}", stdout);
    
    // Check if the plugin is in the list
    Ok(stdout.lines().any(|line| line.contains(plugin_name)))
}

/// Check if a specific Spin template is available
pub async fn check_spin_template(template_name: &str) -> Result<bool> {
    let spin_path = which::which("spin")
        .context("Spin is not installed. Please install Spin from https://developer.fermyon.com/spin/install")?;
    
    let output = Command::new(&spin_path)
        .args(["templates", "list"])
        .output()
        .context("Failed to list Spin templates")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list templates: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("Template list output: {}", stdout);
    
    // Check if the template is in the list
    Ok(stdout.lines().any(|line| line.contains(template_name)))
}

/// Install a Spin plugin
pub async fn install_spin_plugin(plugin_name: &str) -> Result<()> {
    let spin_path = which::which("spin")
        .context("Spin is not installed. Please install Spin from https://developer.fermyon.com/spin/install")?;
    
    info!("Installing {} plugin...", plugin_name);
    
    let output = Command::new(&spin_path)
        .args(["plugins", "install", plugin_name])
        .output()
        .context(format!("Failed to install {} plugin", plugin_name))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to install {} plugin: {}", plugin_name, stderr);
    }
    
    info!("{} plugin installed successfully", plugin_name);
    Ok(())
}

/// Install a Spin template
pub async fn install_spin_template(template_name: &str) -> Result<()> {
    let spin_path = which::which("spin")
        .context("Spin is not installed. Please install Spin from https://developer.fermyon.com/spin/install")?;
    
    info!("Installing {} template...", template_name);
    
    // Implement once we can point to remote templates

    Ok(())
}

/// Ensure required plugins and templates are installed with user confirmation
pub async fn ensure_spin_plugins() -> Result<()> {
    // Check and install trigger-mcp plugin if needed
    if !check_spin_plugin("trigger-mcp").await? {
        let prompt = Confirm::new()
            .with_prompt("The 'trigger-mcp' plugin is required for FTL. Install it now?")
            .default(true)
            .interact()?;
        
        if prompt {
            install_spin_plugin("trigger-mcp").await?;
        } else {
            anyhow::bail!("The 'trigger-mcp' plugin is required to create FTL tools");
        }
    }
    
    // Check and install mcp-rust template if needed
    if !check_spin_template("mcp-rust").await? {
        let prompt = Confirm::new()
            .with_prompt("The 'mcp-rust' template is required for FTL. Install it now?")
            .default(true)
            .interact()?;
        
        if prompt {
            install_spin_template("mcp-rust").await?;
        } else {
            anyhow::bail!("The 'mcp-rust' template is required to create FTL tools");
        }
    }

        // Check and install mcp-rust template if needed
    if !check_spin_template("toolkit-pointer").await? {
        let prompt = Confirm::new()
            .with_prompt("The 'toolkit-pointer' template is required for FTL. Install it now?")
            .default(true)
            .interact()?;
        
        if prompt {
            install_spin_template("toolkit-pointer").await?;
        } else {
            anyhow::bail!("The 'mcp-rust' template is required to create FTL tools");
        }
    }
    
    Ok(())
}