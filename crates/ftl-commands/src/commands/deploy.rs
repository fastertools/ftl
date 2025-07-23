//! Refactored deploy command with dependency injection for testability

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use ftl_core::api_client::{error::ConversionError, types};
use ftl_core::deps::{
    AsyncRuntime, Clock, CommandExecutor, CredentialsProvider, FileSystem,
    FtlApiClient, MessageStyle, UserInterface,
};

/// Build executor trait for running builds
#[async_trait::async_trait]
pub trait BuildExecutor: Send + Sync {
    /// Execute a build
    async fn execute(&self, path: Option<&Path>, release: bool) -> Result<()>;
}

/// Component information extracted from spin.toml
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentInfo {
    /// Component name
    pub name: String,
    /// Path to the component's WASM file
    pub source_path: String,
    /// Component version
    pub version: String,
    /// Allowed outbound hosts for the component
    pub allowed_hosts: Option<Vec<String>>,
}

/// Deploy configuration
pub struct DeployConfig {
    /// Application name from spin.toml
    pub app_name: String,
    /// Components to deploy
    pub components: Vec<ComponentInfo>,
}

/// Dependencies for the deploy command
pub struct DeployDependencies {
    /// File system operations
    pub file_system: Arc<dyn FileSystem>,
    /// Command execution operations
    pub command_executor: Arc<dyn CommandExecutor>,
    /// API client for FTL service
    pub api_client: Arc<dyn FtlApiClient>,
    /// Clock for time operations
    pub clock: Arc<dyn Clock>,
    /// Provider for authentication credentials
    pub credentials_provider: Arc<dyn CredentialsProvider>,
    /// User interface for output
    pub ui: Arc<dyn UserInterface>,
    /// Build executor for running builds
    pub build_executor: Arc<dyn BuildExecutor>,
    /// Async runtime for scheduling tasks
    pub async_runtime: Arc<dyn AsyncRuntime>,
}

/// Execute the deploy command with injected dependencies
#[allow(clippy::too_many_lines)]
pub async fn execute_with_deps(deps: Arc<DeployDependencies>) -> Result<()> {
    deps.ui
        .print(&format!("{} {} Deploying project", "▶", "FTL"));
    deps.ui.print("");

    // Check if we're in a Spin project directory
    let spin_toml_path = PathBuf::from("spin.toml");
    if !deps.file_system.exists(&spin_toml_path) {
        return Err(anyhow!("No spin.toml found. Not in a project directory?"));
    }

    // Create a spinner for status updates
    let spinner = deps.ui.create_spinner();
    spinner.enable_steady_tick(deps.clock.duration_from_millis(100));

    // Build the project first
    spinner.finish_and_clear();
    deps.ui
        .print_styled("→ Building project...", MessageStyle::Cyan);
    deps.ui.print("");

    deps.build_executor.execute(None, true).await?;

    deps.ui.print("");

    // Re-create spinner after build
    let spinner = deps.ui.create_spinner();
    spinner.enable_steady_tick(deps.clock.duration_from_millis(100));

    // Get authentication credentials
    spinner.set_message("Authenticating...");
    let _credentials = match deps.credentials_provider.get_or_refresh_credentials().await {
        Ok(creds) => creds,
        Err(e) => {
            spinner.finish_and_clear();
            if e.to_string().contains("expired") {
                return Err(anyhow!(
                    "Authentication token has expired. Please run 'ftl login' to re-authenticate."
                ));
            }
            return Err(anyhow!("Not logged in to FTL. Run 'ftl login' first."));
        }
    };

    // Parse spin.toml to find user components
    spinner.set_message("Parsing project...");
    let config = parse_deploy_config(&deps.file_system)?;
    if config.components.is_empty() {
        spinner.finish_and_clear();
        return Err(anyhow!("No user components found in spin.toml"));
    }

    // Get ECR credentials
    spinner.set_message("Getting registry credentials...");
    let ecr_creds = deps
        .api_client
        .get_ecr_credentials()
        .await
        .map_err(|e| anyhow!("Failed to get ECR credentials: {}", e))?;

    // Docker login to ECR
    spinner.set_message("Logging into registry...");
    docker_login(&deps.command_executor, &ecr_creds).await?;

    // Create repositories and push components to ECR
    spinner.finish_and_clear();
    let deployed_tools =
        create_repositories_and_push_with_progress(&config.components, deps.clone()).await?;

    // Deploy to FTL
    deps.ui.print("");
    let spinner = deps.ui.create_spinner();
    spinner.enable_steady_tick(deps.clock.duration_from_millis(100));
    spinner.set_message("Starting deployment...");

    // Refresh credentials before deployment in case the token expired
    let _fresh_credentials = match deps.credentials_provider.get_or_refresh_credentials().await {
        Ok(creds) => creds,
        Err(e) => {
            spinner.finish_and_clear();
            return Err(anyhow!("Failed to refresh authentication token: {}", e));
        }
    };

    let deployment =
        deploy_to_ftl_with_progress(deps.clone(), config.app_name, deployed_tools, spinner).await?;

    // Display results
    deps.ui.print("");
    deps.ui
        .print_styled("✓ Deployment successful!", MessageStyle::Success);
    if let Some(deployment_url) = deployment.deployment_url {
        deps.ui.print("");
        deps.ui.print(&format!("  MCP URL: {deployment_url}"));
        deps.ui.print("");
    }

    Ok(())
}

/// Parse deployment configuration from spin.toml
pub fn parse_deploy_config(file_system: &Arc<dyn FileSystem>) -> Result<DeployConfig> {
    let content = file_system.read_to_string(Path::new("spin.toml"))?;
    let toml: toml::Value = toml::from_str(&content).context("Failed to parse spin.toml")?;

    let app_name = toml
        .get("application")
        .and_then(|app| app.get("name"))
        .and_then(|name| name.as_str())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| anyhow!("No application name found in spin.toml"))?;

    let mut components = Vec::new();

    // Look for components that are local files (not from registry)
    if let Some(components_table) = toml.get("component").and_then(|c| c.as_table()) {
        for (name, component) in components_table {
            if let Some(source) = component.get("source") {
                // Check if source is a local file (string) vs registry (table)
                if let Some(source_path) = source.as_str() {
                    // Skip if it's a system component (from registry)
                    if !source_path.contains("ghcr.io")
                        && source_path.to_lowercase().ends_with(".wasm")
                    {
                        // Try to extract version
                        let version = extract_component_version(file_system, name, source_path)?;

                        // Extract allowed_outbound_hosts if present
                        let allowed_hosts = component
                            .get("allowed_outbound_hosts")
                            .and_then(|hosts| hosts.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(std::string::ToString::to_string)
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

    Ok(DeployConfig {
        app_name,
        components,
    })
}

/// Extract component version from Cargo.toml or package.json
pub fn extract_component_version(
    file_system: &Arc<dyn FileSystem>,
    component_name: &str,
    source_path: &str,
) -> Result<String> {
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
    if file_system.exists(&cargo_path) {
        let cargo_content = file_system.read_to_string(&cargo_path)?;
        let cargo_toml: toml::Value = toml::from_str(&cargo_content)?;
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
    if file_system.exists(&package_path) {
        let package_content = file_system.read_to_string(&package_path)?;
        let package_json: serde_json::Value = serde_json::from_str(&package_content)?;
        if let Some(version) = package_json.get("version").and_then(|v| v.as_str()) {
            return Ok(version.to_string());
        }
    }

    // Default to 0.1.0 if no version found
    Ok("0.1.0".to_string())
}

async fn docker_login(
    command_executor: &Arc<dyn CommandExecutor>,
    ecr_creds: &types::GetEcrCredentialsResponse,
) -> Result<()> {
    // ECR authorization tokens are base64 encoded "AWS:password"
    let decoded = general_purpose::STANDARD
        .decode(&ecr_creds.authorization_token)
        .context("Failed to decode ECR authorization token")?;
    let auth_string = String::from_utf8(decoded).context("Invalid UTF-8 in authorization token")?;

    // Extract password after "AWS:"
    let password = auth_string
        .strip_prefix("AWS:")
        .ok_or_else(|| anyhow!("Invalid ECR token format"))?;

    let args = vec![
        "login",
        "--username",
        "AWS",
        "--password-stdin",
        &ecr_creds.registry_uri,
    ];

    let output = command_executor
        .execute_with_stdin("docker", &args, password)
        .await
        .context("Failed to execute docker login")?;

    if !output.success {
        return Err(anyhow!("Docker login failed"));
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn create_repositories_and_push_with_progress(
    components: &[ComponentInfo],
    deps: Arc<DeployDependencies>,
) -> Result<Vec<types::DeploymentRequestToolsItem>> {
    // Check if wkg is available before starting
    deps.command_executor
        .check_command_exists("wkg")
        .await
        .context(
            "wkg not found. Install from: https://github.com/bytecodealliance/wasm-pkg-tools",
        )?;

    deps.ui.print(&format!(
        "→ Pushing {} components in parallel",
        components.len()
    ));
    deps.ui.print("");

    let multi_progress = deps.ui.create_multi_progress();
    let mut tasks = JoinSet::new();
    let deployed_tools = Arc::new(Mutex::new(Vec::new()));

    // Track errors across all tasks
    let error_flag = Arc::new(Mutex::new(None::<String>));

    // Limit concurrent operations
    let semaphore = Arc::new(Semaphore::new(4));

    for component in components {
        let pb = multi_progress.add_spinner();
        pb.set_prefix(format!("[{}]", component.name));
        pb.set_message("Creating repository...");
        pb.enable_steady_tick(deps.clock.duration_from_millis(100));

        let component = component.clone();
        let deps = deps.clone();
        let deployed_tools = Arc::clone(&deployed_tools);
        let error_flag = Arc::clone(&error_flag);
        let semaphore = Arc::clone(&semaphore);

        tasks.spawn(async move {
            // Acquire permit to limit concurrency
            let _permit = semaphore.acquire().await.unwrap();

            // Check if another task has already failed
            if error_flag.lock().await.is_some() {
                pb.finish_with_message("Skipped due to error".to_string());
                return Ok(());
            }

            let start = deps.clock.now();

            // Create repository
            pb.set_message("Creating repository...");
            let repo_response = match deps
                .api_client
                .create_ecr_repository(&types::CreateEcrRepositoryRequest {
                    tool_name: component
                        .name
                        .as_str()
                        .try_into()
                        .map_err(|e: ConversionError| anyhow!("Invalid tool name: {}", e))?,
                })
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    pb.finish_with_message(format!("✗ Failed to create repository: {e}"));
                    let mut error_guard = error_flag.lock().await;
                    if error_guard.is_none() {
                        *error_guard =
                            Some(format!("Component '{}' failed: {}", component.name, e));
                    }
                    return Err(anyhow!("Failed to create repository: {}", e));
                }
            };

            // Push component with version tag
            pb.set_message(&format!("Pushing v{}...", component.version));
            let versioned_tag = format!("{}:{}", repo_response.repository_uri, component.version);
            let output = deps
                .command_executor
                .execute(
                    "wkg",
                    &["oci", "push", &versioned_tag, &component.source_path],
                )
                .await
                .context("Failed to push component with wkg")?;

            if !output.success {
                let error = format!(
                    "Failed to push {}: {}",
                    component.name,
                    String::from_utf8_lossy(&output.stderr)
                );
                pb.finish_with_message(format!("✗ {error}"));
                let mut error_guard = error_flag.lock().await;
                if error_guard.is_none() {
                    *error_guard = Some(error.clone());
                }
                return Err(anyhow!(error));
            }

            // Add to deployed tools
            let mut tools = deployed_tools.lock().await;
            tools.push(types::DeploymentRequestToolsItem {
                name: component
                    .name
                    .as_str()
                    .try_into()
                    .map_err(|e: ConversionError| anyhow!("Invalid tool name: {}", e))?,
                tag: component
                    .version
                    .as_str()
                    .try_into()
                    .map_err(|e: ConversionError| anyhow!("Invalid tag: {}", e))?,
                allowed_hosts: component.allowed_hosts.clone().unwrap_or_default(),
                component_uri: None,
            });

            let duration = start.elapsed();
            pb.finish_with_message(format!(
                "✓ Pushed successfully in {:.1}s",
                duration.as_secs_f64()
            ));

            Ok(())
        });
    }

    // Wait for all tasks to complete
    let mut first_error = None;
    while let Some(result) = tasks.join_next().await {
        if let Err(e) = result? {
            if first_error.is_none() {
                first_error = Some(e);
            }
        }
    }

    // If any component failed, return the first error
    if let Some(e) = first_error {
        return Err(e);
    }

    let tools = Arc::try_unwrap(deployed_tools).unwrap().into_inner();

    deps.ui.print("");
    deps.ui.print_styled(
        "✓ All components pushed successfully!",
        MessageStyle::Success,
    );

    Ok(tools)
}

#[allow(clippy::too_many_lines)]
async fn poll_deployment_status_with_progress(
    deps: Arc<DeployDependencies>,
    deployment_id: &str,
    spinner: Box<dyn ftl_core::deps::ProgressIndicator>,
) -> Result<types::DeploymentStatusDeployment> {
    let max_attempts = 60; // 5 minutes with 5-second intervals
    let mut attempts = 0;

    loop {
        if attempts >= max_attempts {
            spinner.finish_and_clear();
            return Err(anyhow!("Deployment timeout after 5 minutes"));
        }

        let status_response = match deps.api_client.get_deployment_status(deployment_id).await {
            Ok(resp) => resp,
            Err(e) => {
                spinner.finish_and_clear();
                return Err(anyhow!("Failed to get deployment status: {}", e));
            }
        };

        let deployment = status_response.deployment;

        // Update spinner message based on status and stages
        let stages = &deployment.stages;
        let status_msg = if stages.is_empty() {
            format!("Status: {}", deployment.status)
        } else {
            // Find the current stage (first non-completed stage)
            let current_stage = stages
                .iter()
                .find(|s| {
                    !matches!(
                        s.status,
                        types::DeploymentStatusDeploymentStagesItemStatus::Completed
                    )
                })
                .or(stages.last());

            if let Some(stage) = current_stage {
                match &stage.stage {
                    types::DeploymentStatusDeploymentStagesItemStage::ImageBuild => {
                        "Building container image...".to_string()
                    }
                    types::DeploymentStatusDeploymentStagesItemStage::PlatformUpload => {
                        "Uploading to platform...".to_string()
                    }
                    types::DeploymentStatusDeploymentStagesItemStage::PlatformDeploy => {
                        "Deploying application...".to_string()
                    }
                    types::DeploymentStatusDeploymentStagesItemStage::Validation => {
                        "Validating deployment...".to_string()
                    }
                }
            } else {
                "Processing deployment...".to_string()
            }
        };

        spinner.set_message(&status_msg);

        match deployment.status {
            types::DeploymentStatusDeploymentStatus::Deployed => {
                spinner.finish_and_clear();
                return Ok(deployment);
            }
            types::DeploymentStatusDeploymentStatus::Failed
            | types::DeploymentStatusDeploymentStatus::Cancelled => {
                spinner.finish_and_clear();
                let error_msg = deployment
                    .error
                    .as_deref()
                    .unwrap_or("Deployment failed")
                    .to_string();
                return Err(anyhow!("Deployment failed: {}", error_msg));
            }
            _ => {
                // Continue polling for other statuses
                deps.async_runtime
                    .sleep(deps.clock.duration_from_secs(5))
                    .await;
                attempts += 1;
            }
        }
    }
}

async fn deploy_to_ftl_with_progress(
    deps: Arc<DeployDependencies>,
    app_name: String,
    tools: Vec<types::DeploymentRequestToolsItem>,
    spinner: Box<dyn ftl_core::deps::ProgressIndicator>,
) -> Result<types::DeploymentStatusDeployment> {
    let request_body = types::DeploymentRequest {
        app_name: app_name
            .as_str()
            .try_into()
            .map_err(|e: ConversionError| anyhow!("Invalid app name: {}", e))?,
        tools,
        variables: HashMap::default(),
    };

    let deployment_response = deps
        .api_client
        .deploy_app(&request_body)
        .await
        .map_err(|e| {
            spinner.finish_and_clear();
            anyhow!("Failed to start deployment: {}", e)
        })?;

    // Update spinner with deployment ID
    spinner.set_message(&format!(
        "Deployment {} in progress...",
        &deployment_response.deployment_id
    ));

    // Poll for deployment status
    poll_deployment_status_with_progress(
        deps,
        &deployment_response.deployment_id.to_string(),
        spinner,
    )
    .await
}

/// Deploy command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct DeployArgs {
    // Deploy takes no arguments - it uses the current directory
}

// Build executor implementation
struct BuildExecutorImpl;

#[async_trait::async_trait]
impl BuildExecutor for BuildExecutorImpl {
    async fn execute(&self, path: Option<&Path>, release: bool) -> Result<()> {
        use crate::commands::build;
        
        let args = build::BuildArgs {
            path: path.map(|p| p.to_path_buf()),
            release,
        };
        
        build::execute(args).await
    }
}

/// Execute the deploy command with default dependencies
pub async fn execute(_args: DeployArgs) -> Result<()> {
    use ftl_core::deps::{RealCommandExecutor, RealFileSystem, RealFtlApiClient, RealClock, 
                         RealCredentialsProvider, RealAsyncRuntime};
    use ftl_common::RealUserInterface;
    
    // Get credentials first to create authenticated API client
    let credentials_provider = Arc::new(RealCredentialsProvider);
    let credentials = credentials_provider.get_or_refresh_credentials().await?;
    
    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(DeployDependencies {
        file_system: Arc::new(RealFileSystem),
        command_executor: Arc::new(RealCommandExecutor),
        api_client: Arc::new(RealFtlApiClient::new_with_auth(
            ftl_core::api_client::Client::new(&ftl_core::config::DEFAULT_API_BASE_URL),
            credentials.access_token.clone(),
        )),
        clock: Arc::new(RealClock),
        credentials_provider,
        ui: ui.clone(),
        build_executor: Arc::new(BuildExecutorImpl),
        async_runtime: Arc::new(RealAsyncRuntime),
    });

    execute_with_deps(deps).await
}

#[cfg(test)]
#[path = "deploy_tests.rs"]
mod tests;
