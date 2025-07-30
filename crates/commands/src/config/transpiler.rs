//! Transpiler for converting ftl.toml to spin.toml format

use super::ftl_config::{FtlConfig, ToolConfig};
use anyhow::{Context, Result};
use ftl_runtime::deps::FileSystem;
use std::fmt::Write;
use std::sync::Arc;
use toml_edit::{Array, ArrayOfTables, InlineTable, Item, Table, Value};

/// Default Spin manifest version
const SPIN_MANIFEST_VERSION: i64 = 2;

/// Transpile an FTL configuration to Spin TOML format
#[allow(clippy::too_many_lines)]
pub fn transpile_ftl_to_spin(ftl_config: &FtlConfig) -> Result<String> {
    let mut doc = toml_edit::DocumentMut::new();

    // Set manifest version
    doc["spin_manifest_version"] = toml_edit::value(SPIN_MANIFEST_VERSION);

    // Build application section
    let mut app_table = Table::new();
    app_table["name"] = toml_edit::value(&ftl_config.project.name);
    app_table["version"] = toml_edit::value(&ftl_config.project.version);

    if !ftl_config.project.authors.is_empty() {
        let mut authors = Array::new();
        for author in &ftl_config.project.authors {
            authors.push(author.as_str());
        }
        app_table["authors"] = toml_edit::value(authors);
    }

    if !ftl_config.project.description.is_empty() {
        app_table["description"] = toml_edit::value(&ftl_config.project.description);
    }

    doc["application"] = Item::Table(app_table);

    // Build variables section
    let mut vars_table = Table::new();

    // Add application-level variables first
    for (name, var) in &ftl_config.variables {
        match var {
            crate::config::ftl_config::ApplicationVariable::Default { default } => {
                vars_table[name] = create_default_var(default);
            }
            crate::config::ftl_config::ApplicationVariable::Required { required: _ } => {
                // For required variables, we create an empty default
                // Spin will enforce the requirement at runtime
                vars_table[name] = create_required_var();
            }
        }
    }

    // Tool components variable
    let tool_names = ftl_config.tool_components().join(",");
    vars_table["tool_components"] = create_default_var(&tool_names);

    // Auth variables
    vars_table["auth_enabled"] = create_default_var(&ftl_config.auth.enabled.to_string());
    vars_table["auth_gateway_url"] =
        create_default_var("http://ftl-mcp-gateway.spin.internal/mcp-internal");
    vars_table["auth_trace_header"] = create_default_var("X-Trace-Id");

    // Auth provider variables
    vars_table["auth_provider_type"] = create_default_var(&ftl_config.auth.provider);
    vars_table["auth_provider_issuer"] = create_default_var(&ftl_config.auth.issuer);
    vars_table["auth_provider_audience"] = create_default_var(&ftl_config.auth.audience);

    // OIDC-specific variables
    if let Some(oidc) = &ftl_config.auth.oidc {
        vars_table["auth_provider_name"] = create_default_var(&oidc.provider_name);
        vars_table["auth_provider_jwks_uri"] = create_default_var(&oidc.jwks_uri);
        vars_table["auth_provider_authorize_endpoint"] =
            create_default_var(&oidc.authorize_endpoint);
        vars_table["auth_provider_token_endpoint"] = create_default_var(&oidc.token_endpoint);
        vars_table["auth_provider_userinfo_endpoint"] = create_default_var(&oidc.userinfo_endpoint);
        vars_table["auth_provider_allowed_domains"] = create_default_var(&oidc.allowed_domains);
    } else {
        // Set empty defaults for OIDC variables
        vars_table["auth_provider_name"] = create_default_var("");
        vars_table["auth_provider_jwks_uri"] = create_default_var("");
        vars_table["auth_provider_authorize_endpoint"] = create_default_var("");
        vars_table["auth_provider_token_endpoint"] = create_default_var("");
        vars_table["auth_provider_userinfo_endpoint"] = create_default_var("");
        vars_table["auth_provider_allowed_domains"] = create_default_var("");
    }

    doc["variables"] = Item::Table(vars_table);

    // Build HTTP triggers and components
    let mut http_triggers = ArrayOfTables::new();
    let mut components = Table::new();

    // Add main MCP endpoint
    http_triggers.push(create_http_trigger("/mcp", "mcp", false));
    http_triggers.push(create_http_trigger(
        "/.well-known/oauth-protected-resource",
        "mcp",
        false,
    ));
    http_triggers.push(create_http_trigger(
        "/.well-known/oauth-authorization-server",
        "mcp",
        false,
    ));

    // Add MCP authorizer component
    let authorizer_version = if ftl_config.gateway.authorizer_version.is_empty() {
        "0.0.9"
    } else {
        &ftl_config.gateway.authorizer_version
    };
    components["mcp"] = Item::Table(create_mcp_component(authorizer_version));

    // Add gateway endpoint and component
    http_triggers.push(create_http_trigger("", "ftl-mcp-gateway", true));
    let gateway_version = if ftl_config.gateway.version.is_empty() {
        "0.0.9"
    } else {
        &ftl_config.gateway.version
    };
    components["ftl-mcp-gateway"] = Item::Table(create_gateway_component(
        gateway_version,
        ftl_config.gateway.validate_arguments,
    ));

    // Add tool components
    for (tool_name, tool_config) in &ftl_config.tools {
        http_triggers.push(create_http_trigger("", tool_name, true));
        components[tool_name] = Item::Table(create_tool_component(tool_name, tool_config));
    }

    // Add components to document
    doc["component"] = Item::Table(components);

    // For triggers, we need to manually build the TOML to get [[trigger.http]] format
    let mut toml_string = doc.to_string();

    // Add the trigger.http array manually
    toml_string.push('\n');
    for trigger in &http_triggers {
        toml_string.push_str("[[trigger.http]]\n");
        for (key, value) in trigger {
            let value_str = match value {
                Item::Value(v) => v.to_string(),
                _ => value.to_string(),
            };
            let _ = writeln!(&mut toml_string, "{key} = {value_str}");
        }
        toml_string.push('\n');
    }

    Ok(toml_string)
}

/// Create a variable with default value
fn create_default_var(value: &str) -> Item {
    let mut inline_table = InlineTable::new();
    inline_table.insert("default", Value::from(value));
    Item::Value(Value::InlineTable(inline_table))
}

/// Create a required variable (no default value)
fn create_required_var() -> Item {
    let mut inline_table = InlineTable::new();
    inline_table.insert("required", Value::from(true));
    Item::Value(Value::InlineTable(inline_table))
}

/// Create an HTTP trigger configuration
fn create_http_trigger(route: &str, component: &str, private: bool) -> Table {
    let mut trigger = Table::new();
    trigger.set_implicit(true);

    if private {
        let mut route_table = InlineTable::new();
        route_table.insert("private", Value::from(true));
        trigger["route"] = Item::Value(Value::InlineTable(route_table));
    } else if !route.is_empty() {
        trigger["route"] = toml_edit::value(route);
    } else {
        let mut route_table = InlineTable::new();
        route_table.insert("private", Value::from(true));
        trigger["route"] = Item::Value(Value::InlineTable(route_table));
    }

    trigger["component"] = toml_edit::value(component);

    trigger
}

/// Create MCP authorizer component configuration
fn create_mcp_component(version: &str) -> Table {
    let mut component = Table::new();

    // Source configuration
    let mut source = InlineTable::new();
    source.insert("registry", Value::from("ghcr.io"));
    source.insert("package", Value::from("fastertools:mcp-authorizer"));
    source.insert("version", Value::from(version));
    component["source"] = Item::Value(Value::InlineTable(source));

    // Allowed hosts
    let mut hosts = Array::new();
    hosts.push("http://*.spin.internal");
    hosts.push("https://*.authkit.app");
    component["allowed_outbound_hosts"] = toml_edit::value(hosts);

    // Variables
    let mut vars = Table::new();
    vars["auth_enabled"] = toml_edit::value("{{ auth_enabled }}");
    vars["auth_gateway_url"] = toml_edit::value("{{ auth_gateway_url }}");
    vars["auth_trace_header"] = toml_edit::value("{{ auth_trace_header }}");
    vars["auth_provider_type"] = toml_edit::value("{{ auth_provider_type }}");
    vars["auth_provider_issuer"] = toml_edit::value("{{ auth_provider_issuer }}");
    vars["auth_provider_audience"] = toml_edit::value("{{ auth_provider_audience }}");
    vars["auth_provider_name"] = toml_edit::value("{{ auth_provider_name }}");
    vars["auth_provider_jwks_uri"] = toml_edit::value("{{ auth_provider_jwks_uri }}");
    vars["auth_provider_authorize_endpoint"] =
        toml_edit::value("{{ auth_provider_authorize_endpoint }}");
    vars["auth_provider_token_endpoint"] = toml_edit::value("{{ auth_provider_token_endpoint }}");
    vars["auth_provider_userinfo_endpoint"] =
        toml_edit::value("{{ auth_provider_userinfo_endpoint }}");
    vars["auth_provider_allowed_domains"] = toml_edit::value("{{ auth_provider_allowed_domains }}");
    component["variables"] = Item::Table(vars);

    component
}

/// Create gateway component configuration
fn create_gateway_component(version: &str, validate_args: bool) -> Table {
    let mut component = Table::new();

    // Source configuration
    let mut source = InlineTable::new();
    source.insert("registry", Value::from("ghcr.io"));
    source.insert("package", Value::from("fastertools:mcp-gateway"));
    source.insert("version", Value::from(version));
    component["source"] = Item::Value(Value::InlineTable(source));

    // Allowed hosts
    let mut hosts = Array::new();
    hosts.push("http://*.spin.internal");
    component["allowed_outbound_hosts"] = toml_edit::value(hosts);

    // Variables
    let mut vars = Table::new();
    vars["tool_components"] = toml_edit::value("{{ tool_components }}");
    vars["validate_arguments"] = toml_edit::value(validate_args.to_string());
    component["variables"] = Item::Table(vars);

    component
}

/// Create tool component configuration
fn create_tool_component(name: &str, config: &ToolConfig) -> Table {
    let mut component = Table::new();

    // Determine source path based on build command
    let source_path = if config.build.command.contains("cargo") {
        // Rust project - output goes to target/wasm32-wasip1/release/
        let tool_basename = std::path::Path::new(&config.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name);
        format!(
            "{}/target/wasm32-wasip1/release/{}.wasm",
            config.path,
            tool_basename.replace('-', "_")
        )
    } else if config.build.command.contains("npm") || config.build.command.contains("node") {
        // Node.js project - output typically goes to dist/
        format!("{}/dist/{}.wasm", config.path, name)
    } else {
        // Unknown build system - assume output is in the tool directory
        format!("{}/{}.wasm", config.path, name)
    };

    component["source"] = toml_edit::value(source_path);

    // Allowed outbound hosts
    let hosts = config
        .allowed_outbound_hosts
        .iter()
        .map(std::string::String::as_str)
        .collect::<Array>();
    component["allowed_outbound_hosts"] = toml_edit::value(hosts);

    // Build configuration - always present now
    let mut build_table = Table::new();

    // Build command
    build_table["command"] = toml_edit::value(&config.build.command);

    // Working directory - always use tool path
    build_table["workdir"] = toml_edit::value(&config.path);

    // Watch paths
    if !config.build.watch.is_empty() {
        let watch_array: Array = config.build.watch.iter().map(String::as_str).collect();
        build_table["watch"] = toml_edit::value(watch_array);
    }

    // Environment variables
    if !config.build.env.is_empty() {
        let mut env_table = Table::new();
        for (key, value) in &config.build.env {
            env_table[key] = toml_edit::value(value);
        }
        build_table["environment"] = Item::Table(env_table);
    }

    component["build"] = Item::Table(build_table);

    // Variables
    if !config.variables.is_empty() {
        let mut vars_table = Table::new();
        for (key, value) in &config.variables {
            // Check if the value is a template reference (e.g., {{ api_token }})
            if value.starts_with("{{") && value.ends_with("}}") {
                // It's a template reference, pass it through as-is
                vars_table[key] = toml_edit::value(value);
            } else {
                // It's a static value, keep as-is
                vars_table[key] = toml_edit::value(value);
            }
        }
        component["variables"] = Item::Table(vars_table);
    }

    component
}

/// Check if ftl.toml exists and transpile it to spin.toml if needed
/// This function is kept for backward compatibility but now writes to project directory
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
    for tool_config in ftl_config.tools.values_mut() {
        if !tool_config.path.starts_with('/') {
            tool_config.path = abs_project_path
                .join(&tool_config.path)
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
