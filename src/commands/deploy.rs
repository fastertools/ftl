use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use base64::{Engine as _, engine::general_purpose};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::common::spin_installer::check_and_install_spin;
use crate::commands::login::get_stored_credentials;

#[derive(Debug, Deserialize)]
struct EcrCredentialsResponse {
    #[serde(rename = "repositoryUri")]
    registry_url: String,
    #[serde(rename = "authorizationToken")]
    token: String,
    #[serde(rename = "proxyEndpoint")]
    #[allow(dead_code)]
    proxy_endpoint: Option<String>,
    #[serde(rename = "expiresAt")]
    #[allow(dead_code)]
    expires_at: Option<String>,
    #[allow(dead_code)]
    region: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeploymentRequest {
    #[serde(rename = "appName")]
    app_name: String,
    tools: Vec<DeploymentTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
struct DeploymentTool {
    name: String,
    tag: String,
    #[serde(rename = "allowedHosts", skip_serializing_if = "Option::is_none")]
    allowed_hosts: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DeploymentResponse {
    #[allow(dead_code)]
    deployment_id: String,
    #[allow(dead_code)]
    status: String,
    app_url: Option<String>,
    #[allow(dead_code)]
    message: Option<String>,
}

struct ComponentInfo {
    name: String,
    source_path: String,
    version: String,
    allowed_hosts: Option<Vec<String>>,
}

pub async fn execute() -> Result<()> {
    println!("{} Deploying project", style("→").cyan());

    // Check if we're in a Spin project directory
    if !PathBuf::from("spin.toml").exists() {
        anyhow::bail!("No spin.toml found. Not in a project directory?");
    }

    // Get spin path
    let spin_path = check_and_install_spin().await?;

    // Build the project first
    println!("{} Building project...", style("→").dim());
    let build_output = Command::new(&spin_path)
        .args(["build"])
        .output()
        .context("Failed to build project")?;

    if !build_output.status.success() {
        anyhow::bail!(
            "Build failed:\n{}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Get authentication credentials
    let credentials = get_stored_credentials()
        .context("Not logged in to FTL. Run 'ftl login' first.")?;

    // Parse spin.toml to find user components
    let components = parse_spin_toml()?;
    if components.is_empty() {
        anyhow::bail!("No user components found in spin.toml");
    }

    // Get ECR credentials
    println!("{} Getting registry credentials...", style("→").dim());
    let ecr_creds = get_ecr_credentials(&credentials.access_token).await?;

    // Docker login to ECR
    docker_login(&ecr_creds).await?;

    // Push components to ECR
    println!("{} Pushing components to registry...", style("→").dim());
    let deployed_tools = push_components_to_ecr(&components, &ecr_creds).await?;

    // Deploy to FTL
    println!("{} Deploying to FTL...", style("→").dim());
    let app_name = get_app_name()?;
    let deployment = deploy_to_ftl(
        &credentials.access_token,
        app_name,
        deployed_tools,
    )
    .await?;

    // Display results
    println!();
    println!("{} Project deployed successfully!", style("✓").green());
    if let Some(url) = deployment.app_url {
        println!("  URL: {}", style(url).cyan());
    }

    Ok(())
}

fn parse_spin_toml() -> Result<Vec<ComponentInfo>> {
    let content = std::fs::read_to_string("spin.toml")
        .context("Failed to read spin.toml")?;
    let toml: Value = content.parse()
        .context("Failed to parse spin.toml")?;
    
    let mut components = Vec::new();
    
    // Look for components that are local files (not from registry)
    if let Some(components_table) = toml.get("component").and_then(|c| c.as_table()) {
        for (name, component) in components_table {
            if let Some(source) = component.get("source") {
                // Check if source is a local file (string) vs registry (table)
                if let Some(source_path) = source.as_str() {
                    // Skip if it's a system component (from registry)
                    if !source_path.contains("ghcr.io") && source_path.ends_with(".wasm") {
                        // Try to extract version from Cargo.toml or package.json
                        let version = extract_component_version(name, &source_path)?;
                        
                        // Extract allowed_outbound_hosts if present
                        let allowed_hosts = component
                            .get("allowed_outbound_hosts")
                            .and_then(|hosts| hosts.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            });
                        
                        components.push(ComponentInfo {
                            name: name.clone(),
                            source_path: source_path.to_string(),
                            version,
                            allowed_hosts,
                        });
                    }
                }
            }
        }
    }
    
    Ok(components)
}

fn extract_component_version(component_name: &str, source_path: &str) -> Result<String> {
    // Try to determine the component directory from the source path
    let path = PathBuf::from(source_path);
    let component_dir = if path.starts_with(component_name) {
        PathBuf::from(component_name)
    } else if let Some(parent) = path.parent() {
        parent.to_path_buf()
    } else {
        PathBuf::from(".")
    };
    
    // Try Cargo.toml first
    let cargo_path = component_dir.join("Cargo.toml");
    if cargo_path.exists() {
        let cargo_content = std::fs::read_to_string(&cargo_path)?;
        let cargo_toml: Value = cargo_content.parse()?;
        if let Some(version) = cargo_toml
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
        {
            return Ok(version.to_string());
        }
    }
    
    // Try package.json
    let package_path = component_dir.join("package.json");
    if package_path.exists() {
        let package_content = std::fs::read_to_string(&package_path)?;
        let package_json: serde_json::Value = serde_json::from_str(&package_content)?;
        if let Some(version) = package_json.get("version").and_then(|v| v.as_str()) {
            return Ok(version.to_string());
        }
    }
    
    // Default to 0.1.0 if no version found
    Ok("0.1.0".to_string())
}

fn get_app_name() -> Result<String> {
    let content = std::fs::read_to_string("spin.toml")?;
    let toml: Value = content.parse()?;
    
    toml.get("application")
        .and_then(|app| app.get("name"))
        .and_then(|name| name.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("No application name found in spin.toml"))
}

async fn get_ecr_credentials(access_token: &str) -> Result<EcrCredentialsResponse> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );
    
    // TODO: Make API URL configurable
    let api_url = std::env::var("FTL_API_URL")
        .unwrap_or_else(|_| "https://api.ftl.dev".to_string());
    
    let response = client
        .post(&format!("{}/v1/registry/credentials", api_url))
        .headers(headers)
        .send()
        .await
        .context("Failed to get ECR credentials")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow!(
            "Failed to get ECR credentials (HTTP {}): {}",
            status.as_u16(),
            error_text
        ));
    }
    
    response
        .json::<EcrCredentialsResponse>()
        .await
        .context("Failed to parse ECR credentials response")
}

async fn docker_login(ecr_creds: &EcrCredentialsResponse) -> Result<()> {
    // ECR authorization tokens are base64 encoded "AWS:password"
    // We need to extract just the password part
    let decoded = general_purpose::STANDARD.decode(&ecr_creds.token)
        .context("Failed to decode ECR authorization token")?;
    let auth_string = String::from_utf8(decoded)
        .context("Invalid UTF-8 in authorization token")?;
    
    // Extract password after "AWS:"
    let password = auth_string
        .strip_prefix("AWS:")
        .ok_or_else(|| anyhow!("Invalid ECR token format"))?;
    
    // Extract registry endpoint from repository URI
    // Format: 123456789012.dkr.ecr.us-east-1.amazonaws.com/users/userId
    let registry_endpoint = ecr_creds.registry_url
        .split('/')
        .next()
        .ok_or_else(|| anyhow!("Invalid repository URI format"))?;
    
    let mut cmd = Command::new("docker");
    cmd.args(&["login", "--username", "AWS", "--password-stdin", registry_endpoint]);
    cmd.stdin(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()
        .context("Failed to start docker login")?;
    
    // Write password to stdin
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(password.as_bytes())?;
    }
    
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!("Docker login failed"));
    }
    
    Ok(())
}

async fn push_components_to_ecr(
    components: &[ComponentInfo],
    ecr_creds: &EcrCredentialsResponse,
) -> Result<Vec<DeploymentTool>> {
    let mut deployed_tools = Vec::new();
    
    // Extract user ID from registry URL (format: xxx.dkr.ecr.region.amazonaws.com/users/userId)
    let registry_base = &ecr_creds.registry_url;
    
    for component in components {
        println!(
            "  {} Pushing {} (v{})...",
            style("→").dim(),
            component.name,
            component.version
        );
        
        // Check if wkg is available
        which::which("wkg")
            .context("wkg not found. Install from: https://github.com/bytecodealliance/wasm-pkg-tools")?;
        
        // Push with version tag
        let versioned_tag = format!("{}:{}", registry_base, component.version);
        let mut push_cmd = Command::new("wkg");
        push_cmd.args(&["oci", "push", &versioned_tag, &component.source_path]);
        
        let output = push_cmd.output()
            .context("Failed to push component with wkg")?;
        
        if !output.status.success() {
            return Err(anyhow!(
                "Failed to push {}: {}",
                component.name,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Also push as latest
        let latest_tag = format!("{}:latest", registry_base);
        let mut push_latest = Command::new("wkg");
        push_latest.args(&["oci", "push", &latest_tag, &component.source_path]);
        push_latest.output()?;
        
        deployed_tools.push(DeploymentTool {
            name: component.name.clone(),
            tag: component.version.clone(),
            allowed_hosts: component.allowed_hosts.clone(),
        });
    }
    
    Ok(deployed_tools)
}

async fn deploy_to_ftl(
    access_token: &str,
    app_name: String,
    tools: Vec<DeploymentTool>,
) -> Result<DeploymentResponse> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );
    
    let api_url = std::env::var("FTL_API_URL")
        .unwrap_or_else(|_| "https://api.ftl.dev".to_string());
    
    let request_body = DeploymentRequest {
        app_name,
        tools,
        variables: None,
    };
    
    // Show progress while waiting for deployment
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message("Deploying application...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    let response = client
        .post(&format!("{}/v1/deployments", api_url))
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .context("Failed to deploy to FTL")?;
    
    pb.finish_and_clear();
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Deployment failed: {}", error_text));
    }
    
    response
        .json::<DeploymentResponse>()
        .await
        .context("Failed to parse deployment response")
}
