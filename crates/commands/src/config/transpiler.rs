//! Pure transpiler for converting ftl.toml to spin.toml format
//!
//! This module provides pure functions for configuration transpilation.
//! No file I/O, no downloads - just data transformation.

use super::ftl_config::{ApplicationVariable, ComponentConfig, FtlConfig};
use super::spin_config::{
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
pub fn transpile_ftl_to_spin(ftl_config: &FtlConfig) -> Result<String> {
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
    let component_names = ftl_config.component_names().join(",");
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
            default: ftl_config.is_auth_enabled().to_string(),
        },
    );

    // Only add other auth variables if auth is enabled
    if ftl_config.is_auth_enabled() {
        add_auth_variables(&mut variables, ftl_config);
    }

    spin_config.variables = variables;

    // Build components
    let mut components = HashMap::new();
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

    // Add user components
    for (component_name, component_config) in &ftl_config.component {
        components.insert(
            component_name.clone(),
            create_user_component(component_name, component_config, default_registry),
        );
    }

    spin_config.component = components;

    // Build triggers
    let triggers = build_triggers(ftl_config);

    // Generate the TOML with triggers
    spin_config.to_toml_string_with_triggers(&triggers)
}

/// Create spin.toml with resolved component paths
///
/// Takes an FTL config and mappings from `ComponentResolver`,
/// returns the transpiled spin.toml content with local paths.
pub fn create_spin_toml_with_resolved_paths<S: std::hash::BuildHasher>(
    ftl_config: &FtlConfig,
    resolved_mappings: &HashMap<String, PathBuf, S>,
    project_path: &Path,
) -> Result<String> {
    let mut modified_config = ftl_config.clone();
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
pub fn validate_local_auth(ftl_config: &FtlConfig) -> Result<()> {
    if ftl_config.project.access_control == "private" && ftl_config.oauth.is_none() {
        return Err(anyhow::anyhow!(
            "Private access control requires OAuth configuration for local development.\n\
            \n\
            To fix this, either:\n\
            1. Add an [oauth] section to your ftl.toml with your OAuth provider details\n\
            2. Set access_control = \"public\"\n"
        ));
    }
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

fn add_auth_variables(variables: &mut HashMap<String, SpinVariable>, ftl_config: &FtlConfig) {
    add_tenant_variable(variables, ftl_config);
    add_core_mcp_variables(variables, ftl_config);
    add_jwt_variables(variables, ftl_config);
    add_oauth_variables(variables, ftl_config);

    // Static provider variables (legacy)
    variables.insert(
        "mcp_static_tokens".to_string(),
        SpinVariable::Default {
            default: String::new(),
        },
    );
}

fn add_tenant_variable(variables: &mut HashMap<String, SpinVariable>, ftl_config: &FtlConfig) {
    if ftl_config.project.access_control == "private" && ftl_config.oauth.is_none() {
        variables.insert(
            "mcp_tenant_id".to_string(),
            SpinVariable::Required { required: true },
        );
    } else {
        variables.insert(
            "mcp_tenant_id".to_string(),
            SpinVariable::Default {
                default: String::new(),
            },
        );
    }
}

fn add_core_mcp_variables(variables: &mut HashMap<String, SpinVariable>, ftl_config: &FtlConfig) {
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
}

fn add_jwt_variables(variables: &mut HashMap<String, SpinVariable>, ftl_config: &FtlConfig) {
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

    let jwks_uri = ftl_config
        .oauth
        .as_ref()
        .map(|o| o.jwks_uri.clone())
        .unwrap_or_default();
    variables.insert(
        "mcp_jwt_jwks_uri".to_string(),
        SpinVariable::Default { default: jwks_uri },
    );
}

fn add_oauth_variables(variables: &mut HashMap<String, SpinVariable>, ftl_config: &FtlConfig) {
    if let Some(oauth) = &ftl_config.oauth {
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

fn build_triggers(ftl_config: &FtlConfig) -> TriggerConfig {
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

    // Add component endpoints
    for component_name in ftl_config.component.keys() {
        triggers.http.push(HttpTrigger {
            route: RouteConfig::Private { private: true },
            component: component_name.clone(),
            executor: None,
        });
    }

    triggers
}

fn parse_component_source(source: &str, _default_registry: Option<&str>) -> ComponentSource {
    // Always return local source - registry resolution happens elsewhere
    ComponentSource::Local(source.to_string())
}

fn create_mcp_component(registry_uri: &str, default_registry: Option<&str>) -> SpinComponentConfig {
    let uri = if registry_uri.is_empty() {
        "ghcr.io/fastertools/mcp-authorizer:0.0.13"
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
        "mcp_static_tokens",
        "mcp_tenant_id",
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
