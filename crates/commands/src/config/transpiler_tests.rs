//! Comprehensive tests for ftl.toml to spin.toml transpiler
//!
//! These tests ensure complete accuracy and robustness of the transpilation
//! between type-safe FTL and Spin configurations.

use super::*;
use crate::config::ftl_config::*;
use crate::config::spin_config::SpinConfig;
use std::collections::HashMap;

/// Helper function to parse generated spin.toml and validate it
fn validate_spin_toml(spin_toml: &str) -> Result<SpinConfig> {
    SpinConfig::parse(spin_toml)
}

#[test]
fn test_transpile_minimal_config() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "test-project".to_string(),
            version: "0.1.0".to_string(),
            description: "Test project".to_string(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML:\n{result}");

    // Verify basic structure
    assert!(result.contains("spin_manifest_version = 2"));
    assert!(result.contains("[application]"));
    assert!(result.contains("name = \"test-project\""));
    assert!(result.contains("version = \"0.1.0\""));
    assert!(result.contains("[variables]"));
    assert!(result.contains("component_names"));
    assert!(result.contains("[[trigger.http]]"));

    // Auth is disabled by default, gateway is named "mcp"
    assert!(result.contains("[component.mcp]"));
    assert!(!result.contains("[component.ftl-mcp-gateway]"));

    // Validate the generated TOML can be parsed
    let spin_config = validate_spin_toml(&result).unwrap();
    assert_eq!(spin_config.spin_manifest_version, 2);
    assert_eq!(spin_config.application.name, "test-project");
    assert_eq!(spin_config.application.version, "0.1.0");
    assert_eq!(spin_config.application.description, "Test project");
}

#[test]
fn test_transpile_with_components() {
    let mut component = HashMap::new();
    component.insert(
        "echo-tool".to_string(),
        ComponentConfig {
            path: Some("echo-tool".to_string()),
            wasm: Some("echo-tool/target/wasm32-wasip1/release/echo_tool.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                watch: vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()],
                env: HashMap::new(),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec![],
            variables: HashMap::new(),
        },
    );
    component.insert(
        "weather".to_string(),
        ComponentConfig {
            path: Some("weather-ts".to_string()),
            wasm: Some("weather-ts/dist/weather.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "npm run build:custom".to_string(),
                watch: vec!["src/**/*.ts".to_string()],
                env: HashMap::new(),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec!["https://api.weather.com".to_string()],
            variables: HashMap::new(),
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "test-project".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: vec!["Test Author <test@example.com>".to_string()],
            default_registry: None,
        },
        oauth: None,
        component,
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with component:\n{result}");

    // Check components are included
    assert!(result.contains("echo-tool") && result.contains("weather"));
    assert!(result.contains("[component.echo-tool]"));
    assert!(result.contains("echo-tool/target/wasm32-wasip1/release/echo_tool.wasm"));
    assert!(result.contains("[component.weather]"));
    assert!(result.contains("weather-ts/dist/weather.wasm"));
    assert!(result.contains("npm run build:custom"));
    assert!(result.contains("https://api.weather.com"));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(spin_config.component.contains_key("echo-tool"));
    assert!(spin_config.component.contains_key("weather"));

    // Verify component components variable
    assert!(spin_config.variables.contains_key("component_names"));
    if let SpinVariable::Default { default } = &spin_config.variables["component_names"] {
        assert!(default.contains("echo-tool"));
        assert!(default.contains("weather"));
    } else {
        panic!("component_names should be a default variable");
    }
}

#[test]
fn test_transpile_with_variables() {
    let mut component = HashMap::new();
    let mut variables = HashMap::new();
    variables.insert("API_KEY".to_string(), "test-key".to_string());
    variables.insert("DEBUG".to_string(), "true".to_string());

    component.insert(
        "api-tool".to_string(),
        ComponentConfig {
            path: Some("api-tool".to_string()),
            wasm: Some("api-tool/target/wasm32-wasip1/release/api_tool.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                watch: vec![],
                env: HashMap::new(),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec![],
            variables,
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "test-project".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component,
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with variables:\n{result}");

    // Check that variables are included in the component
    assert!(result.contains("[component.api-tool.variables]"));
    assert!(result.contains("API_KEY = \"test-key\""));
    assert!(result.contains("DEBUG = \"true\""));

    // Validate and check component variables
    let spin_config = validate_spin_toml(&result).unwrap();
    let api_component = &spin_config.component["api-tool"];
    assert_eq!(api_component.variables["API_KEY"], "test-key");
    assert_eq!(api_component.variables["DEBUG"], "true");
}

#[test]
fn test_transpile_with_auth() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "auth-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // Check auth configuration - oauth is None so auth should be disabled
    assert!(result.contains("auth_enabled = { default = \"false\" }"));

    // Authentication should be disabled (no oauth)
    // Note: mcp_auth_enabled is no longer used - provider config determines auth

    // Authorization rule variables should be present but empty
    assert!(result.contains("mcp_auth_allowed_subjects = { default = \"\" }"));
    assert!(result.contains("mcp_auth_required_claims = { default = \"\" }"));

    // Validate and check auth variables
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(matches!(
        &spin_config.variables["auth_enabled"],
        SpinVariable::Default { default } if default == "false"
    ));
    // Check that provider variables are empty (auth disabled)
    assert!(matches!(
        &spin_config.variables["mcp_jwt_issuer"],
        SpinVariable::Default { default } if default.is_empty()
    ));
    assert!(matches!(
        &spin_config.variables["mcp_auth_allowed_subjects"],
        SpinVariable::Default { default } if default.is_empty()
    ));
    assert!(matches!(
        &spin_config.variables["mcp_auth_required_claims"],
        SpinVariable::Default { default } if default.is_empty()
    ));
}

#[test]
fn test_transpile_with_oauth_auth() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "oauth-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: Some(OauthConfig {
            issuer: "https://auth.example.com".to_string(),
            audience: "api".to_string(),
            jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            public_key: String::new(),
            algorithm: String::new(),
            required_scopes: String::new(),
            authorize_endpoint: "https://auth.example.com/authorize".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            userinfo_endpoint: "https://auth.example.com/userinfo".to_string(),
            allowed_subjects: vec![],
        }),
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // Check OAuth configuration
    assert!(result.contains("auth_enabled = { default = \"true\" }"));
    assert!(result.contains("mcp_provider_type = { default = \"jwt\" }"));
    assert!(result.contains(
        "mcp_jwt_jwks_uri = { default = \"https://auth.example.com/.well-known/jwks.json\" }"
    ));

    // For authentication enabled, provider should be configured
    assert!(result.contains("mcp_jwt_issuer = { default = \"https://auth.example.com\" }"));
    assert!(result.contains("mcp_auth_allowed_subjects = { default = \"\" }"));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(matches!(
        &spin_config.variables["mcp_provider_type"],
        SpinVariable::Default { default } if default == "jwt"
    ));
    // Check that provider variables are set (auth enabled)
    assert!(matches!(
        &spin_config.variables["mcp_jwt_issuer"],
        SpinVariable::Default { default } if default == "https://auth.example.com"
    ));
    assert!(matches!(
        &spin_config.variables["mcp_auth_allowed_subjects"],
        SpinVariable::Default { default } if default.is_empty()
    ));
}

#[test]
fn test_transpile_with_allowed_subjects() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "restricted-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: Some(OauthConfig {
            issuer: "https://auth.example.com".to_string(),
            audience: "api".to_string(),
            jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            public_key: String::new(),
            algorithm: String::new(),
            required_scopes: String::new(),
            authorize_endpoint: String::new(),
            token_endpoint: String::new(),
            userinfo_endpoint: String::new(),
            allowed_subjects: vec![
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
                "service-account-123".to_string(),
            ],
        }),
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // Check that allowed_subjects is properly converted to comma-separated string
    assert!(result.contains(
        "mcp_auth_allowed_subjects = { default = \"alice@example.com,bob@example.com,service-account-123\" }"
    ));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(matches!(
        &spin_config.variables["mcp_auth_allowed_subjects"],
        SpinVariable::Default { default } if default == "alice@example.com,bob@example.com,service-account-123"
    ));
}

// Static token auth is no longer supported in the new configuration
// This test is replaced with a test for public access control
#[test]
fn test_transpile_with_public_access() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "public-project".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with public access:\n{result}");

    // Check auth is disabled for public access
    assert!(result.contains("auth_enabled = { default = \"false\" }"));

    // Check component names - no auth components
    assert!(result.contains("[component.mcp]"));
    assert!(!result.contains("[component.ftl-mcp-gateway]"));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(matches!(
        &spin_config.variables["auth_enabled"],
        SpinVariable::Default { default } if default == "false"
    ));
}

#[test]
fn test_transpile_with_custom_gateway_uris() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "custom-gateway-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(),
        mcp: McpConfig {
            gateway: "ghcr.io/myorg/custom-gateway:2.0.0".to_string(),
            authorizer: "ghcr.io/myorg/custom-authorizer:2.0.0".to_string(),
            validate_arguments: true,
        },
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with custom gateway URIs:\n{result}");

    // Check that custom URIs are properly parsed into source configurations
    assert!(result.contains("[[trigger.http]]"));

    // Auth is disabled by default, gateway is named "mcp"
    assert!(result.contains("component = \"mcp\""));
    assert!(!result.contains("component = \"ftl-mcp-gateway\""));

    // Gateway component should exist with custom URI (named "mcp")
    assert!(result.contains("[component.mcp]"));
    // Now we just use a simple source string, not a structured registry format
    assert!(result.contains("source = \"ghcr.io/myorg/custom-gateway:2.0.0\""));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();

    // Gateway component exists as "mcp" (auth disabled)
    assert!(spin_config.component.contains_key("mcp"));
    assert!(!spin_config.component.contains_key("ftl-mcp-gateway"));

    // Gateway component should have a local source (string path)
    let gateway_component = &spin_config.component["mcp"];
    if let ComponentSource::Local(path) = &gateway_component.source {
        assert_eq!(path, "ghcr.io/myorg/custom-gateway:2.0.0");
    } else {
        panic!("Expected gateway component to have local source");
    }
}

#[test]
fn test_transpile_with_application_variables() {
    let mut app_vars = HashMap::new();
    app_vars.insert(
        "api_token".to_string(),
        ApplicationVariable::Required { required: true },
    );
    app_vars.insert(
        "api_url".to_string(),
        ApplicationVariable::Default {
            default: "https://api.example.com".to_string(),
        },
    );

    let mut component = HashMap::new();
    let mut component_vars = HashMap::new();
    component_vars.insert("token".to_string(), "{{ api_token }}".to_string());
    component_vars.insert("url".to_string(), "{{ api_url }}".to_string());
    component_vars.insert("version".to_string(), "v1".to_string());

    component.insert(
        "api-consumer".to_string(),
        ComponentConfig {
            path: Some("api-consumer".to_string()),
            wasm: Some("api-consumer/target/wasm32-wasip1/release/api_consumer.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                watch: vec![],
                env: HashMap::new(),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec!["{{ api_url }}".to_string()],
            variables: component_vars,
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "test-project".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component,
        mcp: McpConfig::default(),
        variables: app_vars,
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with app variables:\n{result}");

    // Check application variables
    assert!(result.contains("api_token = { required = true }"));
    assert!(result.contains("api_url = { default = \"https://api.example.com\" }"));

    // Check component variables with templates
    assert!(result.contains("[component.api-consumer.variables]"));
    assert!(result.contains("token = \"{{ api_token }}\""));
    assert!(result.contains("url = \"{{ api_url }}\""));
    assert!(result.contains("version = \"v1\""));

    // Check allowed_outbound_hosts with template
    assert!(result.contains("allowed_outbound_hosts = [\"{{ api_url }}\"]"));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(matches!(
        &spin_config.variables["api_token"],
        SpinVariable::Required { required: true }
    ));
    assert!(matches!(
        &spin_config.variables["api_url"],
        SpinVariable::Default { default } if default == "https://api.example.com"
    ));
}

#[test]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn test_transpile_complete_example() {
    // Create a complete FTL configuration with all features
    let mut app_vars = HashMap::new();
    app_vars.insert(
        "database_url".to_string(),
        ApplicationVariable::Required { required: true },
    );
    app_vars.insert(
        "log_level".to_string(),
        ApplicationVariable::Default {
            default: "info".to_string(),
        },
    );

    let mut component = HashMap::new();

    // Component 1: Database component with environment variables
    let mut db_vars = HashMap::new();
    db_vars.insert("db_url".to_string(), "{{ database_url }}".to_string());
    db_vars.insert("pool_size".to_string(), "10".to_string());

    let mut db_env = HashMap::new();
    db_env.insert("RUST_LOG".to_string(), "debug".to_string());

    component.insert(
        "database".to_string(),
        ComponentConfig {
            path: Some("tools/database".to_string()),
            wasm: Some("tools/database/target/wasm32-wasip1/release/database.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                watch: vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()],
                env: db_env,
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec!["postgres://db.example.com:5432".to_string()],
            variables: db_vars,
        },
    );

    // Component 2: API component with multiple allowed hosts
    let mut api_component_vars = HashMap::new();
    api_component_vars.insert("api_key".to_string(), "{{ api_key }}".to_string());
    api_component_vars.insert("log_level".to_string(), "{{ log_level }}".to_string());

    component.insert(
        "api-client".to_string(),
        ComponentConfig {
            path: Some("tools/api".to_string()),
            wasm: Some("tools/api/dist/api.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "npm run build".to_string(),
                watch: vec!["src/**/*.ts".to_string(), "package.json".to_string()],
                env: HashMap::new(),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec![
                "https://api.example.com".to_string(),
                "https://backup.example.com".to_string(),
                "*://cdn.example.com:*".to_string(),
            ],
            variables: api_component_vars,
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "complete-example".to_string(),
            version: "2.1.0".to_string(),
            description: "A complete example with all features".to_string(),
            authors: vec![
                "John Doe <john@example.com>".to_string(),
                "Jane Smith <jane@example.com>".to_string(),
            ],
            default_registry: None,
        },
        oauth: None,
        component,
        mcp: McpConfig {
            gateway: "ghcr.io/example/gateway:3.0.0".to_string(),
            authorizer: "ghcr.io/example/auth:3.0.0".to_string(),
            validate_arguments: false,
        },
        variables: app_vars,
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated complete TOML:\n{result}");

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();

    // Check application metadata
    assert_eq!(spin_config.application.name, "complete-example");
    assert_eq!(spin_config.application.version, "2.1.0");
    assert_eq!(
        spin_config.application.description,
        "A complete example with all features"
    );
    assert_eq!(spin_config.application.authors.len(), 2);

    // Check all components exist - private mode without OAuth has auth components
    assert!(spin_config.component.contains_key("mcp"));
    assert!(spin_config.component.contains_key("mcp"));
    assert!(spin_config.component.contains_key("database"));
    assert!(spin_config.component.contains_key("api-client"));

    // Check database component details
    let db_component = &spin_config.component["database"];
    assert_eq!(db_component.allowed_outbound_hosts.len(), 1);
    assert_eq!(
        db_component.allowed_outbound_hosts[0],
        "postgres://db.example.com:5432"
    );
    assert!(db_component.build.is_some());
    let db_build = db_component.build.as_ref().unwrap();
    assert_eq!(db_build.environment["RUST_LOG"], "debug");

    // Check API component details
    let api_component = &spin_config.component["api-client"];
    assert_eq!(api_component.allowed_outbound_hosts.len(), 3);
    assert!(
        api_component
            .allowed_outbound_hosts
            .contains(&"https://api.example.com".to_string())
    );

    // Check variables
    assert!(matches!(
        &spin_config.variables["database_url"],
        SpinVariable::Required { required: true }
    ));
    assert!(matches!(
        &spin_config.variables["log_level"],
        SpinVariable::Default { default } if default == "info"
    ));

    // Check auth is disabled (no oauth configured)
    assert!(matches!(
        &spin_config.variables["auth_enabled"],
        SpinVariable::Default { default } if default == "false"
    ));
}

#[test]
fn test_transpile_with_build_profiles() {
    // Test transpilation of components with build profiles
    let mut component = HashMap::new();

    let mut profiles = HashMap::new();
    profiles.insert(
        "dev".to_string(),
        BuildProfile {
            command: "cargo build --target wasm32-wasip1".to_string(),
            watch: vec!["src/**/*.rs".to_string()],
            env: HashMap::from([("RUST_LOG".to_string(), "debug".to_string())]),
        },
    );
    profiles.insert(
        "release".to_string(),
        BuildProfile {
            command: "cargo build --target wasm32-wasip1 --release".to_string(),
            watch: vec![],
            env: HashMap::from([("RUST_LOG".to_string(), "warn".to_string())]),
        },
    );

    component.insert(
        "profiled-tool".to_string(),
        ComponentConfig {
            path: Some("profiled".to_string()),
            wasm: Some("profiled/target/wasm32-wasip1/release/profiled.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "cargo build --target wasm32-wasip1".to_string(),
                watch: vec!["src/**/*.rs".to_string()],
                env: HashMap::new(),
            }),
            profiles: Some(BuildProfiles { profiles }),
            up: Some(UpConfig {
                profile: "dev".to_string(),
            }),
            deploy: Some(DeployConfig {
                profile: "release".to_string(),
                name: None,
            }),
            allowed_outbound_hosts: vec![],
            variables: HashMap::new(),
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "profile-test".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component,
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // The transpiler currently uses the default build config
    // Profiles are handled at build time, not in the manifest
    assert!(result.contains("[component.profiled-tool.build]"));
    assert!(result.contains("command = \"cargo build --target wasm32-wasip1\""));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(spin_config.component.contains_key("profiled-tool"));
}

#[test]
fn test_transpile_with_special_characters() {
    // Test handling of special characters in various fields
    let mut app_vars = HashMap::new();
    app_vars.insert(
        "special-var-name".to_string(),
        ApplicationVariable::Default {
            default: "value with \"quotes\" and 'apostrophes'".to_string(),
        },
    );

    let mut component = HashMap::new();
    let mut component_vars = HashMap::new();
    component_vars.insert("path".to_string(), "/path/with spaces/file.txt".to_string());
    component_vars.insert(
        "url".to_string(),
        "https://example.com/api?key=value&foo=bar".to_string(),
    );

    component.insert(
        "special-chars".to_string(),
        ComponentConfig {
            path: Some("tools/special".to_string()),
            wasm: Some("tools/special/output.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "npm run build -- --config=\"production\"".to_string(),
                watch: vec!["src/**/*.{ts,tsx}".to_string()],
                env: HashMap::from([
                    ("NODE_ENV".to_string(), "production".to_string()),
                    (
                        "API_URL".to_string(),
                        "https://api.example.com/v1".to_string(),
                    ),
                ]),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec!["https://api.example.com:8443".to_string()],
            variables: component_vars,
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "special-chars-test".to_string(),
            version: "0.1.0".to_string(),
            description: "Testing \"special\" characters & symbols".to_string(),
            authors: vec!["Author <test@example.com> (Company & Co.)".to_string()],
            default_registry: None,
        },
        oauth: None,
        component,
        mcp: McpConfig::default(),
        variables: app_vars,
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // Validate the generated TOML can handle special characters
    let spin_config = validate_spin_toml(&result).unwrap();
    assert_eq!(
        spin_config.application.description,
        "Testing \"special\" characters & symbols"
    );

    // Check that variables are properly escaped
    if let SpinVariable::Default { default } = &spin_config.variables["special-var-name"] {
        assert_eq!(default, "value with \"quotes\" and 'apostrophes'");
    } else {
        panic!("special-var-name should be a default variable");
    }
}

#[test]
fn test_transpile_empty_collections() {
    // Test handling of empty collections
    let config = FtlConfig {
        project: ProjectConfig {
            name: "empty-test".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(), // Empty description
            authors: vec![],            // Empty authors
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(), // No components
        mcp: McpConfig::default(),
        variables: HashMap::new(), // No variables
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // Should still generate valid TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert_eq!(spin_config.application.name, "empty-test");
    assert!(spin_config.application.description.is_empty());
    assert!(spin_config.application.authors.is_empty());

    // Auth is disabled by default, gateway is named "mcp"
    assert!(spin_config.component.contains_key("mcp"));
    assert!(!spin_config.component.contains_key("ftl-mcp-gateway"));

    // Should have system variables but no custom ones
    assert!(spin_config.variables.contains_key("component_names"));
    assert!(spin_config.variables.contains_key("auth_enabled"));
}

#[test]
fn test_parse_component_source() {
    // Test that parse_component_source now just returns local paths
    // Since we're downloading registry components with wkg, everything becomes a local path

    let test_cases = vec![
        ("app.wasm", "app.wasm"),
        ("/path/to/app.wasm", "/path/to/app.wasm"),
        // Even registry URLs are now treated as local paths (will be downloaded by wkg)
        (
            "ghcr.io/myorg/my-component:1.0.0",
            "ghcr.io/myorg/my-component:1.0.0",
        ),
    ];

    for (input, expected) in test_cases {
        let result = parse_component_source(input, None);
        match result {
            ComponentSource::Local(path) => {
                assert_eq!(path, expected, "Path mismatch for input: {input}");
            }
            _ => panic!("Expected Local source for input: {input}"),
        }
    }
}

#[test]
fn test_http_trigger_generation() {
    // Test that HTTP triggers are correctly generated
    let mut component = HashMap::new();
    component.insert(
        "tool1".to_string(),
        ComponentConfig {
            path: Some("tool1".to_string()),
            wasm: Some("tool1/output.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "make".to_string(),
                watch: vec![],
                env: HashMap::new(),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec![],
            variables: HashMap::new(),
        },
    );
    component.insert(
        "tool2".to_string(),
        ComponentConfig {
            path: Some("tool2".to_string()),
            wasm: Some("tool2/output.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "make".to_string(),
                watch: vec![],
                env: HashMap::new(),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec![],
            variables: HashMap::new(),
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "trigger-test".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component,
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // Check trigger generation
    assert!(result.contains("[[trigger.http]]"));
    assert!(result.contains("route = \"/...\""));

    // Auth is disabled by default, so OAuth endpoints should NOT be present
    assert!(!result.contains("route = \"/.well-known/oauth-protected-resource\""));
    assert!(!result.contains("route = \"/.well-known/oauth-authorization-server\""));

    // Count private route triggers (2 components = 2, gateway is not private when auth is disabled)
    let private_count = result.matches("route = { private = true }").count();
    assert_eq!(private_count, 2);

    // Each component should have a trigger
    let tool1_triggers = result.matches("component = \"tool1\"").count();
    let tool2_triggers = result.matches("component = \"tool2\"").count();
    assert_eq!(tool1_triggers, 1);
    assert_eq!(tool2_triggers, 1);
}

#[test]
fn test_auth_disabled_omits_authorizer() {
    // Test that when auth is disabled (public access), the authorizer component is completely omitted
    let config = FtlConfig {
        project: ProjectConfig {
            name: "no-auth-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with auth disabled:\n{result}");

    // Check that auth is disabled
    assert!(result.contains("auth_enabled = { default = \"false\" }"));

    // Check that gateway exists as "mcp" component (no separate authorizer)
    assert!(result.contains("[component.mcp]")); // This is the gateway when auth disabled
    assert!(!result.contains("[component.ftl-mcp-gateway]"));

    // Check that OAuth endpoints are NOT present
    assert!(!result.contains("/.well-known/oauth-protected-resource"));
    assert!(!result.contains("/.well-known/oauth-authorization-server"));

    // Check that auth variables are set to empty (auth disabled)
    assert!(result.contains("mcp_provider_type = { default = \"\" }"));
    assert!(result.contains("mcp_jwt_issuer = { default = \"\" }"));
    assert!(result.contains("mcp_jwt_audience = { default = \"\" }"));
    assert!(result.contains("mcp_jwt_jwks_uri = { default = \"\" }"));
    // Gateway URL should be "none" when auth is disabled
    assert!(result.contains("mcp_gateway_url = { default = \"none\" }"));
    // Trace header is always set
    assert!(result.contains("mcp_trace_header = { default = \"x-trace-id\" }"));

    // Check that wildcard route points directly to gateway (named "mcp")
    assert!(result.contains("route = \"/...\""));
    assert!(result.contains("component = \"mcp\""));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();

    // Verify that gateway exists as "mcp" component
    assert!(spin_config.component.contains_key("mcp"));

    // Verify that ftl-mcp-gateway component doesn't exist
    assert!(!spin_config.component.contains_key("ftl-mcp-gateway"));

    // Verify auth_enabled is false
    assert!(matches!(
        &spin_config.variables["auth_enabled"],
        SpinVariable::Default { default } if default == "false"
    ));
}

#[test]
fn test_auth_enabled_includes_authorizer() {
    // Test that when auth is enabled (oauth configured), all auth components are included
    let config = FtlConfig {
        project: ProjectConfig {
            name: "auth-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: Some(OauthConfig {
            issuer: "https://auth.example.com".to_string(),
            audience: String::new(),
            authorize_endpoint: String::new(),
            token_endpoint: String::new(),
            userinfo_endpoint: String::new(),
            jwks_uri: String::new(),
            public_key: String::new(),
            algorithm: String::new(),
            required_scopes: String::new(),
            allowed_subjects: vec![],
        }),
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with auth enabled:\n{result}");

    // Check that auth is enabled
    assert!(result.contains("auth_enabled = { default = \"true\" }"));

    // Check that MCP authorizer component IS present
    assert!(result.contains("[component.mcp]"));
    assert!(result.contains("[component.ftl-mcp-gateway]"));

    // Check that wildcard route is present
    assert!(result.contains("route = \"/...\""));

    // Check that auth variables ARE included
    assert!(result.contains("mcp_provider_type"));
    assert!(result.contains("mcp_jwt_issuer"));
    assert!(result.contains("mcp_jwt_audience"));
    assert!(result.contains("mcp_gateway_url"));
    assert!(result.contains("mcp_trace_header"));

    // Check that wildcard route points to authorizer
    let wildcard_route_matches: Vec<_> = result.match_indices("route = \"/...\"").collect();
    assert!(
        !wildcard_route_matches.is_empty(),
        "Should have wildcard route"
    );

    // Find the component for the wildcard route
    let wildcard_route_pos = wildcard_route_matches[0].0;
    let after_route = &result[wildcard_route_pos..];
    assert!(after_route.contains("component = \"mcp\""));

    // Check that gateway has private route
    assert!(result.contains("route = { private = true }"));
    assert!(result.contains("component = \"ftl-mcp-gateway\""));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();

    // Verify that mcp component exists
    assert!(spin_config.component.contains_key("mcp"));

    // Verify that gateway component exists
    assert!(spin_config.component.contains_key("mcp"));

    // Verify auth_enabled is true
    assert!(matches!(
        &spin_config.variables["auth_enabled"],
        SpinVariable::Default { default } if default == "true"
    ));
}

#[test]
fn test_validate_local_auth() {
    // Test that only public and custom modes are allowed locally

    // Public mode should work
    let public_config = FtlConfig {
        project: ProjectConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };
    assert!(super::validate_local_auth(&public_config).is_ok());

    // Custom mode with OAuth should work
    let custom_with_oauth = FtlConfig {
        project: ProjectConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: Some(OauthConfig {
            issuer: "https://example.com".to_string(),
            audience: String::new(),
            jwks_uri: "https://example.com/jwks".to_string(),
            public_key: String::new(),
            algorithm: String::new(),
            required_scopes: String::new(),
            authorize_endpoint: String::new(),
            token_endpoint: String::new(),
            userinfo_endpoint: String::new(),
            allowed_subjects: vec![],
        }),
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };
    assert!(super::validate_local_auth(&custom_with_oauth).is_ok());

    // Any config without OAuth should also pass validation
    let no_oauth_config = FtlConfig {
        project: ProjectConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };
    assert!(super::validate_local_auth(&no_oauth_config).is_ok());
}

#[test]
fn test_auth_disabled_with_components() {
    // Test that components work correctly when auth is disabled (public access)
    let mut component = HashMap::new();
    component.insert(
        "my-component".to_string(),
        ComponentConfig {
            path: Some("my-component".to_string()),
            wasm: Some("my-component/output.wasm".to_string()),
            repo: None,
            build: Some(BuildConfig {
                command: "make".to_string(),
                watch: vec![],
                env: HashMap::new(),
            }),
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec!["https://api.example.com".to_string()],
            variables: HashMap::new(),
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "no-auth-with-tools".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
            default_registry: None,
        },
        oauth: None,
        component,
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with auth disabled and component:\n{result}");

    // Check that auth is disabled
    assert!(result.contains("auth_enabled = { default = \"false\" }"));

    // Check that gateway exists as "mcp" (no separate authorizer)
    assert!(result.contains("[component.mcp]"));

    // Check that ftl-mcp-gateway component doesn't exist
    assert!(!result.contains("[component.ftl-mcp-gateway]"));

    // Check that component component exists
    assert!(result.contains("[component.my-component]"));

    // Check that wildcard route points directly to gateway
    let wildcard_routes: Vec<_> = result.match_indices("route = \"/...\"").collect();
    assert_eq!(
        wildcard_routes.len(),
        1,
        "Should have exactly one wildcard route"
    );

    // Verify it's followed by gateway component named "mcp"
    let after_wildcard = &result[wildcard_routes[0].0..];
    assert!(after_wildcard.contains("component = \"mcp\""));

    // Check that component has private route
    assert!(result.contains("component = \"my-component\""));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();

    // Verify components
    assert!(spin_config.component.contains_key("mcp")); // Gateway is named "mcp"
    assert!(!spin_config.component.contains_key("ftl-mcp-gateway"));
    assert!(spin_config.component.contains_key("my-component"));
}
