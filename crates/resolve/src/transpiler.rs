//! Pure transpiler for converting ftl.toml to spin.toml format
//!
//! This module provides pure functions for configuration transpilation.
//! No file I/O, no downloads - just data transformation.

use crate::ftl_resolve::{ApplicationVariable, ComponentConfig, FtlConfig};
use crate::spin_config::{
    ComponentBuildConfig, ComponentConfig as SpinComponentConfig, ComponentSource, HttpTrigger,
    RouteConfig, SpinConfig, SpinVariable, TriggerConfig,
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Transpile an FTL configuration to Spin configuration format
///
/// This is a pure function that converts between configuration formats.
/// Component paths are preserved as-is from the input configuration.
#[allow(clippy::too_many_lines)]
pub fn transpile_ftl_to_spin(ftl_resolve: &FtlConfig) -> Result<String> {
    let mut spin_config = SpinConfig::new(ftl_resolve.project.name.clone());

    // Set application metadata
    spin_config
        .application
        .version
        .clone_from(&ftl_resolve.project.version);
    spin_config
        .application
        .description
        .clone_from(&ftl_resolve.project.description);
    spin_config
        .application
        .authors
        .clone_from(&ftl_resolve.project.authors);

    // Build variables
    let mut variables = HashMap::new();

    // Add application-level variables
    for (name, var) in &ftl_resolve.variables {
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
    let component_names = ftl_resolve.component_names().join(",");
    variables.insert(
        "component_names".to_string(),
        SpinVariable::Default {
            default: component_names,
        },
    );

    // Always include auth_enabled variable
    variables.insert(
        "auth_enabled".to_string(),
        SpinVariable::Default {
            default: ftl_resolve.is_auth_enabled().to_string(),
        },
    );

    // Add core MCP variables (always needed)
    add_core_mcp_variables(&mut variables, ftl_resolve);

    // Add authorization rules variables (always needed for platform integration)
    add_ownership_variables(&mut variables, ftl_resolve);

    // Only add auth provider variables if auth is enabled
    // When auth is disabled, we don't set provider variables so the authorizer
    // knows no provider is configured
    if ftl_resolve.is_auth_enabled() {
        add_jwt_variables(&mut variables, ftl_resolve);
        add_oauth_variables(&mut variables, ftl_resolve);
    } else {
        // Explicitly set empty provider variables when auth is disabled
        // This ensures the authorizer knows no provider is configured
        for var in &[
            "mcp_jwt_issuer",
            "mcp_jwt_audience",
            "mcp_jwt_jwks_uri",
            "mcp_jwt_public_key",
            "mcp_jwt_algorithm",
            "mcp_jwt_required_scopes",
            "mcp_oauth_authorize_endpoint",
            "mcp_oauth_token_endpoint",
            "mcp_oauth_userinfo_endpoint",
            "mcp_provider_type",
        ] {
            variables.insert(
                (*var).to_string(),
                SpinVariable::Default {
                    default: String::new(),
                },
            );
        }
    }

    spin_config.variables = variables;

    // Build components
    let mut components = HashMap::new();
    let default_registry = ftl_resolve.project.default_registry.as_deref();

    if ftl_resolve.is_auth_enabled() {
        // When auth is enabled, add authorizer as "mcp" and gateway as "ftl-mcp-gateway"
        components.insert(
            "mcp".to_string(),
            create_mcp_component(&ftl_resolve.mcp.authorizer, default_registry),
        );
        components.insert(
            "ftl-mcp-gateway".to_string(),
            create_gateway_component(
                &ftl_resolve.mcp.gateway,
                ftl_resolve.mcp.validate_arguments,
                default_registry,
            ),
        );
    } else {
        // When auth is disabled, add gateway as "mcp" for consistent route naming
        components.insert(
            "mcp".to_string(),
            create_gateway_component(
                &ftl_resolve.mcp.gateway,
                ftl_resolve.mcp.validate_arguments,
                default_registry,
            ),
        );
    }

    // Add user components
    for (component_name, component_config) in &ftl_resolve.component {
        components.insert(
            component_name.clone(),
            create_user_component(component_name, component_config, default_registry),
        );
    }

    spin_config.component = components;

    // Build triggers
    let triggers = build_triggers(ftl_resolve);

    // Generate the TOML with triggers
    spin_config.to_toml_string_with_triggers(&triggers)
}

/// Create spin.toml with resolved component paths
///
/// Takes an FTL config and mappings from `ComponentResolver`,
/// returns the transpiled spin.toml content with local paths.
pub fn create_spin_toml_with_resolved_paths<S: std::hash::BuildHasher>(
    ftl_resolve: &FtlConfig,
    resolved_mappings: &HashMap<String, PathBuf, S>,
    project_path: &Path,
) -> Result<String> {
    let mut modified_config = ftl_resolve.clone();
    let abs_project_path = project_path
        .canonicalize()
        .unwrap_or_else(|_| project_path.to_path_buf());

    // Update MCP component paths if resolved
    if let Some(gateway_path) = resolved_mappings.get("mcp-gateway") {
        modified_config.mcp.gateway = gateway_path.to_string_lossy().to_string();
    } else {
        make_path_absolute(&mut modified_config.mcp.gateway, &abs_project_path);
    }

    if let Some(authorizer_path) = resolved_mappings.get("mcp-authorizer") {
        modified_config.mcp.authorizer = authorizer_path.to_string_lossy().to_string();
    } else {
        make_path_absolute(&mut modified_config.mcp.authorizer, &abs_project_path);
    }

    // Update user component paths
    for (component_name, component_config) in &mut modified_config.component {
        if let Some(resolved_path) = resolved_mappings.get(component_name) {
            // Registry component was resolved to local path
            component_config.repo = None;
            component_config.wasm = Some(resolved_path.to_string_lossy().to_string());
        } else if let Some(wasm_path) = &component_config.wasm {
            // Local component - make path absolute
            let mut abs_wasm = wasm_path.clone();
            make_path_absolute(&mut abs_wasm, &abs_project_path);
            component_config.wasm = Some(abs_wasm);
        }

        // Update component directory path if present
        if let Some(path) = &component_config.path {
            let mut abs_path = path.clone();
            make_path_absolute(&mut abs_path, &abs_project_path);
            component_config.path = Some(abs_path);
        }
    }

    transpile_ftl_to_spin(&modified_config)
}

/// Validate auth configuration for local development
pub const fn validate_local_auth(_ftl_config: &FtlConfig) -> Result<()> {
    // Local development only supports:
    // - No [oauth] block = public access
    // - With [oauth] block = custom OAuth
    // Private and org modes are only available via ftl eng deploy
    Ok(())
}

// Helper functions

fn make_path_absolute(path: &mut String, base: &Path) {
    // Skip if already absolute or is a registry reference
    if path.starts_with('/')
        || path.contains("://")
        || (path.contains('/')
            && !std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("wasm")))
    {
        return;
    }

    // Only make local WASM files absolute
    if std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("wasm"))
    {
        *path = base.join(&*path).to_string_lossy().to_string();
    }
}

fn add_ownership_variables(variables: &mut HashMap<String, SpinVariable>, ftl_resolve: &FtlConfig) {
    // Authorization rules will be populated by the platform during deployment
    // based on the deployment configuration (org-scoped, user-scoped, etc.)
    // Note: mcp_auth_enabled is no longer used - the authorizer determines auth
    // based on whether a provider is configured

    // Get allowed_subjects from OAuth config if present
    let allowed_subjects = ftl_resolve
        .oauth
        .as_ref()
        .map(|oauth| oauth.allowed_subjects.join(","))
        .unwrap_or_default();

    variables.insert(
        "mcp_auth_allowed_subjects".to_string(),
        SpinVariable::Default {
            default: allowed_subjects,
        },
    );

    variables.insert(
        "mcp_auth_required_claims".to_string(),
        SpinVariable::Default {
            default: String::new(),
        },
    );

    variables.insert(
        "mcp_auth_forward_claims".to_string(),
        SpinVariable::Default {
            default: String::new(),
        },
    );
}

fn add_core_mcp_variables(variables: &mut HashMap<String, SpinVariable>, ftl_resolve: &FtlConfig) {
    // Gateway URL only needed when auth is enabled (points to the actual gateway)
    // When auth is disabled, the gateway is accessed directly
    let gateway_url = if ftl_resolve.is_auth_enabled() {
        "http://ftl-mcp-gateway.spin.internal".to_string()
    } else {
        // No separate authorizer, so no forwarding needed
        "none".to_string()
    };
    variables.insert(
        "mcp_gateway_url".to_string(),
        SpinVariable::Default {
            default: gateway_url,
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
            default: ftl_resolve.auth_provider_type().to_string(),
        },
    );
}

fn add_jwt_variables(variables: &mut HashMap<String, SpinVariable>, ftl_resolve: &FtlConfig) {
    variables.insert(
        "mcp_jwt_issuer".to_string(),
        SpinVariable::Default {
            default: ftl_resolve.auth_issuer().to_string(),
        },
    );
    variables.insert(
        "mcp_jwt_audience".to_string(),
        SpinVariable::Default {
            default: ftl_resolve.auth_audience().to_string(),
        },
    );
    variables.insert(
        "mcp_jwt_required_scopes".to_string(),
        SpinVariable::Default {
            default: ftl_resolve.auth_required_scopes().to_string(),
        },
    );

    let jwks_uri = ftl_resolve
        .oauth
        .as_ref()
        .map(|o| o.jwks_uri.clone())
        .unwrap_or_default();
    variables.insert(
        "mcp_jwt_jwks_uri".to_string(),
        SpinVariable::Default { default: jwks_uri },
    );
}

fn add_oauth_variables(variables: &mut HashMap<String, SpinVariable>, ftl_resolve: &FtlConfig) {
    if let Some(oauth) = &ftl_resolve.oauth {
        variables.insert(
            "mcp_jwt_public_key".to_string(),
            SpinVariable::Default {
                default: oauth.public_key.clone(),
            },
        );
        variables.insert(
            "mcp_jwt_algorithm".to_string(),
            SpinVariable::Default {
                default: oauth.algorithm.clone(),
            },
        );
        variables.insert(
            "mcp_oauth_authorize_endpoint".to_string(),
            SpinVariable::Default {
                default: oauth.authorize_endpoint.clone(),
            },
        );
        variables.insert(
            "mcp_oauth_token_endpoint".to_string(),
            SpinVariable::Default {
                default: oauth.token_endpoint.clone(),
            },
        );
        variables.insert(
            "mcp_oauth_userinfo_endpoint".to_string(),
            SpinVariable::Default {
                default: oauth.userinfo_endpoint.clone(),
            },
        );
    } else {
        for var in &[
            "mcp_jwt_public_key",
            "mcp_jwt_algorithm",
            "mcp_oauth_authorize_endpoint",
            "mcp_oauth_token_endpoint",
            "mcp_oauth_userinfo_endpoint",
        ] {
            variables.insert(
                (*var).to_string(),
                SpinVariable::Default {
                    default: String::new(),
                },
            );
        }
    }
}

fn build_triggers(ftl_resolve: &FtlConfig) -> TriggerConfig {
    let mut triggers = TriggerConfig {
        http: Vec::new(),
        redis: Vec::new(),
    };

    if ftl_resolve.is_auth_enabled() {
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

    // Add component endpoints
    for component_name in ftl_resolve.component.keys() {
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Private { private: true },
            component: component_name.clone(),
            executor: None,
        });
    }

    triggers
}

fn parse_component_source(source: &str, _default_registry: Option<&str>) -> ComponentSource {
    // Check if this looks like a registry reference (contains registry domain and colon)
    // Registry references are in format: registry.domain/namespace/package:version
    // or registry.domain/package:version
    if source.contains('/') && source.contains(':') {
        // Try to parse as a registry reference
        // Find the last colon to separate version
        if let Some(version_sep) = source.rfind(':') {
            let package_part = &source[..version_sep];
            let version = &source[version_sep + 1..];

            // Find the first slash to separate registry from package
            if let Some(registry_sep) = package_part.find('/') {
                let registry = &package_part[..registry_sep];
                let package_name = &package_part[registry_sep + 1..];

                // For OCI registries, we keep the full package path
                return ComponentSource::Registry {
                    registry: registry.to_string(),
                    package: package_name.to_string(),
                    version: version.to_string(),
                };
            }
        }
    }

    // Otherwise treat as a local path
    ComponentSource::Local(source.to_string())
}

fn create_mcp_component(registry_uri: &str, default_registry: Option<&str>) -> SpinComponentConfig {
    let uri = if registry_uri.is_empty() {
        "ghcr.io/fastertools/mcp-authorizer:0.0.14"
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

    // MCP authorizer variables use template syntax
    // Note: mcp_auth_enabled is no longer used - the authorizer determines auth
    // based on whether a provider is configured
    for var in &[
        "mcp_gateway_url",
        "mcp_trace_header",
        "mcp_provider_type",
        "mcp_jwt_issuer",
        "mcp_jwt_audience",
        "mcp_jwt_jwks_uri",
        "mcp_jwt_public_key",
        "mcp_jwt_algorithm",
        "mcp_jwt_required_scopes",
        "mcp_oauth_authorize_endpoint",
        "mcp_oauth_token_endpoint",
        "mcp_oauth_userinfo_endpoint",
        "mcp_auth_allowed_subjects",
        "mcp_auth_required_claims",
        "mcp_auth_forward_claims",
    ] {
        variables.insert((*var).to_string(), format!("{{{{ {var} }}}}"));
    }

    SpinComponentConfig {
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

fn create_gateway_component(
    registry_uri: &str,
    validate_args: bool,
    default_registry: Option<&str>,
) -> SpinComponentConfig {
    let uri = if registry_uri.is_empty() {
        "ghcr.io/fastertools/mcp-gateway:0.0.11"
    } else {
        registry_uri
    };
    let source = parse_component_source(uri, default_registry);

    let allowed_hosts = vec!["http://*.spin.internal".to_string()];

    let mut variables = HashMap::new();
    variables.insert(
        "component_names".to_string(),
        "{{ component_names }}".to_string(),
    );
    variables.insert("validate_arguments".to_string(), validate_args.to_string());

    SpinComponentConfig {
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

fn create_user_component(
    name: &str,
    config: &ComponentConfig,
    default_registry: Option<&str>,
) -> SpinComponentConfig {
    let source = if let Some(wasm_path) = &config.wasm {
        ComponentSource::Local(wasm_path.clone())
    } else if let Some(repo_ref) = &config.repo {
        parse_component_source(repo_ref, default_registry)
    } else {
        ComponentSource::Local(String::from("unknown.wasm"))
    };

    let allowed_hosts = config
        .allowed_outbound_hosts
        .iter()
        .map(String::clone)
        .collect();

    let build = config.build.as_ref().map(|build_config| {
        let component_path = config.get_path(name);
        ComponentBuildConfig {
            command: build_config.command.clone(),
            workdir: component_path,
            watch: build_config.watch.clone(),
            environment: build_config.env.clone(),
        }
    });

    let variables = config.variables.clone();

    SpinComponentConfig {
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

#[cfg(test)]
#[path = "transpiler_tests.rs"]
mod tests;
