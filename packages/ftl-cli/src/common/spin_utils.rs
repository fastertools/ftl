use std::{
    path::Path,
    process::{Child, Command, Stdio},
};

use anyhow::{Context, Result};
use tracing::{debug, info};

use super::spin_installer::check_and_install_spin;

/// Start a spin server for development (with spin path provided)
pub fn start_spin_server_with_path<P: AsRef<Path>>(
    spin_path: &Path,
    tool_path: P,
    port: u16,
    spin_toml_path: Option<&Path>,
) -> Result<Child> {
    let spin_toml = spin_toml_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| tool_path.as_ref().join(".ftl/spin.toml"));

    info!("Starting Spin server on port {}", port);

    let child = Command::new(spin_path)
        .arg("up")
        .arg("--listen")
        .arg(format!("127.0.0.1:{port}"))
        .arg("-f")
        .arg(&spin_toml)
        .current_dir(tool_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start Spin server")?;

    Ok(child)
}

/// Deploy to FTL EdgeWorkers using Spin
pub async fn deploy_to_akamai<P: AsRef<Path>>(
    tool_path: P,
    app_name: Option<&str>,
) -> Result<DeploymentInfo> {
    let spin_path = check_and_install_spin().await?;
    check_akamai_auth(&spin_path).await?;

    let tool_path = tool_path.as_ref();
    let spin_toml = tool_path.join(".ftl/spin.toml");

    // Check if spin.toml exists
    if !spin_toml.exists() {
        anyhow::bail!(
            "spin.toml not found at {:?}. Did you build the tool first?",
            spin_toml
        );
    }

    // Get absolute path for spin.toml to avoid relative path issues
    let spin_toml_abs = spin_toml
        .canonicalize()
        .context("Failed to get absolute path for spin.toml")?;

    // Check if app is already linked by running 'spin aka app status'
    debug!(
        "Checking if app is already linked from: {:?}",
        spin_toml_abs
    );
    let status_output = Command::new(&spin_path)
        .args([
            "aka",
            "app",
            "status",
            "-f",
            &spin_toml_abs.to_string_lossy(),
        ])
        .current_dir(tool_path)
        .output()
        .context("Failed to run spin aka app status")?;

    let app_linked = status_output.status.success();

    if !app_linked {
        // Log the error for debugging but don't fail - this is expected for first
        // deployment
        let stderr = String::from_utf8_lossy(&status_output.stderr);
        debug!("App not linked (expected for first deployment): {}", stderr);
    }

    // If app is linked, deploy without --create-name
    if app_linked {
        debug!("App is linked, deploying without --create-name");
        let output = Command::new(&spin_path)
            .args([
                "aka",
                "deploy",
                "--from",
                &spin_toml_abs.to_string_lossy(),
                "--no-confirm",
            ])
            .current_dir(tool_path)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return parse_deployment_info(&stdout);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Deployment failed:\n{}", stderr);
        }
    }

    // App doesn't exist or isn't linked, need to create/deploy with --create-name
    let app_name = app_name.ok_or_else(|| {
        anyhow::anyhow!("Tool/toolkit not linked. First deployment requires a name")
    })?;

    info!("Creating new tool(kit): {}", app_name);

    let output = Command::new(&spin_path)
        .args([
            "aka",
            "deploy",
            "--from",
            &spin_toml_abs.to_string_lossy(),
            "--create-name",
            app_name,
            "--no-confirm",
        ])
        .current_dir(tool_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Deployment failed:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_deployment_info(&stdout)
}

/// Check if Akamai CLI is authenticated
pub async fn check_akamai_auth(spin_path: &Path) -> Result<bool> {
    let output = Command::new(spin_path)
        .args(["aka", "apps", "list"])
        .output()
        .context("Failed to check Akamai authentication")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not logged in") || stderr.contains("authentication") {
            anyhow::bail!("Not authenticated with Akamai. Please run: ftl login");
        }
    }

    Ok(true)
}

#[derive(Debug)]
pub struct DeploymentInfo {
    pub app_name: String,
    pub url: String,
}

fn parse_deployment_info(output: &str) -> Result<DeploymentInfo> {
    // Parse deployment output to extract URL and app name
    // Example output: "- string-formatter: https://...aka.fermyon.tech/mcp (wildcard)"
    let url = output
        .lines()
        .find(|line| line.contains("http"))
        .and_then(|line| {
            // Extract URL from the line
            line.split_whitespace().find(|s| s.starts_with("http"))
        })
        .ok_or_else(|| anyhow::anyhow!("Could not parse deployment URL"))?;

    // Extract app name from the route line "- app-name: https://..."
    let app_name = output
        .lines()
        .find(|line| line.contains("http") && line.contains("-"))
        .and_then(|line| {
            // Extract the part between "- " and ":"
            line.trim_start_matches('-')
                .split(':')
                .next()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "ftl-tool".to_string());

    Ok(DeploymentInfo {
        app_name,
        url: url.to_string(),
    })
}
