//! Refactored deploy command with dependency injection for testability

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use ftl_runtime::api_client::types;
use ftl_runtime::deps::{
    AsyncRuntime, Clock, CommandExecutor, CredentialsProvider, FileSystem, FtlApiClient,
    MessageStyle, UserInterface,
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
    pub allowed_outbound_hosts: Option<Vec<String>>,
}

/// Deploy configuration
pub struct DeployConfig {
    /// Box name from spin.toml
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
pub async fn execute_with_deps(
    deps: Arc<DeployDependencies>,
    variables: Vec<String>,
) -> Result<()> {
    deps.ui.print(&format!("{} {} Deploying box", "▶", "FTL"));
    deps.ui.print("");

    // Generate temporary spin.toml from ftl.toml
    let temp_spin_toml =
        crate::config::transpiler::generate_temp_spin_toml(&deps.file_system, &PathBuf::from("."))?;

    // We must have a temp spin.toml since ftl.toml is required
    let manifest_path = temp_spin_toml
        .ok_or_else(|| anyhow!("No ftl.toml found. Not in an FTL project directory?"))?;

    // Run the deployment with the manifest path
    // Clean up is handled by tempfile crate when temp_spin_toml goes out of scope
    execute_deploy_inner(deps.clone(), variables, manifest_path, true).await
}

/// Inner deployment logic separated for proper cleanup handling
#[allow(clippy::too_many_lines)]
async fn execute_deploy_inner(
    deps: Arc<DeployDependencies>,
    variables: Vec<String>,
    manifest_path: PathBuf,
    _is_temp_manifest: bool,
) -> Result<()> {
    // Create a spinner for status updates
    let spinner = deps.ui.create_spinner();
    spinner.enable_steady_tick(deps.clock.duration_from_millis(100));

    // Parse ftl.toml to determine build profiles and deploy names
    let ftl_content = deps.file_system.read_to_string(Path::new("ftl.toml"))?;
    let ftl_config = crate::config::ftl_config::FtlConfig::parse(&ftl_content)?;

    // Determine which build profile to use for deployment
    // For now, use release mode if any tool has deploy.profile = "release" or no profile specified
    let use_release = ftl_config.tools.values().any(|tool| {
        tool.deploy.as_ref().is_none_or(|d| d.profile == "release") // Default to release if no deploy config
    });

    // Build the project first
    spinner.finish_and_clear();
    deps.ui
        .print_styled("→ Building project...", MessageStyle::Cyan);
    deps.ui.print("");

    deps.build_executor.execute(None, use_release).await?;

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
    let config = parse_deploy_config(&deps.file_system, &manifest_path)?;
    if config.components.is_empty() {
        spinner.finish_and_clear();
        return Err(anyhow!("No user components found in spin.toml"));
    }

    // Get ECR credentials
    spinner.set_message("Getting registry credentials...");
    let ecr_creds = deps
        .api_client
        .create_ecr_token()
        .await
        .map_err(|e| anyhow!("Failed to get ECR token: {}", e))?;

    // Docker login to ECR
    spinner.set_message("Logging into registry...");
    docker_login(&deps.command_executor, &ecr_creds).await?;

    // Get or create the app first
    spinner.set_message("Checking if box exists...");
    let existing_apps = deps
        .api_client
        .list_apps(None, None, Some(&config.app_name))
        .await
        .map_err(|e| anyhow!("Failed to check existing apps: {}", e))?;

    let app_id = if existing_apps.apps.is_empty() {
        // Box doesn't exist, create it first
        spinner.set_message("Creating box...");
        let create_app_request = types::CreateAppRequest {
            app_name: config.app_name
                .as_str()
                .try_into()
                .map_err(|e| anyhow!("Invalid app name: {}", e))?,
        };

        let create_response = deps
            .api_client
            .create_app(&create_app_request)
            .await
            .map_err(|e| anyhow!("Failed to create box: {}", e))?;

        create_response.app_id
    } else {
        // Box exists, use its ID
        existing_apps.apps[0].app_id
    };

    // Create a mapping of tool names to their deploy names
    let deploy_names: HashMap<String, String> = ftl_config
        .tools
        .iter()
        .filter_map(|(name, tool)| {
            tool.deploy
                .as_ref()
                .and_then(|d| d.name.as_ref())
                .map(|deploy_name| (name.clone(), deploy_name.clone()))
        })
        .collect();

    // Ensure components exist and push to ECR
    spinner.finish_and_clear();
    let deployed_components = ensure_components_and_push(
        &app_id.to_string(),
        &config.components,
        deploy_names,
        deps.clone(),
    )
    .await?;

    // Parse variables from command line
    let mut parsed_variables = parse_variables(&variables)?;

    // If we have ftl.toml, extract auth configuration and add to variables
    if deps.file_system.exists(Path::new("ftl.toml")) {
        add_auth_variables_from_ftl(&deps.file_system, &mut parsed_variables)?;
    }

    // Deploy to FTL
    deps.ui.print("");
    let spinner = deps.ui.create_spinner();
    spinner.enable_steady_tick(deps.clock.duration_from_millis(100));
    spinner.set_message("Starting box deployment...");

    // Refresh credentials before deployment in case the token expired
    let _fresh_credentials = match deps.credentials_provider.get_or_refresh_credentials().await {
        Ok(creds) => creds,
        Err(e) => {
            spinner.finish_and_clear();
            return Err(anyhow!("Failed to refresh authentication token: {}", e));
        }
    };

    let deployment = deploy_to_ftl_with_progress(
        deps.clone(),
        app_id.to_string(),
        deployed_components,
        parsed_variables,
        spinner,
    )
    .await?;

    // Display results
    deps.ui.print("");
    deps.ui
        .print_styled("✓ Box deployed successfully!", MessageStyle::Success);
    if let Some(deployment_url) = deployment.provider_url {
        deps.ui.print("");
        deps.ui.print(&format!("  MCP URL: {deployment_url}"));
        deps.ui.print("");
    }

    Ok(())
}

/// Parse KEY=VALUE variable pairs from command line arguments
pub fn parse_variables(variables: &[String]) -> Result<HashMap<String, String>> {
    let mut parsed = HashMap::new();

    for var in variables {
        let parts: Vec<&str> = var.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "Invalid variable format '{}'. Expected KEY=VALUE format.",
                var
            ));
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        if key.is_empty() {
            return Err(anyhow!("Variable key cannot be empty"));
        }

        parsed.insert(key.to_string(), value.to_string());
    }

    Ok(parsed)
}

/// Add auth-related variables from ftl.toml to the variables map
fn add_auth_variables_from_ftl(
    file_system: &Arc<dyn FileSystem>,
    variables: &mut HashMap<String, String>,
) -> Result<()> {
    use crate::config::ftl_config::FtlConfig;

    let content = file_system.read_to_string(Path::new("ftl.toml"))?;
    let config = FtlConfig::parse(&content)?;

    // Only add auth variables if they're not already provided via command line
    if config.auth.enabled && !variables.contains_key("auth_enabled") {
        variables.insert("auth_enabled".to_string(), "true".to_string());
    }

    let provider_type = config.auth.provider_type();
    if !provider_type.is_empty() && !variables.contains_key("auth_provider_type") {
        variables.insert("auth_provider_type".to_string(), provider_type.to_string());
    }

    let issuer = config.auth.issuer();
    if !issuer.is_empty() && !variables.contains_key("auth_provider_issuer") {
        variables.insert("auth_provider_issuer".to_string(), issuer.to_string());
    }

    let audience = config.auth.audience();
    if !audience.is_empty() && !variables.contains_key("auth_provider_audience") {
        variables.insert("auth_provider_audience".to_string(), audience.to_string());
    }

    // Add OIDC-specific variables if present
    if let Some(oidc) = &config.auth.oidc {
        if !oidc.provider_name.is_empty() && !variables.contains_key("auth_provider_name") {
            variables.insert("auth_provider_name".to_string(), oidc.provider_name.clone());
        }

        if !oidc.jwks_uri.is_empty() && !variables.contains_key("auth_provider_jwks_uri") {
            variables.insert("auth_provider_jwks_uri".to_string(), oidc.jwks_uri.clone());
        }

        if !oidc.authorize_endpoint.is_empty()
            && !variables.contains_key("auth_provider_authorize_endpoint")
        {
            variables.insert(
                "auth_provider_authorize_endpoint".to_string(),
                oidc.authorize_endpoint.clone(),
            );
        }

        if !oidc.token_endpoint.is_empty()
            && !variables.contains_key("auth_provider_token_endpoint")
        {
            variables.insert(
                "auth_provider_token_endpoint".to_string(),
                oidc.token_endpoint.clone(),
            );
        }

        if !oidc.userinfo_endpoint.is_empty()
            && !variables.contains_key("auth_provider_userinfo_endpoint")
        {
            variables.insert(
                "auth_provider_userinfo_endpoint".to_string(),
                oidc.userinfo_endpoint.clone(),
            );
        }

        if !oidc.allowed_domains.is_empty()
            && !variables.contains_key("auth_provider_allowed_domains")
        {
            variables.insert(
                "auth_provider_allowed_domains".to_string(),
                oidc.allowed_domains.clone(),
            );
        }
    }

    Ok(())
}

/// Parse deployment configuration from spin.toml
pub fn parse_deploy_config(
    file_system: &Arc<dyn FileSystem>,
    manifest_path: &Path,
) -> Result<DeployConfig> {
    // For temporary files, we need to read directly since FileSystem trait doesn't know about them
    let content = if manifest_path.starts_with("/tmp") || manifest_path.starts_with("/var/folders")
    {
        std::fs::read_to_string(manifest_path).context("Failed to read temporary spin.toml")?
    } else {
        file_system.read_to_string(manifest_path)?
    };
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
                        let allowed_outbound_hosts = component
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
                            allowed_outbound_hosts,
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
    ecr_creds: &types::CreateEcrTokenResponse,
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
async fn ensure_components_and_push(
    app_id: &str,
    components: &[ComponentInfo],
    deploy_names: HashMap<String, String>,
    deps: Arc<DeployDependencies>,
) -> Result<Vec<types::CreateDeploymentRequestComponentsItem>> {
    // Check if wkg is available before starting
    deps.command_executor
        .check_command_exists("wkg")
        .await
        .context(
            "wkg not found. Install from: https://github.com/bytecodealliance/wasm-pkg-tools",
        )?;

    // Step 1: Ensure all components exist with their repositories
    deps.ui.print("→ Ensuring components and repositories...");
    deps.ui.print("");
    
    // Build the update request with all components
    let component_updates: Vec<_> = components.iter().map(|comp| {
        let component_name = deploy_names
            .get(&comp.name)
            .cloned()
            .unwrap_or_else(|| comp.name.clone());
        
        Ok(types::UpdateComponentsRequestComponentsItem {
            component_name: component_name.as_str().try_into().map_err(|e| anyhow!("Invalid component name: {}", e))?,
            description: None,
        })
    }).collect::<Result<Vec<_>>>()?;
    
    let update_request = types::UpdateComponentsRequest {
        components: component_updates,
    };
    
    // Update all components in one atomic operation
    let update_response = deps.api_client
        .update_components(app_id, &update_request)
        .await
        .map_err(|e| anyhow!("Failed to update components: {}", e))?;
    
    // Log what changed
    if !update_response.changes.created.is_empty() {
        deps.ui.print_styled(
            &format!("  ✓ Created components: {}", update_response.changes.created.join(", ")),
            MessageStyle::Success
        );
    }
    if !update_response.changes.updated.is_empty() {
        deps.ui.print_styled(
            &format!("  ✓ Updated components: {}", update_response.changes.updated.join(", ")),
            MessageStyle::Success
        );
    }
    
    // Step 2: Push all components to their repositories
    deps.ui.print("");
    deps.ui.print(&format!(
        "→ Pushing {} components in parallel",
        components.len()
    ));
    deps.ui.print("");

    let multi_progress = deps.ui.create_multi_progress();
    let mut tasks = JoinSet::new();
    let deployed_components = Arc::new(Mutex::new(Vec::new()));

    // Track errors across all tasks
    let error_flag = Arc::new(Mutex::new(None::<String>));

    // Limit concurrent operations
    let semaphore = Arc::new(Semaphore::new(4));

    // Create a lookup map for component -> repository URI
    let component_repos: HashMap<String, String> = update_response.components
        .iter()
        .filter_map(|c| {
            c.repository_uri.as_ref().map(|uri| {
                (c.component_name.clone(), uri.clone())
            })
        })
        .collect();

    for component in components {
        let pb = multi_progress.add_spinner();
        pb.set_prefix(format!("[{}]", component.name));
        pb.set_message("Preparing to push...");
        pb.enable_steady_tick(deps.clock.duration_from_millis(100));

        let component = component.clone();
        let component_repos = component_repos.clone();
        let deploy_names = deploy_names.clone();
        let deps = deps.clone();
        let deployed_components = Arc::clone(&deployed_components);
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

            // Get the deployed component name (with any overrides applied)
            let component_name = deploy_names
                .get(&component.name)
                .cloned()
                .unwrap_or_else(|| component.name.clone());

            // Get repository URI from the update response
            let repository_uri = component_repos
                .get(&component_name)
                .ok_or_else(|| anyhow!("Component '{}' not found in update response", component_name))?
                .clone();

            // Push component with version tag
            pb.set_message(&format!("Pushing v{}...", component.version));
            let versioned_tag = format!("{}:{}", repository_uri, component.version);
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

            // Add to deployment request
            let mut components = deployed_components.lock().await;
            components.push(types::CreateDeploymentRequestComponentsItem {
                component_name: component_name.as_str().try_into()?,
                tag: component.version.as_str().try_into()?,
                allowed_hosts: component.allowed_outbound_hosts.clone().unwrap_or_default(),
            });

            let duration = start.elapsed();
            pb.finish_with_message(format!("✓ Pushed in {:.1}s", duration.as_secs_f64()));

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

    let components = Arc::try_unwrap(deployed_components).unwrap().into_inner();

    deps.ui.print("");
    deps.ui.print_styled(
        "✓ All components pushed successfully!",
        MessageStyle::Success,
    );

    Ok(components)
}

async fn poll_app_deployment_status_with_progress(
    deps: Arc<DeployDependencies>,
    app_id: &str,
    spinner: Box<dyn ftl_runtime::deps::ProgressIndicator>,
) -> Result<types::App> {
    let max_attempts = 60; // 5 minutes with 5-second intervals
    let mut attempts = 0;

    loop {
        if attempts >= max_attempts {
            spinner.finish_and_clear();
            return Err(anyhow!("Box deployment timeout after 5 minutes"));
        }

        let app = match deps.api_client.get_app(app_id).await {
            Ok(app) => app,
            Err(e) => {
                spinner.finish_and_clear();
                return Err(anyhow!("Failed to get box status: {}", e));
            }
        };

        // Update spinner message based on status
        let status_msg = match &app.status {
            types::AppStatus::Pending => "Initializing box deployment...",
            types::AppStatus::Creating => "Deploying box...",
            types::AppStatus::Active => "Box deployment succeeded!",
            types::AppStatus::Failed => "Box deployment failed",
            types::AppStatus::Deleting => "Box is being deleted",
            types::AppStatus::Deleted => "Box has been deleted",
        };

        spinner.set_message(status_msg);

        match app.status {
            types::AppStatus::Active => {
                spinner.finish_and_clear();
                return Ok(app);
            }
            types::AppStatus::Failed => {
                spinner.finish_and_clear();
                let error_msg = app
                    .provider_error
                    .as_deref()
                    .unwrap_or("Box deployment failed")
                    .to_string();
                return Err(anyhow!("Box deployment failed: {}", error_msg));
            }
            types::AppStatus::Deleted | types::AppStatus::Deleting => {
                spinner.finish_and_clear();
                return Err(anyhow!("Box was deleted during deployment"));
            }
            types::AppStatus::Pending | types::AppStatus::Creating => {
                // Continue polling for pending/creating statuses
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
    app_id: String,
    components: Vec<types::CreateDeploymentRequestComponentsItem>,
    variables: HashMap<String, String>,
    spinner: Box<dyn ftl_runtime::deps::ProgressIndicator>,
) -> Result<types::App> {
    // Create the deployment
    spinner.set_message("Creating box deployment...");

    // Use new API format with components
    let deployment_request = types::CreateDeploymentRequest {
        components,
        variables,
    };

    let deployment_response = deps
        .api_client
        .create_deployment(&app_id.to_string(), &deployment_request)
        .await
        .map_err(|e| {
            spinner.finish_and_clear();
            anyhow!("Failed to create box deployment: {}", e)
        })?;

    // Update spinner with deployment ID
    spinner.set_message(&format!(
        "Box deployment {} in progress...",
        &deployment_response.deployment_id
    ));

    // Poll the box status to know when deployment is complete
    poll_app_deployment_status_with_progress(deps, &app_id.to_string(), spinner).await
}

/// Deploy command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct DeployArgs {
    /// Variable(s) to be passed to the app (KEY=VALUE format)
    pub variables: Vec<String>,
}

// Build executor implementation
struct BuildExecutorImpl;

#[async_trait::async_trait]
impl BuildExecutor for BuildExecutorImpl {
    async fn execute(&self, path: Option<&Path>, release: bool) -> Result<()> {
        use crate::commands::build;

        let args = build::BuildArgs {
            path: path.map(std::path::Path::to_path_buf),
            release,
            export: None,
            export_out: None,
        };

        build::execute(args).await
    }
}

/// Execute the deploy command with default dependencies
pub async fn execute(args: DeployArgs) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_runtime::deps::{
        RealAsyncRuntime, RealClock, RealCommandExecutor, RealCredentialsProvider, RealFileSystem,
        RealFtlApiClient,
    };

    // Get credentials first to create authenticated API client
    let credentials_provider = Arc::new(RealCredentialsProvider);
    let credentials = credentials_provider.get_or_refresh_credentials().await?;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(DeployDependencies {
        file_system: Arc::new(RealFileSystem),
        command_executor: Arc::new(RealCommandExecutor),
        api_client: Arc::new(RealFtlApiClient::new_with_auth(
            ftl_runtime::api_client::Client::new(ftl_runtime::config::DEFAULT_API_BASE_URL),
            credentials.access_token.clone(),
        )),
        clock: Arc::new(RealClock),
        credentials_provider,
        ui: ui.clone(),
        build_executor: Arc::new(BuildExecutorImpl),
        async_runtime: Arc::new(RealAsyncRuntime),
    });

    execute_with_deps(deps, args.variables).await
}

#[cfg(test)]
#[path = "deploy_tests.rs"]
mod tests;
