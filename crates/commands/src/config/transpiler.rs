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

/// Parse a registry URI into a Spin source configuration
/// Example: "ghcr.io/myorg/my-component:1.0.0" -> `ComponentSource::Registry`
fn parse_registry_uri_to_source(uri: &str) -> ComponentSource {
    // Split by the last colon to separate version tag
    if let Some((image, version)) = uri.rsplit_once(':') {
        // Split the image part by the first slash to get registry and package
        if let Some((registry, package)) = image.split_once('/') {
            // Convert package from "owner/name" format to "owner:name" format for Spin
            let package = package.replace('/', ":");
            return ComponentSource::Registry {
                registry: registry.to_string(),
                package,
                version: version.to_string(),
            };
        }
    }

    // If parsing fails, return as a local path (shouldn't happen with valid URIs)
    ComponentSource::Local(uri.to_string())
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
            default: ftl_config.auth.enabled.to_string(),
        },
    );

    // Only add other auth variables if auth is enabled
    if ftl_config.auth.enabled {
        variables.insert(
            "auth_gateway_url".to_string(),
            SpinVariable::Default {
                default: "http://ftl-mcp-gateway.spin.internal/mcp-internal".to_string(),
            },
        );
        variables.insert(
            "auth_trace_header".to_string(),
            SpinVariable::Default {
                default: "X-Trace-Id".to_string(),
            },
        );

        // Auth provider variables
        variables.insert(
            "auth_provider_type".to_string(),
            SpinVariable::Default {
                default: ftl_config.auth.provider_type().to_string(),
            },
        );
        variables.insert(
            "auth_provider_issuer".to_string(),
            SpinVariable::Default {
                default: ftl_config.auth.issuer().to_string(),
            },
        );
        variables.insert(
            "auth_provider_audience".to_string(),
            SpinVariable::Default {
                default: ftl_config.auth.audience().to_string(),
            },
        );

        // OIDC-specific variables
        if let Some(oidc) = &ftl_config.auth.oidc {
            variables.insert(
                "auth_provider_name".to_string(),
                SpinVariable::Default {
                    default: oidc.provider_name.clone(),
                },
            );
            variables.insert(
                "auth_provider_jwks_uri".to_string(),
                SpinVariable::Default {
                    default: oidc.jwks_uri.clone(),
                },
            );
            variables.insert(
                "auth_provider_authorize_endpoint".to_string(),
                SpinVariable::Default {
                    default: oidc.authorize_endpoint.clone(),
                },
            );
            variables.insert(
                "auth_provider_token_endpoint".to_string(),
                SpinVariable::Default {
                    default: oidc.token_endpoint.clone(),
                },
            );
            variables.insert(
                "auth_provider_userinfo_endpoint".to_string(),
                SpinVariable::Default {
                    default: oidc.userinfo_endpoint.clone(),
                },
            );
            variables.insert(
                "auth_provider_allowed_domains".to_string(),
                SpinVariable::Default {
                    default: oidc.allowed_domains.clone(),
                },
            );
        } else {
            // Set empty defaults for OIDC variables
            let oidc_vars = [
                "auth_provider_name",
                "auth_provider_jwks_uri",
                "auth_provider_authorize_endpoint",
                "auth_provider_token_endpoint",
                "auth_provider_userinfo_endpoint",
                "auth_provider_allowed_domains",
            ];
            for var in &oidc_vars {
                variables.insert(
                    (*var).to_string(),
                    SpinVariable::Default {
                        default: String::new(),
                    },
                );
            }
        }
    }

    spin_config.variables = variables;

    // Build components
    let mut components = HashMap::new();

    if ftl_config.auth.enabled {
        // When auth is enabled, add authorizer as "mcp" and gateway as "ftl-mcp-gateway"
        components.insert(
            "mcp".to_string(),
            create_mcp_component(&ftl_config.mcp.authorizer),
        );
        components.insert(
            "ftl-mcp-gateway".to_string(),
            create_gateway_component(&ftl_config.mcp.gateway, ftl_config.mcp.validate_arguments),
        );
    } else {
        // When auth is disabled, add gateway as "mcp" for consistent route naming
        components.insert(
            "mcp".to_string(),
            create_gateway_component(&ftl_config.mcp.gateway, ftl_config.mcp.validate_arguments),
        );
    }

    // Add tool components
    for (tool_name, tool_config) in &ftl_config.tools {
        components.insert(
            tool_name.clone(),
            create_tool_component(tool_name, tool_config),
        );
    }

    spin_config.component = components;

    // Build triggers
    let mut triggers = TriggerConfig {
        http: Vec::new(),
        redis: Vec::new(),
    };

    if ftl_config.auth.enabled {
        // When auth is enabled, MCP endpoint goes through authorizer
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Path("/mcp".to_string()),
            component: "mcp".to_string(),
            executor: None,
        });

        // Add OAuth endpoints
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Path("/.well-known/oauth-protected-resource".to_string()),
            component: "mcp".to_string(),
            executor: None,
        });
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Path("/.well-known/oauth-authorization-server".to_string()),
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
        // When auth is disabled, MCP endpoint goes directly to gateway (named "mcp")
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Path("/mcp".to_string()),
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
fn create_mcp_component(registry_uri: &str) -> ComponentConfig {
    let source = parse_registry_uri_to_source(registry_uri);

    let allowed_hosts = vec![
        "http://*.spin.internal".to_string(),
        "https://*.authkit.app".to_string(),
    ];

    let mut variables = HashMap::new();
    variables.insert("auth_enabled".to_string(), "{{ auth_enabled }}".to_string());
    variables.insert(
        "auth_gateway_url".to_string(),
        "{{ auth_gateway_url }}".to_string(),
    );
    variables.insert(
        "auth_trace_header".to_string(),
        "{{ auth_trace_header }}".to_string(),
    );
    variables.insert(
        "auth_provider_type".to_string(),
        "{{ auth_provider_type }}".to_string(),
    );
    variables.insert(
        "auth_provider_issuer".to_string(),
        "{{ auth_provider_issuer }}".to_string(),
    );
    variables.insert(
        "auth_provider_audience".to_string(),
        "{{ auth_provider_audience }}".to_string(),
    );
    variables.insert(
        "auth_provider_name".to_string(),
        "{{ auth_provider_name }}".to_string(),
    );
    variables.insert(
        "auth_provider_jwks_uri".to_string(),
        "{{ auth_provider_jwks_uri }}".to_string(),
    );
    variables.insert(
        "auth_provider_authorize_endpoint".to_string(),
        "{{ auth_provider_authorize_endpoint }}".to_string(),
    );
    variables.insert(
        "auth_provider_token_endpoint".to_string(),
        "{{ auth_provider_token_endpoint }}".to_string(),
    );
    variables.insert(
        "auth_provider_userinfo_endpoint".to_string(),
        "{{ auth_provider_userinfo_endpoint }}".to_string(),
    );
    variables.insert(
        "auth_provider_allowed_domains".to_string(),
        "{{ auth_provider_allowed_domains }}".to_string(),
    );

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

/// Create gateway component configuration
fn create_gateway_component(registry_uri: &str, validate_args: bool) -> ComponentConfig {
    let source = parse_registry_uri_to_source(registry_uri);

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
fn create_tool_component(name: &str, config: &ToolConfig) -> ComponentConfig {
    let source = ComponentSource::Local(config.wasm.clone());

    let allowed_hosts = config
        .allowed_outbound_hosts
        .iter()
        .map(String::clone)
        .collect();

    // Build configuration
    let tool_path = config.get_path(name);
    let build = Some(ComponentBuildConfig {
        command: config.build.command.clone(),
        workdir: tool_path,
        watch: config.build.watch.clone(),
        environment: config.build.env.clone(),
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

/// Generate spin.toml from ftl.toml in a temporary location
/// Returns the path to the temporary spin.toml file, or None if ftl.toml doesn't exist
pub fn generate_temp_spin_toml(
    file_system: &Arc<dyn FileSystem>,
    project_path: &std::path::Path,
) -> Result<Option<std::path::PathBuf>> {
    let ftl_toml_path = project_path.join("ftl.toml");

    // If ftl.toml doesn't exist, return None
    if !file_system.exists(&ftl_toml_path) {
        return Ok(None);
    }

    // Read and parse ftl.toml
    let ftl_content = file_system
        .read_to_string(&ftl_toml_path)
        .context("Failed to read ftl.toml")?;

    let mut ftl_config = FtlConfig::parse(&ftl_content)?;

    // Convert all relative paths to absolute paths based on project directory
    let abs_project_path = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());

    // Update tool paths to be absolute
    for (tool_name, tool_config) in &mut ftl_config.tools {
        let tool_path = tool_config.get_path(tool_name);
        if !tool_path.starts_with('/') {
            tool_config.path = Some(
                abs_project_path
                    .join(&tool_path)
                    .to_string_lossy()
                    .to_string(),
            );
        }

        // Also make the wasm path absolute
        if !tool_config.wasm.starts_with('/') {
            tool_config.wasm = abs_project_path
                .join(&tool_config.wasm)
                .to_string_lossy()
                .to_string();
        }
    }

    let spin_content = transpile_ftl_to_spin(&ftl_config)?;

    // Create a temporary directory for FTL artifacts
    let temp_dir = tempfile::Builder::new()
        .prefix("ftlup-")
        .tempdir()
        .context("Failed to create temporary directory")?;

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
