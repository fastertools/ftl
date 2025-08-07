//! Transpiler for converting ftl.toml to spin.toml format
//!
//! This module uses type-safe schemas for both FTL and Spin configurations,
//! ensuring robust and accurate transpilation between formats.

use super::ftl_config::{ApplicationVariable, FtlConfig, ToolConfig};
use super::spin_config::{
    ComponentBuildConfig, ComponentConfig, ComponentSource, HttpTrigger, RouteConfig, SpinConfig,
    SpinVariable, TriggerConfig,
};
use anyhow::{Context, Result};
use ftl_runtime::deps::FileSystem;
use std::collections::HashMap;
use std::sync::Arc;

/// Parse a component source into a Spin source configuration
/// Handles both registry URIs and local files, using `default_registry` when needed
fn parse_component_source(source: &str, _default_registry: Option<&str>) -> ComponentSource {
    // For local WASM files, use them directly
    ComponentSource::Local(source.to_string())
}

/// Transpile an FTL configuration to Spin configuration
#[allow(clippy::too_many_lines)]
pub fn transpile_ftl_to_spin(ftl_config: &FtlConfig) -> Result<String> {
    // Create the base Spin configuration
    let mut spin_config = SpinConfig::new(ftl_config.project.name.clone());

    // Set application metadata
    spin_config
        .application
        .version
        .clone_from(&ftl_config.project.version);
    spin_config
        .application
        .description
        .clone_from(&ftl_config.project.description);
    spin_config
        .application
        .authors
        .clone_from(&ftl_config.project.authors);

    // Build variables
    let mut variables = HashMap::new();

    // Add application-level variables
    for (name, var) in &ftl_config.variables {
        let spin_var = match var {
            ApplicationVariable::Default { default } => SpinVariable::Default {
                default: default.clone(),
            },
            ApplicationVariable::Required { required: _ } => {
                SpinVariable::Required { required: true }
            }
        };
        variables.insert(name.clone(), spin_var);
    }

    // Add system variables
    let tool_names = ftl_config.tool_components().join(",");
    variables.insert(
        "tool_components".to_string(),
        SpinVariable::Default {
            default: tool_names,
        },
    );

    // Always include auth_enabled variable
    variables.insert(
        "auth_enabled".to_string(),
        SpinVariable::Default {
            default: ftl_config.is_auth_enabled().to_string(),
        },
    );

    // Only add other auth variables if auth is enabled
    if ftl_config.is_auth_enabled() {
        // Add tenant_id variable for private mode without OIDC (platform will provide the value)
        if ftl_config.project.access_control == "private" && ftl_config.oidc.is_none() {
            variables.insert(
                "mcp_tenant_id".to_string(),
                SpinVariable::Required { required: true },
            );
        } else {
            // For public mode or private with OIDC, tenant_id is empty
            variables.insert(
                "mcp_tenant_id".to_string(),
                SpinVariable::Default {
                    default: String::new(),
                },
            );
        }
        // Core MCP variables
        variables.insert(
            "mcp_gateway_url".to_string(),
            SpinVariable::Default {
                default: "http://ftl-mcp-gateway.spin.internal/mcp-internal".to_string(),
            },
        );
        variables.insert(
            "mcp_trace_header".to_string(),
            SpinVariable::Default {
                default: "x-trace-id".to_string(),
            },
        );
        variables.insert(
            "mcp_provider_type".to_string(),
            SpinVariable::Default {
                default: ftl_config.auth_provider_type().to_string(),
            },
        );

        // JWT provider variables (both FTL AuthKit and custom OIDC use JWT)
        variables.insert(
            "mcp_jwt_issuer".to_string(),
            SpinVariable::Default {
                default: ftl_config.auth_issuer().to_string(),
            },
        );
        variables.insert(
            "mcp_jwt_audience".to_string(),
            SpinVariable::Default {
                default: ftl_config.auth_audience().to_string(),
            },
        );
        variables.insert(
            "mcp_jwt_required_scopes".to_string(),
            SpinVariable::Default {
                default: ftl_config.auth_required_scopes().to_string(),
            },
        );

        // JWKS URI - empty for FTL AuthKit (auto-derived), explicit for OIDC
        let jwks_uri = if let Some(oidc) = &ftl_config.oidc {
            oidc.jwks_uri.clone()
        } else {
            String::new()
        };
        variables.insert(
            "mcp_jwt_jwks_uri".to_string(),
            SpinVariable::Default { default: jwks_uri },
        );

        // Public key and algorithm (OIDC only)
        if let Some(oidc) = &ftl_config.oidc {
            variables.insert(
                "mcp_jwt_public_key".to_string(),
                SpinVariable::Default {
                    default: oidc.public_key.clone(),
                },
            );
            variables.insert(
                "mcp_jwt_algorithm".to_string(),
                SpinVariable::Default {
                    default: oidc.algorithm.clone(),
                },
            );
        } else {
            variables.insert(
                "mcp_jwt_public_key".to_string(),
                SpinVariable::Default {
                    default: String::new(),
                },
            );
            variables.insert(
                "mcp_jwt_algorithm".to_string(),
                SpinVariable::Default {
                    default: String::new(),
                },
            );
        }

        // OAuth discovery endpoints
        if let Some(oidc) = &ftl_config.oidc {
            variables.insert(
                "mcp_oauth_authorize_endpoint".to_string(),
                SpinVariable::Default {
                    default: oidc.authorize_endpoint.clone(),
                },
            );
            variables.insert(
                "mcp_oauth_token_endpoint".to_string(),
                SpinVariable::Default {
                    default: oidc.token_endpoint.clone(),
                },
            );
            variables.insert(
                "mcp_oauth_userinfo_endpoint".to_string(),
                SpinVariable::Default {
                    default: oidc.userinfo_endpoint.clone(),
                },
            );
        } else {
            // Empty defaults for OAuth endpoints
            let oauth_vars = [
                "mcp_oauth_authorize_endpoint",
                "mcp_oauth_token_endpoint",
                "mcp_oauth_userinfo_endpoint",
            ];
            for var in &oauth_vars {
                variables.insert(
                    (*var).to_string(),
                    SpinVariable::Default {
                        default: String::new(),
                    },
                );
            }
        }

        // Static provider variables - no longer supported in new configuration
        variables.insert(
            "mcp_static_tokens".to_string(),
            SpinVariable::Default {
                default: String::new(),
            },
        );
    }

    spin_config.variables = variables;

    // Build components
    let mut components = HashMap::new();

    // Get default registry from project config
    let default_registry = ftl_config.project.default_registry.as_deref();

    if ftl_config.is_auth_enabled() {
        // When auth is enabled, add authorizer as "mcp" and gateway as "ftl-mcp-gateway"
        components.insert(
            "mcp".to_string(),
            create_mcp_component(&ftl_config.mcp.authorizer, default_registry),
        );
        components.insert(
            "ftl-mcp-gateway".to_string(),
            create_gateway_component(
                &ftl_config.mcp.gateway,
                ftl_config.mcp.validate_arguments,
                default_registry,
            ),
        );
    } else {
        // When auth is disabled, add gateway as "mcp" for consistent route naming
        components.insert(
            "mcp".to_string(),
            create_gateway_component(
                &ftl_config.mcp.gateway,
                ftl_config.mcp.validate_arguments,
                default_registry,
            ),
        );
    }

    // Add tool components
    for (tool_name, tool_config) in &ftl_config.tools {
        components.insert(
            tool_name.clone(),
            create_tool_component(tool_name, tool_config, default_registry),
        );
    }

    spin_config.component = components;

    // Build triggers
    let mut triggers = TriggerConfig {
        http: Vec::new(),
        redis: Vec::new(),
    };

    if ftl_config.is_auth_enabled() {
        // When auth is enabled, all routes go through authorizer
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Path("/...".to_string()),
            component: "mcp".to_string(),
            executor: None,
        });

        // Gateway is private when auth is enabled
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Private { private: true },
            component: "ftl-mcp-gateway".to_string(),
            executor: None,
        });
    } else {
        // When auth is disabled, all routes go directly to gateway (named "mcp")
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Path("/...".to_string()),
            component: "mcp".to_string(),
            executor: None,
        });
    }

    // Add tool endpoints
    for tool_name in ftl_config.tools.keys() {
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Private { private: true },
            component: tool_name.clone(),
            executor: None,
        });
    }

    // Generate the TOML with triggers
    spin_config.to_toml_string_with_triggers(&triggers)
}

/// Create MCP authorizer component configuration
fn create_mcp_component(registry_uri: &str, default_registry: Option<&str>) -> ComponentConfig {
    // Use default if empty
    let uri = if registry_uri.is_empty() {
        "ghcr.io/fastertools/mcp-authorizer:0.0.12"
    } else {
        registry_uri
    };
    let source = parse_component_source(uri, default_registry);

    let allowed_hosts = vec![
        "http://*.spin.internal".to_string(),
        "https://*.authkit.app".to_string(),
        "https://*.workos.com".to_string(),
    ];

    let mut variables = HashMap::new();

    // Core MCP settings
    variables.insert(
        "mcp_gateway_url".to_string(),
        "{{ mcp_gateway_url }}".to_string(),
    );
    variables.insert(
        "mcp_trace_header".to_string(),
        "{{ mcp_trace_header }}".to_string(),
    );
    variables.insert(
        "mcp_provider_type".to_string(),
        "{{ mcp_provider_type }}".to_string(),
    );

    // JWT provider settings
    variables.insert(
        "mcp_jwt_issuer".to_string(),
        "{{ mcp_jwt_issuer }}".to_string(),
    );
    variables.insert(
        "mcp_jwt_audience".to_string(),
        "{{ mcp_jwt_audience }}".to_string(),
    );
    variables.insert(
        "mcp_jwt_jwks_uri".to_string(),
        "{{ mcp_jwt_jwks_uri }}".to_string(),
    );
    variables.insert(
        "mcp_jwt_public_key".to_string(),
        "{{ mcp_jwt_public_key }}".to_string(),
    );
    variables.insert(
        "mcp_jwt_algorithm".to_string(),
        "{{ mcp_jwt_algorithm }}".to_string(),
    );
    variables.insert(
        "mcp_jwt_required_scopes".to_string(),
        "{{ mcp_jwt_required_scopes }}".to_string(),
    );

    // OAuth discovery settings
    variables.insert(
        "mcp_oauth_authorize_endpoint".to_string(),
        "{{ mcp_oauth_authorize_endpoint }}".to_string(),
    );
    variables.insert(
        "mcp_oauth_token_endpoint".to_string(),
        "{{ mcp_oauth_token_endpoint }}".to_string(),
    );
    variables.insert(
        "mcp_oauth_userinfo_endpoint".to_string(),
        "{{ mcp_oauth_userinfo_endpoint }}".to_string(),
    );

    // Static provider settings
    variables.insert(
        "mcp_static_tokens".to_string(),
        "{{ mcp_static_tokens }}".to_string(),
    );

    // Tenant ID for private mode
    variables.insert(
        "mcp_tenant_id".to_string(),
        "{{ mcp_tenant_id }}".to_string(),
    );

    ComponentConfig {
        description: String::new(),
        source,
        files: Vec::new(),
        exclude_files: Vec::new(),
        allowed_outbound_hosts: allowed_hosts,
        key_value_stores: vec!["default".to_string()],
        environment: HashMap::new(),
        build: None,
        variables,
        dependencies_inherit_configuration: false,
        dependencies: HashMap::new(),
    }
}

/// Create gateway component configuration
fn create_gateway_component(
    registry_uri: &str,
    validate_args: bool,
    default_registry: Option<&str>,
) -> ComponentConfig {
    // Use default if empty
    let uri = if registry_uri.is_empty() {
        "ghcr.io/fastertools/mcp-gateway:0.0.10"
    } else {
        registry_uri
    };
    let source = parse_component_source(uri, default_registry);

    let allowed_hosts = vec!["http://*.spin.internal".to_string()];

    let mut variables = HashMap::new();
    variables.insert(
        "tool_components".to_string(),
        "{{ tool_components }}".to_string(),
    );
    variables.insert("validate_arguments".to_string(), validate_args.to_string());

    ComponentConfig {
        description: String::new(),
        source,
        files: Vec::new(),
        exclude_files: Vec::new(),
        allowed_outbound_hosts: allowed_hosts,
        key_value_stores: Vec::new(),
        environment: HashMap::new(),
        build: None,
        variables,
        dependencies_inherit_configuration: false,
        dependencies: HashMap::new(),
    }
}

/// Create tool component configuration
fn create_tool_component(
    name: &str,
    config: &ToolConfig,
    default_registry: Option<&str>,
) -> ComponentConfig {
    // Determine source based on whether it's a local component or registry component
    let source = if let Some(wasm_path) = &config.wasm {
        // Local component with wasm file path
        ComponentSource::Local(wasm_path.clone())
    } else if let Some(repo_ref) = &config.repo {
        // Registry component with repository reference
        parse_component_source(repo_ref, default_registry)
    } else {
        // This shouldn't happen due to validation, but provide a fallback
        ComponentSource::Local(String::from("unknown.wasm"))
    };

    let allowed_hosts = config
        .allowed_outbound_hosts
        .iter()
        .map(String::clone)
        .collect();

    // Build configuration (only if build config exists)
    let build = config.build.as_ref().map(|build_config| {
        let tool_path = config.get_path(name);
        ComponentBuildConfig {
            command: build_config.command.clone(),
            workdir: tool_path,
            watch: build_config.watch.clone(),
            environment: build_config.env.clone(),
        }
    });

    // Variables - pass through as-is (including template references)
    let variables = config.variables.clone();

    ComponentConfig {
        description: String::new(),
        source,
        files: Vec::new(),
        exclude_files: Vec::new(),
        allowed_outbound_hosts: allowed_hosts,
        key_value_stores: Vec::new(),
        environment: HashMap::new(),
        build,
        variables,
        dependencies_inherit_configuration: false,
        dependencies: HashMap::new(),
    }
}

/// Check if ftl.toml exists and transpile it to spin.toml if needed
pub fn ensure_spin_toml(file_system: &Arc<dyn FileSystem>, path: &std::path::Path) -> Result<()> {
    let ftl_toml_path = path.join("ftl.toml");
    let spin_toml_path = path.join("spin.toml");

    // If ftl.toml exists, transpile it
    if file_system.exists(&ftl_toml_path) {
        let ftl_content = file_system
            .read_to_string(&ftl_toml_path)
            .context("Failed to read ftl.toml")?;

        let ftl_config = FtlConfig::parse(&ftl_content)?;
        let spin_content = transpile_ftl_to_spin(&ftl_config)?;

        // Write or update spin.toml
        file_system
            .write_string(&spin_toml_path, &spin_content)
            .context("Failed to write spin.toml")?;
    }

    Ok(())
}

/// Configuration for generating temporary spin.toml
pub struct GenerateSpinConfig<'a> {
    /// File system abstraction
    pub file_system: &'a Arc<dyn FileSystem>,
    /// Path to the project directory
    pub project_path: &'a std::path::Path,
    /// Whether to download registry components
    pub download_components: bool,
    /// Whether to validate auth config for local development
    pub validate_local_auth: bool,
}

/// Generate spin.toml from ftl.toml in a temporary location
/// Returns the path to the temporary spin.toml file, or None if ftl.toml doesn't exist
#[allow(clippy::too_many_lines)]
pub fn generate_temp_spin_toml(config: &GenerateSpinConfig) -> Result<Option<std::path::PathBuf>> {
    let ftl_toml_path = config.project_path.join("ftl.toml");

    // If ftl.toml doesn't exist, return None
    if !config.file_system.exists(&ftl_toml_path) {
        return Ok(None);
    }

    // Read and parse ftl.toml
    let ftl_content = config
        .file_system
        .read_to_string(&ftl_toml_path)
        .context("Failed to read ftl.toml")?;

    let mut ftl_config = FtlConfig::parse(&ftl_content)?;

    // Validate auth configuration for local development
    if config.validate_local_auth
        && ftl_config.project.access_control == "private"
        && ftl_config.oidc.is_none()
    {
        return Err(anyhow::anyhow!(
            "Private access control requires OIDC configuration for local development.\n\
            \n\
            To fix this, either:\n\
            1. Add an [oidc] section to your ftl.toml with your OIDC provider details\n\
            2. Set access_control = \"public\"\n"
        ));
    }

    // Convert all relative paths to absolute paths based on project directory
    let abs_project_path = config
        .project_path
        .canonicalize()
        .unwrap_or_else(|_| config.project_path.to_path_buf());

    // Create a temporary directory for FTL artifacts
    let temp_dir = tempfile::Builder::new()
        .prefix("ftlup-")
        .tempdir()
        .context("Failed to create temporary directory")?;

    // Download MCP components if they're from registries
    let default_registry = ftl_config.project.default_registry.as_deref();

    // Download MCP gateway (skip in test environment or if not downloading)
    // Check for NEXTEST env var (set by nextest) or if we're in cfg(test)
    let is_test = std::env::var("NEXTEST").is_ok() || cfg!(test);
    if !ftl_config.mcp.gateway.to_lowercase().ends_with(".wasm")
        && config.download_components
        && !is_test
    {
        let resolved_url =
            crate::registry::resolve_registry_url(&ftl_config.mcp.gateway, default_registry);
        let wasm_path = temp_dir.path().join("mcp-gateway.wasm");
        eprintln!("Pulling MCP gateway from {resolved_url}...");
        if let Err(e) = crate::registry::pull_component(&resolved_url, &wasm_path.to_string_lossy())
        {
            eprintln!("Error: Failed to pull MCP gateway: {e}");
            eprintln!("Please ensure wkg is installed: cargo install wkg");
            return Err(anyhow::anyhow!("Failed to pull required MCP gateway component"));
        } else {
            ftl_config.mcp.gateway = wasm_path.to_string_lossy().to_string();
        }
    }

    // Download MCP authorizer if auth is enabled (skip in test environment or if not downloading)
    if ftl_config.is_auth_enabled()
        && !ftl_config.mcp.authorizer.to_lowercase().ends_with(".wasm")
        && config.download_components
        && !is_test
    {
        let resolved_url =
            crate::registry::resolve_registry_url(&ftl_config.mcp.authorizer, default_registry);
        let wasm_path = temp_dir.path().join("mcp-authorizer.wasm");
        eprintln!("Pulling MCP authorizer from {resolved_url}...");
        if let Err(e) = crate::registry::pull_component(&resolved_url, &wasm_path.to_string_lossy())
        {
            eprintln!("Error: Failed to pull MCP authorizer: {e}");
            eprintln!("Please ensure wkg is installed: cargo install wkg");
            return Err(anyhow::anyhow!("Failed to pull required MCP authorizer component"));
        } else {
            ftl_config.mcp.authorizer = wasm_path.to_string_lossy().to_string();
        }
    }

    // Process tools and download registry components if needed
    for (tool_name, tool_config) in &mut ftl_config.tools {
        if let Some(repo_ref) = &tool_config.repo {
            // This is a registry component - pull it using wkg (skip in test environment or if not downloading)
            if config.download_components && !is_test {
                let resolved_url =
                    crate::registry::resolve_registry_url(repo_ref, default_registry);

                // Download to temp directory with a descriptive name
                let wasm_filename = format!("{tool_name}.wasm");
                let wasm_path = temp_dir.path().join(&wasm_filename);

                eprintln!("Pulling registry component {tool_name} from {resolved_url}...");
                crate::registry::pull_component(&resolved_url, &wasm_path.to_string_lossy())
                    .with_context(|| {
                        format!("Failed to pull component {tool_name} from {resolved_url}")
                    })?;

                // Update config to use the downloaded WASM file
                tool_config.repo = None;
                tool_config.wasm = Some(wasm_path.to_string_lossy().to_string());
            } else {
                // In test environment, convert to OCI reference
                let resolved_url =
                    crate::registry::resolve_registry_url(repo_ref, default_registry);
                tool_config.repo = None;
                tool_config.wasm = Some(format!("oci://{resolved_url}"));
            }
        } else {
            // Local component - update paths to be absolute
            let tool_path = tool_config.get_path(tool_name);
            if !tool_path.starts_with('/') {
                tool_config.path = Some(
                    abs_project_path
                        .join(&tool_path)
                        .to_string_lossy()
                        .to_string(),
                );
            }

            // Also make the wasm path absolute (only for local components)
            if let Some(wasm_path) = &tool_config.wasm {
                if !wasm_path.starts_with('/') {
                    tool_config.wasm = Some(
                        abs_project_path
                            .join(wasm_path)
                            .to_string_lossy()
                            .to_string(),
                    );
                }
            }
        }
    }

    let spin_content = transpile_ftl_to_spin(&ftl_config)?;

    // Create spin.toml in the temp directory
    let temp_file = temp_dir.path().join("spin.toml");

    // Write spin.toml to temp location
    std::fs::write(&temp_file, &spin_content).context("Failed to write temporary spin.toml")?;

    // Keep the directory alive (it will be cleaned up on process exit)
    let _kept_dir = temp_dir.keep();

    Ok(Some(temp_file))
}

#[cfg(test)]
#[path = "transpiler_tests.rs"]
mod tests;
