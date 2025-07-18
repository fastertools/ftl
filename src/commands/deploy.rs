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
use crate::commands::login::get_or_refresh_credentials;

const FTL_API_URL: &str = "https://fqwe5s59ob.execute-api.us-east-1.amazonaws.com";

#[derive(Debug, Serialize)]
struct CreateRepositoryRequest {
    #[serde(rename = "toolName")]
    tool_name: String,
}

#[derive(Debug, Deserialize)]
struct CreateRepositoryResponse {
    #[serde(rename = "repositoryUri")]
    repository_uri: String,
    #[serde(rename = "repositoryName")]
    #[allow(dead_code)]
    repository_name: String,
    #[serde(rename = "alreadyExists")]
    #[allow(dead_code)]
    already_exists: bool,
}

#[derive(Debug, Deserialize)]
struct EcrCredentialsResponse {
    #[serde(rename = "registryUri")]
    registry_uri: String,
    #[serde(rename = "authorizationToken")]
    authorization_token: String,
    #[serde(rename = "expiresAt")]
    #[allow(dead_code)]
    expires_at: String,
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
struct StartDeploymentResponse {
    #[serde(rename = "deploymentId")]
    deployment_id: String,
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    message: Option<String>,
    #[serde(rename = "buildId")]
    #[allow(dead_code)]
    build_id: Option<String>,
    #[serde(rename = "statusUrl")]
    status_url: String,
}

#[derive(Debug, Deserialize)]
struct DeploymentStatusResponse {
    deployment: DeploymentDetails,
    #[serde(rename = "buildStatus")]
    #[allow(dead_code)]
    build_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeploymentDetails {
    #[serde(rename = "deploymentId")]
    #[allow(dead_code)]
    deployment_id: String,
    #[serde(rename = "appName")]
    #[allow(dead_code)]
    app_name: String,
    status: String,
    #[serde(rename = "statusMessage")]
    status_message: Option<String>,
    #[serde(rename = "deploymentUrl")]
    deployment_url: Option<String>,
    #[serde(rename = "errorReason")]
    error_reason: Option<String>,
    #[serde(rename = "createdAt")]
    #[allow(dead_code)]
    created_at: String,
    #[serde(rename = "updatedAt")]
    #[allow(dead_code)]
    updated_at: String,
    #[serde(rename = "completedAt")]
    #[allow(dead_code)]
    completed_at: Option<String>,
    #[allow(dead_code)]
    duration: Option<u64>,
}

struct ComponentInfo {
    name: String,
    source_path: String,
    version: String,
    allowed_hosts: Option<Vec<String>>,
}

pub async fn execute() -> Result<()> {
    println!("{} {} Deploying project", style("▶").cyan(), style("FTL").bold());
    println!();

    // Check if we're in a Spin project directory
    if !PathBuf::from("spin.toml").exists() {
        anyhow::bail!("No spin.toml found. Not in a project directory?");
    }

    // Create a progress bar for the entire deployment process
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}% {msg}")
            .unwrap()
            .progress_chars("█▓▒░")
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Get spin path
    pb.set_position(5);
    pb.set_message("Checking environment...");
    let spin_path = check_and_install_spin().await?;

    // Build the project first
    pb.set_position(10);
    pb.set_message("Building project...");
    let build_output = Command::new(&spin_path)
        .args(["build"])
        .output()
        .context("Failed to build project")?;

    if !build_output.status.success() {
        pb.finish_and_clear();
        anyhow::bail!(
            "Build failed:\n{}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Get authentication credentials
    pb.set_position(20);
    pb.set_message("Authenticating...");
    let credentials = match get_or_refresh_credentials().await {
        Ok(creds) => creds,
        Err(e) => {
            pb.finish_and_clear();
            if e.to_string().contains("expired") {
                anyhow::bail!("Authentication token has expired. Please run 'ftl login' to re-authenticate.");
            } else {
                anyhow::bail!("Not logged in to FTL. Run 'ftl login' first.");
            }
        }
    };

    // Parse spin.toml to find user components
    pb.set_position(25);
    pb.set_message("Parsing project...");
    let components = parse_spin_toml()?;
    if components.is_empty() {
        pb.finish_and_clear();
        anyhow::bail!("No user components found in spin.toml");
    }

    // Get ECR credentials
    pb.set_position(30);
    pb.set_message("Getting registry credentials...");
    let ecr_creds = get_ecr_credentials(&credentials.access_token).await?;

    // Docker login to ECR
    pb.set_position(35);
    pb.set_message("Logging into registry...");
    docker_login(&ecr_creds).await?;

    // Create repositories and push components to ECR
    pb.set_position(40);
    pb.set_message("Pushing components...");
    let deployed_tools = create_repositories_and_push_with_progress(&components, &ecr_creds, &credentials.access_token, &pb).await?;

    // Deploy to FTL
    pb.set_position(70);
    pb.set_message("Starting deployment...");
    let app_name = get_app_name()?;
    let deployment = deploy_to_ftl_with_progress(
        &credentials.access_token,
        app_name,
        deployed_tools,
        pb,
    )
    .await?;

    // Display results
    println!();
    println!("{} Deployment successful!", style("✓").green().bold());
    if let Some(deployment_url) = deployment.deployment_url {
        println!();
        println!("  {} {}", style("MCP URL:").bold(), style(deployment_url).cyan().underlined());
        println!();
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
    
    
    let response = client
        .post(&format!("{}/v1/registry/credentials", FTL_API_URL))
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
    let decoded = general_purpose::STANDARD.decode(&ecr_creds.authorization_token)
        .context("Failed to decode ECR authorization token")?;
    let auth_string = String::from_utf8(decoded)
        .context("Invalid UTF-8 in authorization token")?;
    
    // Extract password after "AWS:"
    let password = auth_string
        .strip_prefix("AWS:")
        .ok_or_else(|| anyhow!("Invalid ECR token format"))?;
    
    // Use the registry URI directly
    let registry_endpoint = &ecr_creds.registry_uri;
    
    let mut cmd = Command::new("docker");
    cmd.args(&["login", "--username", "AWS", "--password-stdin", registry_endpoint]);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());
    
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

async fn create_repository(access_token: &str, tool_name: &str) -> Result<CreateRepositoryResponse> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );
    
    
    let request_body = CreateRepositoryRequest {
        tool_name: tool_name.to_string(),
    };
    
    let response = client
        .post(&format!("{}/v1/registry/repositories", FTL_API_URL))
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .context("Failed to create ECR repository")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow!(
            "Failed to create ECR repository (HTTP {}): {}",
            status.as_u16(),
            error_text
        ));
    }
    
    response
        .json::<CreateRepositoryResponse>()
        .await
        .context("Failed to parse create repository response")
}

async fn create_repositories_and_push_with_progress(
    components: &[ComponentInfo],
    _ecr_creds: &EcrCredentialsResponse,
    access_token: &str,
    pb: &ProgressBar,
) -> Result<Vec<DeploymentTool>> {
    let mut deployed_tools = Vec::new();
    let total_components = components.len();
    
    for (idx, component) in components.iter().enumerate() {
        // Update progress
        let progress = 40 + (30 * idx / total_components) as u64;
        pb.set_position(progress);
        pb.set_message(format!("Pushing {} v{}...", component.name, component.version));
        
        // Create repository for this component
        let repo_response = create_repository(access_token, &component.name).await?;
        
        // Check if wkg is available
        which::which("wkg")
            .context("wkg not found. Install from: https://github.com/bytecodealliance/wasm-pkg-tools")?;
        
        // Push with version tag
        let versioned_tag = format!("{}:{}", repo_response.repository_uri, component.version);
        let mut push_cmd = Command::new("wkg");
        push_cmd.args(&["oci", "push", &versioned_tag, &component.source_path]);
        push_cmd.stdout(std::process::Stdio::null());
        push_cmd.stderr(std::process::Stdio::piped());
        
        let output = push_cmd.output()
            .context("Failed to push component with wkg")?;
        
        if !output.status.success() {
            pb.finish_and_clear();
            return Err(anyhow!(
                "Failed to push {}: {}",
                component.name,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Also push as latest
        let latest_tag = format!("{}:latest", repo_response.repository_uri);
        let mut push_latest = Command::new("wkg");
        push_latest.args(&["oci", "push", &latest_tag, &component.source_path]);
        push_latest.stdout(std::process::Stdio::null());
        push_latest.stderr(std::process::Stdio::null());
        push_latest.output()?;
        
        deployed_tools.push(DeploymentTool {
            name: component.name.clone(),
            tag: component.version.clone(),
            allowed_hosts: component.allowed_hosts.clone(),
        });
    }
    
    Ok(deployed_tools)
}

async fn poll_deployment_status_with_progress(
    client: &reqwest::Client,
    headers: HeaderMap,
    status_url: &str,
    _deployment_id: &str,
    pb: ProgressBar,
) -> Result<DeploymentDetails> {
    let max_attempts = 60; // 5 minutes with 5-second intervals
    let mut attempts = 0;
    
    loop {
        if attempts >= max_attempts {
            pb.finish_and_clear();
            return Err(anyhow!("Deployment timeout after 5 minutes"));
        }
        
        let response = client
            .get(status_url)
            .headers(headers.clone())
            .send()
            .await
            .context("Failed to check deployment status")?;
        
        if !response.status().is_success() {
            pb.finish_and_clear();
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get deployment status: {}", error_text));
        }
        
        let status_response: DeploymentStatusResponse = response
            .json()
            .await
            .context("Failed to parse deployment status")?;
        
        let deployment = status_response.deployment;
        
        // Update progress based on status
        let (progress, status_msg) = match deployment.status.as_str() {
            "INITIALIZING" => (75, "Initializing deployment...".to_string()),
            "BUILDING" => (80, "Building application...".to_string()),
            "PROVISIONING" => (85, "Provisioning environment...".to_string()),
            "AUTHENTICATING" => (90, "Authenticating with registries...".to_string()),
            "DEPLOYING" => (95, "Finalizing deployment...".to_string()),
            _status => {
                let msg = deployment.status_message.as_deref()
                    .unwrap_or("Processing...")
                    .to_string();
                (pb.position().min(95), msg)
            }
        };
        
        pb.set_position(progress);
        pb.set_message(status_msg);
        
        match deployment.status.as_str() {
            "COMPLETED" => {
                pb.set_position(100);
                pb.set_message("Deployment complete!");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                pb.finish_and_clear();
                return Ok(deployment);
            }
            "FAILED" => {
                pb.finish_and_clear();
                let error_msg = deployment.error_reason.as_deref()
                    .or(deployment.status_message.as_deref())
                    .unwrap_or("Deployment failed")
                    .to_string();
                return Err(anyhow!("Deployment failed: {}", error_msg));
            }
            _ => {
                // Continue polling
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                attempts += 1;
            }
        }
    }
}

async fn deploy_to_ftl_with_progress(
    access_token: &str,
    app_name: String,
    tools: Vec<DeploymentTool>,
    pb: ProgressBar,
) -> Result<DeploymentDetails> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );
    
    
    let request_body = DeploymentRequest {
        app_name,
        tools,
        variables: None,
    };
    
    let response = client
        .post(&format!("{}/v1/deployments", FTL_API_URL))
        .headers(headers.clone())
        .json(&request_body)
        .send()
        .await
        .context("Failed to start deployment")?;
    
    if response.status() != 202 {
        pb.finish_and_clear();
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to start deployment: {}", error_text));
    }
    
    let start_response: StartDeploymentResponse = response
        .json()
        .await
        .context("Failed to parse deployment start response")?;
    
    // Poll for deployment status
    let status_url = if start_response.status_url.starts_with("/") {
        format!("{}{}", FTL_API_URL, start_response.status_url)
    } else {
        start_response.status_url
    };
    
    poll_deployment_status_with_progress(
        &client,
        headers,
        &status_url,
        &start_response.deployment_id,
        pb,
    ).await
}
