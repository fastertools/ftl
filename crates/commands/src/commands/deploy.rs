//! Refactored deploy command with dependency injection for testability

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

use crate::component_resolver::{ComponentResolutionStrategy, ComponentResolver};
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
    /// Engine name from spin.toml
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
pub async fn execute_with_deps(deps: Arc<DeployDependencies>, args: DeployArgs) -> Result<()> {
    let project_path = PathBuf::from(".");
    let ftl_toml_path = project_path.join("ftl.toml");

    // Check if ftl.toml exists
    if !deps.file_system.exists(&ftl_toml_path) {
        return Err(anyhow!(
            "No ftl.toml found. Not in an FTL project directory?"
        ));
    }

    // Parse ftl.toml
    let ftl_content = deps.file_system.read_to_string(&ftl_toml_path)?;
    let ftl_config = crate::config::ftl_config::FtlConfig::parse(&ftl_content)?;

    // Display deployment header
    let project_name = &ftl_config.project.name;
    if args.dry_run {
        deps.ui.print_styled(
            &format!("{} Deploying {} to FTL Engine (DRY RUN)", "▶", project_name),
            MessageStyle::Bold,
        );
    } else {
        deps.ui.print_styled(
            &format!("{} Deploying {} to FTL Engine", "▶", project_name),
            MessageStyle::Bold,
        );
    }
    deps.ui.print("");

    // Resolve only user registry components for deployment
    let resolver = ComponentResolver::new(deps.ui.clone());
    let resolved_components = resolver
        .resolve_components(
            &ftl_config,
            ComponentResolutionStrategy::Deploy {
                push_user_only: true, // Only resolve user components, not MCP
            },
        )
        .await?;

    // Create spin.toml with resolved paths (for parsing components)
    let spin_content = crate::config::transpiler::create_spin_toml_with_resolved_paths(
        &ftl_config,
        resolved_components.mappings(),
        &project_path,
    )?;

    // Write spin.toml to temporary location
    let temp_dir = tempfile::Builder::new()
        .prefix("ftl-deploy-")
        .tempdir()
        .context("Failed to create temporary directory")?;
    let manifest_path = temp_dir.path().join("spin.toml");
    std::fs::write(&manifest_path, &spin_content).context("Failed to write temporary spin.toml")?;

    // Keep temp directory alive
    let _temp_dir = temp_dir.keep();

    // Run the deployment
    execute_deploy_inner(deps.clone(), args, manifest_path, ftl_config, true).await
}

/// Inner deployment logic separated for proper cleanup handling
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
async fn execute_deploy_inner(
    deps: Arc<DeployDependencies>,
    args: DeployArgs,
    manifest_path: PathBuf,
    ftl_config: crate::config::ftl_config::FtlConfig,
    _is_temp_manifest: bool,
) -> Result<()> {
    // Create a spinner for status updates
    let spinner = deps.ui.create_spinner();
    spinner.enable_steady_tick(deps.clock.duration_from_millis(100));

    // Determine which build profile to use for deployment
    // For now, use release mode if any component has deploy.profile = "release" or no profile specified
    let use_release = ftl_config.component.values().any(|component| {
        component
            .deploy
            .as_ref()
            .is_none_or(|d| d.profile == "release") // Default to release if no deploy config
    });

    // Build the project first
    spinner.finish_and_clear();

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

    // Create a mapping of component names to their deploy names
    let deploy_names: HashMap<String, String> = ftl_config
        .component
        .iter()
        .filter_map(|(name, component)| {
            component
                .deploy
                .as_ref()
                .and_then(|d| d.name.as_ref())
                .map(|deploy_name| (name.clone(), deploy_name.clone()))
        })
        .collect();

    // Parse variables following precedence: CLI flags > env vars > ftl.toml
    let mut parsed_variables = HashMap::new();

    // Step 1: Load from ftl.toml first (lowest priority)
    // Note: We use the already-parsed ftl_config to avoid re-reading the file
    add_variables_from_config(&ftl_config, &mut parsed_variables);
    add_auth_variables_from_config(&ftl_config, &mut parsed_variables);

    // Step 2: Override with environment variables (middle priority)
    // Spin uses SPIN_VARIABLE_ prefix for runtime variables
    for (key, value) in std::env::vars() {
        if let Some(var_name) = key.strip_prefix("SPIN_VARIABLE_") {
            parsed_variables.insert(var_name.to_string(), value);
        }
    }

    // Also handle FTL_ACCESS_CONTROL env var which affects auth_enabled
    if let Ok(access_control) = std::env::var("FTL_ACCESS_CONTROL") {
        match access_control.as_str() {
            "public" => {
                parsed_variables.insert("auth_enabled".to_string(), "false".to_string());
            }
            "private" | "custom" => {
                parsed_variables.insert("auth_enabled".to_string(), "true".to_string());

                // Set provider type based on mode
                if access_control == "private" {
                    // Use FTL's AuthKit for private mode (unless overridden by SPIN_VARIABLE_)
                    if !parsed_variables.contains_key("mcp_provider_type") {
                        parsed_variables.insert("mcp_provider_type".to_string(), "jwt".to_string());
                    }
                    if !parsed_variables.contains_key("mcp_jwt_issuer") {
                        parsed_variables.insert(
                            "mcp_jwt_issuer".to_string(),
                            "https://divine-lion-50-staging.authkit.app".to_string(),
                        );
                    }
                }
            }
            _ => {} // Invalid values handled later
        }
    }

    // Apply other FTL_ env vars for auth configuration
    if let Ok(provider) = std::env::var("FTL_AUTH_PROVIDER") {
        parsed_variables.insert("mcp_provider_type".to_string(), provider);
    }
    if let Ok(issuer) = std::env::var("FTL_JWT_ISSUER") {
        parsed_variables.insert("mcp_jwt_issuer".to_string(), issuer);
    }
    if let Ok(audience) = std::env::var("FTL_AUTH_AUDIENCE") {
        parsed_variables.insert("mcp_jwt_audience".to_string(), audience);
    }

    // Step 3: Override with CLI variables (highest priority)
    let cli_variables = parse_variables(&args.variables)?;
    for (key, value) in cli_variables {
        parsed_variables.insert(key, value);
    }

    // Step 4: Handle auth-related CLI flags which should also affect variables
    // The --access-control flag should override auth_enabled and related variables
    // IMPORTANT: If --auth-issuer is provided, treat it as custom auth regardless of access_control
    if let Some(access_control) = &args.access_control {
        match access_control.as_str() {
            "public" => {
                // Disable auth
                parsed_variables.insert("auth_enabled".to_string(), "false".to_string());
            }
            "private" => {
                // Enable auth
                parsed_variables.insert("auth_enabled".to_string(), "true".to_string());

                // If jwt_issuer is provided, treat as custom auth
                if let Some(issuer) = &args.jwt_issuer {
                    // Custom auth mode with custom issuer
                    parsed_variables.insert("mcp_provider_type".to_string(), "jwt".to_string());
                    parsed_variables.insert("mcp_jwt_issuer".to_string(), issuer.clone());
                } else if access_control == "private" {
                    // For private mode without custom OAuth, use FTL's AuthKit
                    // Check if we need to override issuer for tenant-scoped AuthKit
                    if !parsed_variables.contains_key("mcp_jwt_issuer") {
                        parsed_variables.insert(
                            "mcp_jwt_issuer".to_string(),
                            "https://divine-lion-50-staging.authkit.app".to_string(),
                        );
                    }
                    if !parsed_variables.contains_key("mcp_provider_type") {
                        parsed_variables.insert("mcp_provider_type".to_string(), "jwt".to_string());
                    }
                }
            }
            _ => {
                // Invalid value will be caught later in resolve_auth_config
            }
        }
    }

    // If this is a dry run, display summary and exit
    if args.dry_run {
        spinner.finish_and_clear();
        let auth_config = resolve_auth_config(&deps.file_system, &args)?;
        let auth_mode = auth_config.as_ref().map(|(mode, _, _, _)| mode);
        display_dry_run_summary(
            &deps,
            &config,
            use_release,
            &parsed_variables,
            auth_mode,
            &deploy_names,
        );
        return Ok(());
    }

    // Check if engine exists and show confirmation before proceeding
    if !args.yes && !args.dry_run {
        spinner.finish_and_clear();

        // Check if engine already exists
        let check_spinner = deps.ui.create_spinner();
        check_spinner.set_message("Checking engine status...");
        check_spinner.enable_steady_tick(deps.clock.duration_from_millis(100));

        let existing_apps = deps
            .api_client
            .list_apps(None, None, Some(&config.app_name))
            .await
            .map_err(|e| anyhow!("Failed to check existing apps: {}", e))?;

        check_spinner.finish_and_clear();

        deps.ui.print("");
        deps.ui
            .print_styled("Deployment Summary", MessageStyle::Cyan);
        deps.ui.print("");
        let app_name = &config.app_name;
        deps.ui.print(&format!("Engine: {app_name}"));
        let component_count = config.components.len();
        deps.ui
            .print(&format!("Components: {component_count} tools"));
        let profile = if use_release { "release" } else { "debug" };
        deps.ui.print(&format!("Build Profile: {profile}"));

        if existing_apps.apps.is_empty() {
            deps.ui.print("");
            deps.ui
                .print_styled("This will create a new engine.", MessageStyle::Cyan);
        } else {
            let app = &existing_apps.apps[0];
            deps.ui.print("");
            deps.ui
                .print_styled("⚠ Existing engine found", MessageStyle::Yellow);
            deps.ui.print(&format!("  Status: {:?}", app.status));
            if let Some(url) = &app.provider_url {
                deps.ui.print(&format!("  Current URL: {url}"));
            }
            if !app.updated_at.is_empty() {
                let updated_at = &app.updated_at;
                deps.ui.print(&format!("  Last Updated: {updated_at}"));
            }
            deps.ui.print("");
            deps.ui.print_styled(
                "This deployment will update the existing engine.",
                MessageStyle::Yellow,
            );
        }

        deps.ui.print("");

        if !deps.ui.prompt_confirm("Continue with deployment?", true)? {
            deps.ui
                .print_styled("Deployment cancelled", MessageStyle::Yellow);
            return Ok(());
        }

        deps.ui.print("");
    } else {
        // For automated deployment (args.yes == true), we still need to clear the spinner
        spinner.finish_and_clear();
    }

    // Create new spinner for the rest of the deployment
    let spinner = deps.ui.create_spinner();
    spinner.enable_steady_tick(deps.clock.duration_from_millis(100));

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
    spinner.set_message("Checking if engine exists...");
    let existing_apps = deps
        .api_client
        .list_apps(None, None, Some(&config.app_name))
        .await
        .map_err(|e| anyhow!("Failed to check existing apps: {}", e))?;

    let app_id = if existing_apps.apps.is_empty() {
        // Engine doesn't exist, create it first
        spinner.set_message("Creating engine...");
        let create_app_request = types::CreateAppRequest {
            app_name: config
                .app_name
                .as_str()
                .try_into()
                .map_err(|e| anyhow!("Invalid app name: {}", e))?,
        };

        let create_response = deps
            .api_client
            .create_app(&create_app_request)
            .await
            .map_err(|e| anyhow!("Failed to create engine: {}", e))?;

        create_response.app_id
    } else {
        // Box exists, use its ID
        existing_apps.apps[0].app_id
    };

    // Ensure components exist and push to ECR
    spinner.finish_and_clear();
    let deployed_components = ensure_components_and_push(
        &app_id.to_string(),
        &config.components,
        deploy_names.clone(),
        deps.clone(),
    )
    .await?;

    // Update auth configuration BEFORE deployment
    // This follows the hierarchy: CLI flags > env vars > ftl.toml
    let auth_config = resolve_auth_config(&deps.file_system, &args)?;
    if let Some((mode, provider, issuer, audience)) = auth_config {
        deps.ui.print("");
        deps.ui.print_styled(
            "→ Configuring MCP authorization settings...",
            MessageStyle::Cyan,
        );

        update_auth_config(
            deps.clone(),
            &app_id.to_string(),
            &mode,
            provider.as_ref(),
            issuer.as_ref(),
            audience.as_ref(),
        )
        .await?;

        deps.ui.print_styled(
            &format!("✓ MCP authorization set to: {mode}"),
            MessageStyle::Warning,
        );
    }

    // Deploy to FTL
    deps.ui.print("");

    // Show deployment variables first, before creating any spinners
    if parsed_variables.is_empty() {
        deps.ui
            .print_styled("→ No variables to deploy", MessageStyle::Yellow);
    } else {
        deps.ui.print_styled(
            &format!(
                "→ Deploying with {} variable{}:",
                parsed_variables.len(),
                if parsed_variables.len() == 1 { "" } else { "s" }
            ),
            MessageStyle::Cyan,
        );

        // Sort variables for consistent display
        let mut sorted_vars: Vec<_> = parsed_variables.iter().collect();
        sorted_vars.sort_by_key(|(k, _)| k.as_str());

        for (key, value) in sorted_vars {
            // Check if this is a sensitive variable
            let is_sensitive = is_sensitive_variable(key);

            let display_value = if is_sensitive {
                // Show first few chars for debugging, but redact the rest
                if value.len() > 4 {
                    format!("{}***", &value[..2])
                } else {
                    "***".to_string()
                }
            } else {
                value.clone()
            };

            // Add a lock icon for sensitive variables
            let icon = if is_sensitive { "🔒 " } else { "   " };
            deps.ui.print(&format!("{icon}{key} = {display_value}"));
        }
    }

    deps.ui.print("");

    // Now create the spinner for deployment after all output is done
    let spinner = deps.ui.create_spinner();
    spinner.enable_steady_tick(deps.clock.duration_from_millis(100));
    spinner.set_message("Starting engine deployment...");

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
    deps.ui.print_styled("✓ Deployed!", MessageStyle::Success);
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

/// Check if a variable name indicates it contains sensitive data
pub(crate) fn is_sensitive_variable(name: &str) -> bool {
    let name_lower = name.to_lowercase();

    // Specific non-sensitive auth configuration variables
    let config_exceptions = [
        "auth_enabled",
        "mcp_jwt_issuer",
        "mcp_jwt_audience",
        "mcp_jwt_required_scopes",
        "mcp_jwt_algorithm",
        "mcp_provider_type",
        "mcp_jwt_jwks_uri",
        "mcp_oauth_authorize_endpoint",
        "mcp_oauth_token_endpoint",
        "mcp_oauth_userinfo_endpoint",
    ];

    // If it's a known configuration variable, it's not sensitive
    if config_exceptions.iter().any(|&exc| name_lower == exc) {
        return false;
    }

    // Common patterns for sensitive variable names
    let sensitive_patterns = [
        "token",
        "key",
        "secret",
        "password",
        "pwd",
        "pass",
        "auth",
        "credential",
        "cred",
        "api_key",
        "apikey",
        "private",
        "priv",
        "cert",
        "certificate",
        "sign",
        "jwt",
        "bearer",
        "oauth",
        "access",
        "refresh",
    ];

    // Check if the variable name contains any sensitive pattern
    sensitive_patterns
        .iter()
        .any(|pattern| name_lower.contains(pattern))
}

/// Display dry-run summary of what would be deployed
fn display_dry_run_summary(
    deps: &Arc<DeployDependencies>,
    config: &DeployConfig,
    use_release: bool,
    parsed_variables: &HashMap<String, String>,
    auth_mode: Option<&String>,
    deploy_names: &HashMap<String, String>,
) {
    deps.ui.print("");
    deps.ui.print_styled(
        "🔍 DRY RUN MODE - No changes will be made",
        MessageStyle::Bold,
    );
    deps.ui.print("");

    // Engine information
    deps.ui
        .print_styled("Engine Configuration:", MessageStyle::Cyan);
    let app_name = &config.app_name;
    deps.ui.print(&format!("  Name: {app_name}"));
    deps.ui.print(&format!(
        "  Build Profile: {}",
        if use_release { "release" } else { "debug" }
    ));
    deps.ui.print("");

    // Components to deploy
    deps.ui
        .print_styled("Components to Deploy:", MessageStyle::Cyan);
    for component in &config.components {
        let deploy_name = deploy_names
            .get(&component.name)
            .cloned()
            .unwrap_or_else(|| component.name.clone());

        let version = &component.version;
        deps.ui.print(&format!("  • {deploy_name} (v{version})"));
        let source_path = &component.source_path;
        deps.ui.print(&format!("    Source: {source_path}"));
        if let Some(hosts) = &component.allowed_outbound_hosts
            && !hosts.is_empty()
        {
            let hosts_str = hosts.join(", ");
            deps.ui
                .print(&format!("    Allowed outbound hosts: {hosts_str}"));
        }
    }
    deps.ui.print("");

    // Variables
    if !parsed_variables.is_empty() {
        deps.ui.print_styled(
            &format!("Variables ({}):", parsed_variables.len()),
            MessageStyle::Cyan,
        );

        let mut sorted_vars: Vec<_> = parsed_variables.iter().collect();
        sorted_vars.sort_by_key(|(k, _)| k.as_str());

        for (key, value) in sorted_vars {
            let is_sensitive = is_sensitive_variable(key);
            let display_value = if is_sensitive {
                if value.len() > 4 {
                    format!("{}***", &value[..2])
                } else {
                    "***".to_string()
                }
            } else {
                value.clone()
            };

            let icon = if is_sensitive { "🔒 " } else { "   " };
            deps.ui.print(&format!("{icon}{key} = {display_value}"));
        }
        deps.ui.print("");
    }

    // Authorization configuration
    if let Some(mode) = auth_mode {
        deps.ui
            .print_styled("Authorization Configuration:", MessageStyle::Cyan);
        deps.ui.print(&format!("  Mode: {mode}"));
        deps.ui.print("");
    }

    deps.ui.print_styled(
        "✓ Dry run complete. No changes were made.",
        MessageStyle::Success,
    );
    deps.ui.print("");
    deps.ui
        .print("To perform the actual deployment, run the command without --dry-run");
}

/// Update authorization configuration for a deployed app
async fn update_auth_config(
    deps: Arc<DeployDependencies>,
    app_id: &str,
    access_control_mode: &str,
    auth_provider: Option<&String>,
    auth_issuer: Option<&String>,
    auth_audience: Option<&String>,
) -> Result<()> {
    use types::UpdateAuthConfigRequestAccessControl;

    let access_control = match access_control_mode {
        "public" => UpdateAuthConfigRequestAccessControl::Public,
        "private" => UpdateAuthConfigRequestAccessControl::Private,
        "custom" => UpdateAuthConfigRequestAccessControl::Custom,
        _ => {
            return Err(anyhow!(
                "Invalid access control mode: {}. Must be one of: public, private, custom",
                access_control_mode
            ));
        }
    };

    let mut custom_config = None;

    // Handle different access control modes
    match access_control_mode {
        "public" | "private" => {
            // No additional config needed
        }
        "custom" => {
            // Custom mode is only reached when we have OAuth config or --jwt-issuer
            // The issuer should always be present at this point
            if auth_issuer.is_none() {
                return Err(anyhow!("Internal error: custom auth mode without issuer"));
            }

            custom_config = Some(types::UpdateAuthConfigRequestCustomConfig {
                provider: auth_provider
                    .cloned()
                    .unwrap_or_else(|| "jwt".to_string())
                    .try_into()
                    .map_err(|_| anyhow!("Invalid provider name"))?,
                issuer: auth_issuer
                    .unwrap()
                    .clone()
                    .try_into()
                    .map_err(|_| anyhow!("Invalid issuer URL"))?,
                audience: auth_audience.cloned(),
                jwks_uri: None,
            });
        }
        _ => unreachable!(), // Already handled above
    }

    let request = types::UpdateAuthConfigRequest {
        access_control,
        custom_config,
    };

    deps.api_client
        .update_auth_config(app_id, &request)
        .await
        .map_err(|e| anyhow!("Failed to update auth config: {}", e))?;

    Ok(())
}

/// Add auth-related variables from parsed `FtlConfig` to the variables map
fn add_auth_variables_from_config(
    config: &crate::config::ftl_config::FtlConfig,
    variables: &mut HashMap<String, String>,
) {
    // Always add auth_enabled variable (will be overridden by env/CLI if present)
    variables.insert(
        "auth_enabled".to_string(),
        config.is_auth_enabled().to_string(),
    );

    // Only add other auth-related variables if auth is enabled
    if config.is_auth_enabled() {
        // Add provider type
        variables.insert(
            "mcp_provider_type".to_string(),
            config.auth_provider_type().to_string(),
        );

        // Add issuer
        let issuer = config.auth_issuer();
        if !issuer.is_empty() {
            variables.insert("mcp_jwt_issuer".to_string(), issuer.to_string());
        }

        // Add audience
        let audience = config.auth_audience();
        if !audience.is_empty() {
            variables.insert("mcp_jwt_audience".to_string(), audience.to_string());
        }

        // Add required scopes
        let required_scopes = config.auth_required_scopes();
        if !required_scopes.is_empty() {
            variables.insert(
                "mcp_jwt_required_scopes".to_string(),
                required_scopes.to_string(),
            );
        }
    }

    // Add OAuth-specific variables if present and auth is enabled
    if config.is_auth_enabled()
        && let Some(oauth) = &config.oauth
    {
        if !oauth.jwks_uri.is_empty() {
            variables.insert("mcp_jwt_jwks_uri".to_string(), oauth.jwks_uri.clone());
        }

        if !oauth.public_key.is_empty() {
            variables.insert("mcp_jwt_public_key".to_string(), oauth.public_key.clone());
        }

        if !oauth.algorithm.is_empty() {
            variables.insert("mcp_jwt_algorithm".to_string(), oauth.algorithm.clone());
        }

        if !oauth.authorize_endpoint.is_empty() {
            variables.insert(
                "mcp_oauth_authorize_endpoint".to_string(),
                oauth.authorize_endpoint.clone(),
            );
        }

        if !oauth.token_endpoint.is_empty() {
            variables.insert(
                "mcp_oauth_token_endpoint".to_string(),
                oauth.token_endpoint.clone(),
            );
        }

        if !oauth.userinfo_endpoint.is_empty() {
            variables.insert(
                "mcp_oauth_userinfo_endpoint".to_string(),
                oauth.userinfo_endpoint.clone(),
            );
        }
    }
}

/// Add general variables from parsed `FtlConfig` to the variables map
fn add_variables_from_config(
    config: &crate::config::ftl_config::FtlConfig,
    variables: &mut HashMap<String, String>,
) {
    use crate::config::ftl_config::ApplicationVariable;

    // Add application-level variables that have default values
    // Required variables without defaults must be provided via CLI or env vars
    for (name, var) in &config.variables {
        match var {
            ApplicationVariable::Default { default } => {
                // Always add defaults from ftl.toml (will be overridden by env/CLI if present)
                variables.insert(name.clone(), default.clone());
            }
            ApplicationVariable::Required { .. } => {
                // Required variables must be provided at runtime
                // We don't add them here, they'll be handled by Spin
            }
        }
    }
}

/// Parse deployment configuration from spin.toml
pub fn parse_deploy_config(
    file_system: &Arc<dyn FileSystem>,
    manifest_path: &Path,
) -> Result<DeployConfig> {
    // Always use the FileSystem trait for consistency and testability
    let content = file_system
        .read_to_string(manifest_path)
        .context("Failed to read spin.toml")?;
    let toml: toml::Value = toml::from_str(&content).context("Failed to parse spin.toml")?;

    let app_name = toml
        .get("application")
        .and_then(|app| app.get("name"))
        .and_then(|name| name.as_str())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| anyhow!("No application name found in spin.toml"))?;

    let mut components = Vec::new();

    // Look for user components (not MCP system components)
    if let Some(components_table) = toml.get("component").and_then(|c| c.as_table()) {
        for (name, component) in components_table {
            // Skip MCP system components
            if name == "mcp" || name == "ftl-mcp-gateway" {
                continue;
            }

            if let Some(source) = component.get("source") {
                // Both local files and registry components should be deployed
                // The source_path will be the actual local path to the WASM file
                let source_path = if let Some(path) = source.as_str() {
                    // Local file source
                    path.to_string()
                } else if source.is_table() {
                    // Registry source - skip for now, will be handled separately if needed
                    // User components from registries should be pulled locally first
                    continue;
                } else {
                    continue;
                };

                // Only process local WASM files
                if !source_path.to_lowercase().ends_with(".wasm") || source_path.contains("://") {
                    continue;
                }

                // Try to extract version
                let version = extract_component_version(file_system, name, &source_path)?;

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
                    source_path,
                    version,
                    allowed_outbound_hosts,
                });
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

    // Try pyproject.toml for Python projects
    let pyproject_path = component_dir.join("pyproject.toml");
    if file_system.exists(&pyproject_path) {
        let pyproject_content = file_system.read_to_string(&pyproject_path)?;
        let pyproject_toml: toml::Value = toml::from_str(&pyproject_content)?;
        if let Some(version) = pyproject_toml
            .get("project")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
        {
            return Ok(version.to_string());
        }
    }

    // Try go.mod for Go projects
    let go_mod_path = component_dir.join("go.mod");
    if file_system.exists(&go_mod_path) {
        let go_mod_content = file_system.read_to_string(&go_mod_path)?;
        // Go modules don't have a standard version field in go.mod
        // Look for a version comment pattern: // Version: vX.Y.Z
        if let Some(version_line) = go_mod_content
            .lines()
            .find(|line| line.contains("// Version:"))
            && let Some(version_str) = version_line.split("// Version:").nth(1)
        {
            let version = version_str.trim().trim_start_matches('v');
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
    //
    // Build the update request with all components
    let component_updates: Vec<_> = components
        .iter()
        .map(|comp| {
            let component_name = deploy_names
                .get(&comp.name)
                .cloned()
                .unwrap_or_else(|| comp.name.clone());

            Ok(types::UpdateComponentsRequestComponentsItem {
                component_name: component_name
                    .as_str()
                    .try_into()
                    .map_err(|e| anyhow!("Invalid component name: {}", e))?,
                description: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let update_request = types::UpdateComponentsRequest {
        components: component_updates,
    };

    // Update all components in one atomic operation
    let update_response = deps
        .api_client
        .update_components(app_id, &update_request)
        .await
        .map_err(|e| anyhow!("Failed to update components: {}", e))?;

    // Log what changed
    if !update_response.changes.created.is_empty() {
        deps.ui.print_styled(
            &format!(
                "  ✓ Created components: {}",
                update_response.changes.created.join(", ")
            ),
            MessageStyle::Success,
        );
    }
    if !update_response.changes.updated.is_empty() {
        deps.ui.print_styled(
            &format!(
                "  ✓ Updated components: {}",
                update_response.changes.updated.join(", ")
            ),
            MessageStyle::Success,
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
    let component_repos: HashMap<String, String> = update_response
        .components
        .iter()
        .filter_map(|c| {
            c.repository_uri
                .as_ref()
                .map(|uri| (c.component_name.clone(), uri.clone()))
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
                .ok_or_else(|| {
                    anyhow!(
                        "Component '{}' not found in update response",
                        component_name
                    )
                })?
                .clone();

            // Push component with version tag
            pb.set_message(&format!("Pushing v{}...", component.version));
            let version = &component.version;
            let versioned_tag = format!("{repository_uri}:{version}");
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
        if let Err(e) = result?
            && first_error.is_none()
        {
            first_error = Some(e);
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
            return Err(anyhow!("Engine deployment timeout after 5 minutes"));
        }

        let app = match deps.api_client.get_app(app_id).await {
            Ok(app) => app,
            Err(e) => {
                spinner.finish_and_clear();
                return Err(anyhow!("Failed to get engine status: {}", e));
            }
        };

        // Update spinner message based on status
        let status_msg = match &app.status {
            types::AppStatus::Pending => "Initializing engine deployment...",
            types::AppStatus::Creating => "Deploying engine...",
            types::AppStatus::Active => "Engine deployment succeeded!",
            types::AppStatus::Failed => "Engine deployment failed",
            types::AppStatus::Deleting => "Engine is being deleted",
            types::AppStatus::Deleted => "Engine has been deleted",
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
                    .unwrap_or("Engine deployment failed")
                    .to_string();
                return Err(anyhow!("Engine deployment failed: {}", error_msg));
            }
            types::AppStatus::Deleted | types::AppStatus::Deleting => {
                spinner.finish_and_clear();
                return Err(anyhow!("Engine was deleted during deployment"));
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
    spinner.set_message("Creating engine deployment...");

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
            anyhow!("Failed to create engine deployment: {}", e)
        })?;

    // Update spinner with deployment ID
    spinner.set_message(&format!(
        "Engine deployment {} in progress...",
        &deployment_response.deployment_id
    ));

    // Poll the engine status to know when deployment is complete
    poll_app_deployment_status_with_progress(deps, &app_id.to_string(), spinner).await
}

/// Deploy command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct DeployArgs {
    /// Variable(s) to be passed to the app (KEY=VALUE format)
    pub variables: Vec<String>,
    /// Access control mode to set for the deployed app
    pub access_control: Option<String>,
    /// JWT issuer URL
    pub jwt_issuer: Option<String>,
    /// Run without making any changes (preview what would be deployed)
    pub dry_run: bool,
    /// Skip confirmation prompt
    pub yes: bool,
}

/// Auth configuration resolved from various sources
type ResolvedAuthConfig = (String, Option<String>, Option<String>, Option<String>);

/// Resolve auth configuration following hierarchy: CLI flags > env vars > ftl.toml
fn resolve_auth_config(
    file_system: &Arc<dyn FileSystem>,
    args: &DeployArgs,
) -> Result<Option<ResolvedAuthConfig>> {
    use crate::config::ftl_config::FtlConfig;

    // Start with ftl.toml as the base
    let mut auth_mode = None;
    let mut auth_provider = None;
    let mut auth_issuer = None;
    let mut auth_audience = None;

    // Load from ftl.toml if it exists
    if file_system.exists(Path::new("ftl.toml")) {
        let content = file_system.read_to_string(Path::new("ftl.toml"))?;
        let config = FtlConfig::parse(&content)?;

        // Determine auth mode based on configuration
        if config.project.access_control == "public" {
            auth_mode = Some("public".to_string());
        } else if config.project.access_control == "private" {
            // Check if we have custom OAuth config
            if config.oauth.is_some() {
                auth_mode = Some("custom".to_string());
            } else {
                auth_mode = Some("private".to_string());
            }
        }

        // Extract provider details only when auth is enabled
        if config.is_auth_enabled() {
            // Extract provider details based on configuration
            if let Some(oauth) = &config.oauth {
                auth_provider = Some("oauth".to_string());
                auth_issuer = Some(oauth.issuer.clone());
                auth_audience = if oauth.audience.is_empty() {
                    None
                } else {
                    Some(oauth.audience.clone())
                };
            } else {
                // Using FTL's built-in AuthKit
                auth_provider = Some("jwt".to_string());
                auth_issuer = Some(config.auth_issuer().to_string());
                auth_audience = None; // FTL AuthKit doesn't use audience
            }
        }
    }

    // Override with environment variables if set
    if let Ok(mode) = std::env::var("FTL_ACCESS_CONTROL") {
        auth_mode = Some(mode);
    }
    if let Ok(provider) = std::env::var("FTL_AUTH_PROVIDER") {
        auth_provider = Some(provider);
    }
    if let Ok(issuer) = std::env::var("FTL_JWT_ISSUER") {
        auth_issuer = Some(issuer);
    }
    if let Ok(audience) = std::env::var("FTL_AUTH_AUDIENCE") {
        auth_audience = Some(audience);
    }

    // Override with CLI flags (highest priority)
    // IMPORTANT: If --jwt-issuer is provided, treat as custom auth
    if let Some(issuer) = &args.jwt_issuer {
        // When jwt_issuer is explicitly provided, this is custom auth
        auth_mode = Some("custom".to_string());
        auth_issuer = Some(issuer.clone());
        // Always use jwt provider for custom auth
        auth_provider = Some("jwt".to_string());
    }

    // Set access control mode if provided and no custom issuer override
    if let Some(mode) = &args.access_control
        && args.jwt_issuer.is_none()
    {
        auth_mode = Some(mode.clone());
    }

    // Return None if no auth mode is configured
    match auth_mode {
        Some(mode) => Ok(Some((mode, auth_provider, auth_issuer, auth_audience))),
        None => Ok(None),
    }
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

    execute_with_deps(deps, args).await
}

#[cfg(test)]
#[path = "deploy_tests.rs"]
mod tests;
