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
        },
        auth: AuthConfig::default(),
        tools: HashMap::new(),
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
    assert!(result.contains("tool_components"));
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
fn test_generate_temp_spin_toml_absolute_paths() {
    use crate::test_helpers::MockFileSystemMock;
    use ftl_runtime::deps::FileSystem;
    use std::sync::Arc;

    // Use a real temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let project_path = temp_dir.path();
    let ftl_path = project_path.join("ftl.toml");

    let mut fs_mock = MockFileSystemMock::new();

    // Mock ftl.toml exists
    let ftl_path_clone = ftl_path.clone();
    fs_mock
        .expect_exists()
        .withf(move |path| *path == ftl_path_clone)
        .returning(|_| true);

    // Mock reading ftl.toml with relative paths
    let ftl_content = r#"
[project]
name = "test-project"
version = "0.1.0"

[tools.my-tool]
path = "my-tool"
wasm = "my-tool/target/wasm32-wasip1/release/my_tool.wasm"

[tools.my-tool.build]
command = "cargo build --release --target wasm32-wasip1"
"#;

    fs_mock
        .expect_read_to_string()
        .withf(move |path| *path == ftl_path)
        .returning(move |_| Ok(ftl_content.to_string()));

    let fs: Arc<dyn FileSystem> = Arc::new(fs_mock);

    // Generate temp spin.toml
    let result = generate_temp_spin_toml(&fs, project_path).unwrap();
    assert!(result.is_some());

    let temp_path = result.unwrap();

    // Read the generated spin.toml
    let spin_content = std::fs::read_to_string(&temp_path).unwrap();

    // Verify that paths are absolute - they should start with /
    // Note: We can't check exact paths because canonicalize() may resolve symlinks
    // (e.g., /var -> /private/var on macOS)

    // Check that wasm path is absolute
    assert!(
        spin_content.contains("source = \"/"),
        "Expected wasm path to be absolute (start with /) in:\n{spin_content}"
    );

    // Check that the wasm path ends with the expected relative part
    assert!(
        spin_content.contains("/my-tool/target/wasm32-wasip1/release/my_tool.wasm\""),
        "Expected wasm path to contain the correct relative path in:\n{spin_content}"
    );

    // Check that workdir is absolute
    assert!(
        spin_content.contains("workdir = \"/"),
        "Expected workdir to be absolute (start with /) in:\n{spin_content}"
    );

    // Check that workdir ends with the expected relative part
    assert!(
        spin_content.contains("/my-tool\""),
        "Expected workdir to contain the correct relative path in:\n{spin_content}"
    );
}

#[test]
fn test_transpile_with_tools() {
    let mut tools = HashMap::new();
    tools.insert(
        "echo-tool".to_string(),
        ToolConfig {
            path: Some("echo-tool".to_string()),
            wasm: "echo-tool/target/wasm32-wasip1/release/echo_tool.wasm".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                watch: vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()],
                env: HashMap::new(),
            },
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec![],
            variables: HashMap::new(),
        },
    );
    tools.insert(
        "weather".to_string(),
        ToolConfig {
            path: Some("weather-ts".to_string()),
            wasm: "weather-ts/dist/weather.wasm".to_string(),
            build: BuildConfig {
                command: "npm run build:custom".to_string(),
                watch: vec!["src/**/*.ts".to_string()],
                env: HashMap::new(),
            },
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
        },
        auth: AuthConfig::default(),
        tools,
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with tools:\n{result}");

    // Check tools are included
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

    // Verify tool components variable
    assert!(spin_config.variables.contains_key("tool_components"));
    if let SpinVariable::Default { default } = &spin_config.variables["tool_components"] {
        assert!(default.contains("echo-tool"));
        assert!(default.contains("weather"));
    } else {
        panic!("tool_components should be a default variable");
    }
}

#[test]
fn test_transpile_with_variables() {
    let mut tools = HashMap::new();
    let mut variables = HashMap::new();
    variables.insert("API_KEY".to_string(), "test-key".to_string());
    variables.insert("DEBUG".to_string(), "true".to_string());

    tools.insert(
        "api-tool".to_string(),
        ToolConfig {
            path: Some("api-tool".to_string()),
            wasm: "api-tool/target/wasm32-wasip1/release/api_tool.wasm".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                watch: vec![],
                env: HashMap::new(),
            },
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
        },
        auth: AuthConfig::default(),
        tools,
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
    let api_tool = &spin_config.component["api-tool"];
    assert_eq!(api_tool.variables["API_KEY"], "test-key");
    assert_eq!(api_tool.variables["DEBUG"], "true");
}

#[test]
fn test_transpile_with_auth() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "auth-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
        },
        auth: AuthConfig {
            enabled: true,
            authkit: Some(AuthKitConfig {
                issuer: "https://my-tenant.authkit.app".to_string(),
                audience: "mcp-api".to_string(),
                required_scopes: String::new(),
            }),
            oidc: None,
            static_token: None,
        },
        tools: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // Check auth configuration
    assert!(result.contains("auth_enabled = { default = \"true\" }"));
    assert!(result.contains("mcp_provider_type = { default = \"jwt\" }"));
    assert!(result.contains("mcp_jwt_issuer = { default = \"https://my-tenant.authkit.app\" }"));
    assert!(result.contains("mcp_jwt_audience = { default = \"mcp-api\" }"));

    // Validate and check auth variables
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(matches!(
        &spin_config.variables["auth_enabled"],
        SpinVariable::Default { default } if default == "true"
    ));
    assert!(matches!(
        &spin_config.variables["mcp_provider_type"],
        SpinVariable::Default { default } if default == "jwt"
    ));
}

#[test]
fn test_transpile_with_oidc_auth() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "oidc-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
        },
        auth: AuthConfig {
            enabled: true,
            authkit: None,
            oidc: Some(OidcConfig {
                issuer: "https://auth.example.com".to_string(),
                audience: "api".to_string(),
                jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
                public_key: String::new(),
                algorithm: String::new(),
                required_scopes: String::new(),
                authorize_endpoint: "https://auth.example.com/authorize".to_string(),
                token_endpoint: "https://auth.example.com/token".to_string(),
                userinfo_endpoint: "https://auth.example.com/userinfo".to_string(),
            }),
            static_token: None,
        },
        tools: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    // Check OIDC configuration
    assert!(result.contains("auth_enabled = { default = \"true\" }"));
    assert!(result.contains("mcp_provider_type = { default = \"jwt\" }"));
    assert!(result.contains(
        "mcp_jwt_jwks_uri = { default = \"https://auth.example.com/.well-known/jwks.json\" }"
    ));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(matches!(
        &spin_config.variables["mcp_provider_type"],
        SpinVariable::Default { default } if default == "jwt"
    ));
}

#[test]
fn test_transpile_with_static_token_auth() {
    let config = FtlConfig {
        project: ProjectConfig {
            name: "static-auth-project".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: vec![],
        },
        auth: AuthConfig {
            enabled: true,
            authkit: None,
            oidc: None,
            static_token: Some(StaticTokenConfig {
                tokens:
                    "dev-token:client1:user1:read,write;admin-token:admin:admin:admin:1735689600"
                        .to_string(),
                required_scopes: "read".to_string(),
            }),
        },
        tools: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with static token auth:\n{result}");

    // Check auth is enabled
    assert!(result.contains("auth_enabled = { default = \"true\" }"));

    // Check static provider type
    assert!(result.contains("mcp_provider_type = { default = \"static\" }"));
    assert!(result.contains("mcp_static_tokens = { default = \"dev-token:client1:user1:read,write;admin-token:admin:admin:admin:1735689600\" }"));
    assert!(result.contains("mcp_jwt_required_scopes = { default = \"read\" }"));

    // Check component names
    assert!(result.contains("[component.mcp]"));
    assert!(result.contains("[component.ftl-mcp-gateway]"));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();
    assert!(matches!(
        &spin_config.variables["mcp_provider_type"],
        SpinVariable::Default { default } if default == "static"
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
        },
        auth: AuthConfig::default(),
        tools: HashMap::new(),
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
    assert!(result.contains("[component.mcp.source]"));
    assert!(result.contains("package = \"myorg:custom-gateway\""));
    assert!(result.contains("version = \"2.0.0\""));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();

    // Gateway component exists as "mcp" (auth disabled)
    assert!(spin_config.component.contains_key("mcp"));
    assert!(!spin_config.component.contains_key("ftl-mcp-gateway"));

    // Gateway component should exist with custom source
    let gateway_component = &spin_config.component["mcp"];
    if let ComponentSource::Registry {
        registry,
        package,
        version,
    } = &gateway_component.source
    {
        assert_eq!(registry, "ghcr.io");
        assert_eq!(package, "myorg:custom-gateway");
        assert_eq!(version, "2.0.0");
    } else {
        panic!("Gateway component should have registry source");
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

    let mut tools = HashMap::new();
    let mut tool_vars = HashMap::new();
    tool_vars.insert("token".to_string(), "{{ api_token }}".to_string());
    tool_vars.insert("url".to_string(), "{{ api_url }}".to_string());
    tool_vars.insert("version".to_string(), "v1".to_string());

    tools.insert(
        "api-consumer".to_string(),
        ToolConfig {
            path: Some("api-consumer".to_string()),
            wasm: "api-consumer/target/wasm32-wasip1/release/api_consumer.wasm".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                watch: vec![],
                env: HashMap::new(),
            },
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec!["{{ api_url }}".to_string()],
            variables: tool_vars,
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "test-project".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            authors: vec![],
        },
        auth: AuthConfig::default(),
        tools,
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

    let mut tools = HashMap::new();

    // Tool 1: Database tool with environment variables
    let mut db_vars = HashMap::new();
    db_vars.insert("db_url".to_string(), "{{ database_url }}".to_string());
    db_vars.insert("pool_size".to_string(), "10".to_string());

    let mut db_env = HashMap::new();
    db_env.insert("RUST_LOG".to_string(), "debug".to_string());

    tools.insert(
        "database".to_string(),
        ToolConfig {
            path: Some("tools/database".to_string()),
            wasm: "tools/database/target/wasm32-wasip1/release/database.wasm".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                watch: vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()],
                env: db_env,
            },
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec!["postgres://db.example.com:5432".to_string()],
            variables: db_vars,
        },
    );

    // Tool 2: API tool with multiple allowed hosts
    let mut api_tool_vars = HashMap::new();
    api_tool_vars.insert("api_key".to_string(), "{{ api_key }}".to_string());
    api_tool_vars.insert("log_level".to_string(), "{{ log_level }}".to_string());

    tools.insert(
        "api-client".to_string(),
        ToolConfig {
            path: Some("tools/api".to_string()),
            wasm: "tools/api/dist/api.wasm".to_string(),
            build: BuildConfig {
                command: "npm run build".to_string(),
                watch: vec!["src/**/*.ts".to_string(), "package.json".to_string()],
                env: HashMap::new(),
            },
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec![
                "https://api.example.com".to_string(),
                "https://backup.example.com".to_string(),
                "*://cdn.example.com:*".to_string(),
            ],
            variables: api_tool_vars,
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
        },
        auth: AuthConfig {
            enabled: true,
            authkit: Some(AuthKitConfig {
                issuer: "https://example.authkit.app".to_string(),
                audience: "complete-example-api".to_string(),
                required_scopes: String::new(),
            }),
            oidc: None,
            static_token: None,
        },
        tools,
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

    // Check all components exist
    assert!(spin_config.component.contains_key("mcp"));
    assert!(spin_config.component.contains_key("ftl-mcp-gateway"));
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

    // Check auth is properly configured
    assert!(matches!(
        &spin_config.variables["auth_enabled"],
        SpinVariable::Default { default } if default == "true"
    ));
    assert!(matches!(
        &spin_config.variables["mcp_provider_type"],
        SpinVariable::Default { default } if default == "jwt"
    ));
}

#[test]
fn test_transpile_with_build_profiles() {
    // Test transpilation of tools with build profiles
    let mut tools = HashMap::new();

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

    tools.insert(
        "profiled-tool".to_string(),
        ToolConfig {
            path: Some("profiled".to_string()),
            wasm: "profiled/target/wasm32-wasip1/release/profiled.wasm".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1".to_string(),
                watch: vec!["src/**/*.rs".to_string()],
                env: HashMap::new(),
            },
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
        },
        auth: AuthConfig::default(),
        tools,
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

    let mut tools = HashMap::new();
    let mut tool_vars = HashMap::new();
    tool_vars.insert("path".to_string(), "/path/with spaces/file.txt".to_string());
    tool_vars.insert(
        "url".to_string(),
        "https://example.com/api?key=value&foo=bar".to_string(),
    );

    tools.insert(
        "special-chars".to_string(),
        ToolConfig {
            path: Some("tools/special".to_string()),
            wasm: "tools/special/output.wasm".to_string(),
            build: BuildConfig {
                command: "npm run build -- --config=\"production\"".to_string(),
                watch: vec!["src/**/*.{ts,tsx}".to_string()],
                env: HashMap::from([
                    ("NODE_ENV".to_string(), "production".to_string()),
                    (
                        "API_URL".to_string(),
                        "https://api.example.com/v1".to_string(),
                    ),
                ]),
            },
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec!["https://api.example.com:8443".to_string()],
            variables: tool_vars,
        },
    );

    let config = FtlConfig {
        project: ProjectConfig {
            name: "special-chars-test".to_string(),
            version: "0.1.0".to_string(),
            description: "Testing \"special\" characters & symbols".to_string(),
            authors: vec!["Author <test@example.com> (Company & Co.)".to_string()],
        },
        auth: AuthConfig::default(),
        tools,
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
        },
        auth: AuthConfig::default(),
        tools: HashMap::new(), // No tools
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
    assert!(spin_config.variables.contains_key("tool_components"));
    assert!(spin_config.variables.contains_key("auth_enabled"));
}

#[test]
fn test_registry_uri_parsing() {
    // Test various registry URI formats
    let test_cases = vec![
        (
            "ghcr.io/myorg/my-tool:1.0.0",
            ComponentSource::Registry {
                registry: "ghcr.io".to_string(),
                package: "myorg:my-tool".to_string(),
                version: "1.0.0".to_string(),
            },
        ),
        (
            "docker.io/namespace/component:v2.1.0-beta",
            ComponentSource::Registry {
                registry: "docker.io".to_string(),
                package: "namespace:component".to_string(),
                version: "v2.1.0-beta".to_string(),
            },
        ),
        (
            "registry.example.com:5000/org/suborg/tool:latest",
            ComponentSource::Registry {
                registry: "registry.example.com:5000".to_string(),
                package: "org:suborg:tool".to_string(),
                version: "latest".to_string(),
            },
        ),
    ];

    for (uri, expected) in test_cases {
        let result = parse_registry_uri_to_source(uri);
        match (result, expected) {
            (
                ComponentSource::Registry {
                    registry: r1,
                    package: p1,
                    version: v1,
                },
                ComponentSource::Registry {
                    registry: r2,
                    package: p2,
                    version: v2,
                },
            ) => {
                assert_eq!(r1, r2, "Registry mismatch for URI: {uri}");
                assert_eq!(p1, p2, "Package mismatch for URI: {uri}");
                assert_eq!(v1, v2, "Version mismatch for URI: {uri}");
            }
            _ => panic!("Unexpected parse result for URI: {uri}"),
        }
    }
}

#[test]
fn test_http_trigger_generation() {
    // Test that HTTP triggers are correctly generated
    let mut tools = HashMap::new();
    tools.insert(
        "tool1".to_string(),
        ToolConfig {
            path: Some("tool1".to_string()),
            wasm: "tool1/output.wasm".to_string(),
            build: BuildConfig {
                command: "make".to_string(),
                watch: vec![],
                env: HashMap::new(),
            },
            profiles: None,
            up: None,
            deploy: None,
            allowed_outbound_hosts: vec![],
            variables: HashMap::new(),
        },
    );
    tools.insert(
        "tool2".to_string(),
        ToolConfig {
            path: Some("tool2".to_string()),
            wasm: "tool2/output.wasm".to_string(),
            build: BuildConfig {
                command: "make".to_string(),
                watch: vec![],
                env: HashMap::new(),
            },
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
        },
        auth: AuthConfig::default(),
        tools,
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

    // Count private route triggers (2 tools = 2, gateway is not private when auth is disabled)
    let private_count = result.matches("route = { private = true }").count();
    assert_eq!(private_count, 2);

    // Each tool should have a trigger
    let tool1_triggers = result.matches("component = \"tool1\"").count();
    let tool2_triggers = result.matches("component = \"tool2\"").count();
    assert_eq!(tool1_triggers, 1);
    assert_eq!(tool2_triggers, 1);
}

#[test]
fn test_auth_disabled_omits_authorizer() {
    // Test that when auth is disabled, the authorizer component is completely omitted
    let config = FtlConfig {
        project: ProjectConfig {
            name: "no-auth-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
        },
        auth: AuthConfig {
            enabled: false,
            authkit: None,
            oidc: None,
            static_token: None,
        },
        tools: HashMap::new(),
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

    // Check that auth variables are NOT included (except auth_enabled)
    assert!(!result.contains("mcp_provider_type"));
    assert!(!result.contains("mcp_jwt_issuer"));
    assert!(!result.contains("mcp_jwt_audience"));
    assert!(!result.contains("mcp_jwt_jwks_uri"));
    assert!(!result.contains("mcp_gateway_url"));
    assert!(!result.contains("mcp_trace_header"));

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
    // Test that when auth is enabled, all auth components are included
    let config = FtlConfig {
        project: ProjectConfig {
            name: "auth-project".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            authors: vec![],
        },
        auth: AuthConfig {
            enabled: true,
            authkit: Some(AuthKitConfig {
                issuer: "https://example.authkit.app".to_string(),
                audience: "test-api".to_string(),
                required_scopes: String::new(),
            }),
            oidc: None,
            static_token: None,
        },
        tools: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with auth enabled:\n{result}");

    // Check that auth is enabled
    assert!(result.contains("auth_enabled = { default = \"true\" }"));

    // Check that MCP authorizer component IS present
    assert!(result.contains("[component.mcp]"));

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
    assert!(spin_config.component.contains_key("ftl-mcp-gateway"));

    // Verify auth_enabled is true
    assert!(matches!(
        &spin_config.variables["auth_enabled"],
        SpinVariable::Default { default } if default == "true"
    ));
}

#[test]
fn test_auth_disabled_with_tools() {
    // Test that tools work correctly when auth is disabled
    let mut tools = HashMap::new();
    tools.insert(
        "my-tool".to_string(),
        ToolConfig {
            path: Some("my-tool".to_string()),
            wasm: "my-tool/output.wasm".to_string(),
            build: BuildConfig {
                command: "make".to_string(),
                watch: vec![],
                env: HashMap::new(),
            },
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
        },
        auth: AuthConfig {
            enabled: false,
            authkit: None,
            oidc: None,
            static_token: None,
        },
        tools,
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let result = transpile_ftl_to_spin(&config).unwrap();

    println!("Generated TOML with auth disabled and tools:\n{result}");

    // Check that auth is disabled
    assert!(result.contains("auth_enabled = { default = \"false\" }"));

    // Check that gateway exists as "mcp" (no separate authorizer)
    assert!(result.contains("[component.mcp]"));

    // Check that ftl-mcp-gateway component doesn't exist
    assert!(!result.contains("[component.ftl-mcp-gateway]"));

    // Check that tool component exists
    assert!(result.contains("[component.my-tool]"));

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

    // Check that tool has private route
    assert!(result.contains("component = \"my-tool\""));

    // Validate the generated TOML
    let spin_config = validate_spin_toml(&result).unwrap();

    // Verify components
    assert!(spin_config.component.contains_key("mcp")); // Gateway is named "mcp"
    assert!(!spin_config.component.contains_key("ftl-mcp-gateway"));
    assert!(spin_config.component.contains_key("my-tool"));
}
