//! Validation tests for Spin configuration edge cases and error scenarios
//!
//! These tests ensure proper validation and error handling in the Spin
//! configuration schema using garde.

use super::*;
use garde::Validate;

#[test]
fn test_invalid_manifest_version() {
    let mut config = SpinConfig::new("test-app".to_string());

    // Test version too low
    config.spin_manifest_version = 1;
    assert!(config.validate().is_err());

    // Test version too high
    config.spin_manifest_version = 3;
    assert!(config.validate().is_err());

    // Test correct version
    config.spin_manifest_version = 2;
    assert!(config.validate().is_ok());
}

#[test]
fn test_invalid_application_names() {
    // Test various invalid application names
    let invalid_names = vec![
        "",          // empty
        "123app",    // starts with number
        "-app",      // starts with hyphen
        "_app",      // starts with underscore
        "my app",    // contains space
        "my.app",    // contains dot
        "my@app",    // contains special char
        "my/app",    // contains slash
        "my\\app",   // contains backslash
        "my:app",    // contains colon
        "my;app",    // contains semicolon
        "my,app",    // contains comma
        "my|app",    // contains pipe
        "my?app",    // contains question mark
        "my*app",    // contains asterisk
        "my[app]",   // contains brackets
        "my{app}",   // contains braces
        "my(app)",   // contains parentheses
        "my<app>",   // contains angle brackets
        "my'app'",   // contains quotes
        "my\"app\"", // contains double quotes
        "my`app`",   // contains backticks
        "my~app",    // contains tilde
        "my!app",    // contains exclamation
        "my#app",    // contains hash
        "my$app",    // contains dollar
        "my%app",    // contains percent
        "my^app",    // contains caret
        "my&app",    // contains ampersand
        "my=app",    // contains equals
        "my+app",    // contains plus
    ];

    for name in invalid_names {
        let config = SpinConfig::new(name.to_string());
        let result = config.validate();
        assert!(
            result.is_err(),
            "Application name '{name}' should be invalid but validation passed"
        );
    }

    // Test valid application names
    let valid_names = vec![
        "a",
        "A",
        "myapp",
        "MyApp",
        "my-app",
        "my_app",
        "my-app_123",
        "app123",
        "APP-123_test",
        "a1b2c3",
        "test-123-app_v2",
    ];

    for name in valid_names {
        let config = SpinConfig::new(name.to_string());
        let result = config.validate();
        assert!(
            result.is_ok(),
            "Application name '{}' should be valid but validation failed: {:?}",
            name,
            result.err()
        );
    }
}

#[test]
fn test_invalid_version_format() {
    let mut config = SpinConfig::new("test-app".to_string());

    // Test invalid version formats
    let invalid_versions = vec![
        "",           // empty
        "1",          // missing minor and patch
        "1.0",        // missing patch
        "1.0.0.0",    // too many parts
        "v1.0.0",     // has prefix
        "1.0.0-beta", // has suffix
        "1.a.0",      // non-numeric
        "1.0.a",      // non-numeric
        "a.0.0",      // non-numeric
        "1.0.0 ",     // trailing space
        " 1.0.0",     // leading space
        "1.0.0\n",    // contains newline
        "1.0.0\t",    // contains tab
        "1..0",       // double dot
        ".1.0.0",     // leading dot
        "1.0.0.",     // trailing dot
        "1.0.0.0.0",  // way too many parts
        "1_0_0",      // wrong separator
        "1-0-0",      // wrong separator
        "1,0,0",      // wrong separator
        "1/0/0",      // wrong separator
    ];

    for version in invalid_versions {
        config.application.version = version.to_string();
        let result = config.validate();
        assert!(
            result.is_err(),
            "Version '{version}' should be invalid but validation passed"
        );
    }

    // Test valid version formats
    let valid_versions = vec![
        "0.0.0",
        "0.1.0",
        "1.0.0",
        "1.2.3",
        "10.20.30",
        "999.999.999",
        "2.0.0",
        "0.0.1",
    ];

    for version in valid_versions {
        config.application.version = version.to_string();
        let result = config.validate();
        assert!(
            result.is_ok(),
            "Version '{version}' should be valid but validation failed"
        );
    }
}

#[test]
fn test_component_name_validation() {
    let mut config = SpinConfig::new("test-app".to_string());

    // Test invalid component names
    let invalid_names = vec![
        "my_component", // underscore not allowed
        "my.component", // dot not allowed
        "my component", // space not allowed
        "123component", // starts with number
        "-component",   // starts with hyphen
        "my/component", // slash not allowed
        "my@component", // special char not allowed
        "",             // empty
    ];

    for name in invalid_names {
        config.component.clear();
        config.component.insert(
            name.to_string(),
            ComponentConfig {
                description: String::new(),
                source: ComponentSource::Local("test.wasm".to_string()),
                files: Vec::new(),
                exclude_files: Vec::new(),
                allowed_outbound_hosts: Vec::new(),
                key_value_stores: Vec::new(),
                environment: HashMap::new(),
                build: None,
                variables: HashMap::new(),
                dependencies_inherit_configuration: false,
                dependencies: HashMap::new(),
            },
        );

        let result = config.validate();
        assert!(
            result.is_err(),
            "Component name '{name}' should be invalid but validation passed"
        );
    }

    // Test valid component names
    let valid_names = vec![
        "my-component",
        "component",
        "a",
        "component-123",
        "test-tool-v2",
        "my-long-component-name",
    ];

    for name in valid_names {
        config.component.clear();
        config.component.insert(
            name.to_string(),
            ComponentConfig {
                description: String::new(),
                source: ComponentSource::Local("test.wasm".to_string()),
                files: Vec::new(),
                exclude_files: Vec::new(),
                allowed_outbound_hosts: Vec::new(),
                key_value_stores: Vec::new(),
                environment: HashMap::new(),
                build: None,
                variables: HashMap::new(),
                dependencies_inherit_configuration: false,
                dependencies: HashMap::new(),
            },
        );

        let result = config.validate();
        assert!(
            result.is_ok(),
            "Component name '{name}' should be valid but validation failed"
        );
    }
}

#[test]
fn test_component_source_validation() {
    // Test empty local source
    let source = ComponentSource::Local(String::new());
    assert!(source.validate().is_err());

    // Test valid local source
    let source = ComponentSource::Local("my-app.wasm".to_string());
    assert!(source.validate().is_ok());

    // Test invalid remote sources
    let invalid_remotes = vec![
        ("", "sha256:abcdef"),                           // empty URL
        ("ftp://example.com/app.wasm", "sha256:abcdef"), // wrong protocol
        ("http://example.com/app.wasm", ""),             // empty digest
        ("http://example.com/app.wasm", "abcdef"),       // missing sha256: prefix
        ("http://example.com/app.wasm", "md5:abcdef"),   // wrong hash type
        ("://example.com/app.wasm", "sha256:abcdef"),    // malformed URL
        ("http://", "sha256:abcdef"),                    // incomplete URL
    ];

    for (url, digest) in invalid_remotes {
        let source = ComponentSource::Remote {
            url: url.to_string(),
            digest: digest.to_string(),
        };
        assert!(
            source.validate().is_err(),
            "Remote source with URL '{url}' and digest '{digest}' should be invalid"
        );
    }

    // Test valid remote sources
    let valid_remotes = vec![
        ("http://example.com/app.wasm", "sha256:abcdef123456"),
        ("https://example.com/app.wasm", "sha256:0123456789abcdef"),
        (
            "http://example.com:8080/path/to/app.wasm",
            "sha256:fedcba987654321",
        ),
        (
            "https://cdn.example.com/v1/app.wasm?token=abc",
            "sha256:123456",
        ),
    ];

    for (url, digest) in valid_remotes {
        let source = ComponentSource::Remote {
            url: url.to_string(),
            digest: digest.to_string(),
        };
        assert!(
            source.validate().is_ok(),
            "Remote source with URL '{url}' and digest '{digest}' should be valid"
        );
    }

    // Test invalid registry sources
    let invalid_registries = vec![
        ("", "myorg:myapp", "1.0.0"),        // empty registry
        ("ghcr.io", "", "1.0.0"),            // empty package
        ("ghcr.io", "myorg/myapp", "1.0.0"), // wrong package format (slash instead of colon)
        ("ghcr.io", "myapp", "1.0.0"),       // missing namespace in package
        ("ghcr.io", "myorg:myapp", ""),      // empty version
    ];

    for (registry, package, version) in invalid_registries {
        let source = ComponentSource::Registry {
            registry: registry.to_string(),
            package: package.to_string(),
            version: version.to_string(),
        };
        assert!(
            source.validate().is_err(),
            "Registry source '{registry}' / '{package}' / '{version}' should be invalid"
        );
    }

    // Test valid registry sources
    let valid_registries = vec![
        ("ghcr.io", "myorg:myapp", "1.0.0"),
        ("docker.io", "namespace:component", "v2.1.0-beta"),
        ("registry.example.com:5000", "org:suborg:tool", "latest"),
        ("ttl.sh", "user:app", "1.0.0"),
        ("localhost:5000", "test:app", "dev"),
    ];

    for (registry, package, version) in valid_registries {
        let source = ComponentSource::Registry {
            registry: registry.to_string(),
            package: package.to_string(),
            version: version.to_string(),
        };
        assert!(
            source.validate().is_ok(),
            "Registry source '{registry}' / '{package}' / '{version}' should be valid"
        );
    }
}

#[test]
fn test_variable_validation() {
    // Test required variable with required: false
    let mut vars = HashMap::new();
    vars.insert(
        "bad_required".to_string(),
        SpinVariable::Required { required: false },
    );
    assert!(validate_variables(&vars, &()).is_err());

    // Test secret required variable with required: false
    vars.clear();
    vars.insert(
        "bad_secret_required".to_string(),
        SpinVariable::SecretRequired {
            required: false,
            secret: true,
        },
    );
    assert!(validate_variables(&vars, &()).is_err());

    // Test valid variables
    vars.clear();
    vars.insert(
        "default_var".to_string(),
        SpinVariable::Default {
            default: "value".to_string(),
        },
    );
    vars.insert(
        "required_var".to_string(),
        SpinVariable::Required { required: true },
    );
    vars.insert(
        "secret_default".to_string(),
        SpinVariable::SecretDefault {
            default: "secret_value".to_string(),
            secret: true,
        },
    );
    vars.insert(
        "secret_required".to_string(),
        SpinVariable::SecretRequired {
            required: true,
            secret: true,
        },
    );
    assert!(validate_variables(&vars, &()).is_ok());

    // Test empty variable name
    vars.clear();
    vars.insert(
        String::new(),
        SpinVariable::Default {
            default: "value".to_string(),
        },
    );
    assert!(validate_variables(&vars, &()).is_err());
}

#[test]
fn test_outbound_hosts_validation() {
    // Test empty host
    let hosts = vec![String::new()];
    assert!(validate_outbound_hosts(&hosts, &()).is_err());

    // Test host without scheme
    let hosts = vec!["example.com".to_string()];
    assert!(validate_outbound_hosts(&hosts, &()).is_err());

    // Test host with just scheme
    let hosts = vec!["http://".to_string()];
    assert!(validate_outbound_hosts(&hosts, &()).is_ok()); // This is actually valid as a wildcard

    // Test various valid hosts
    let hosts = vec![
        "http://example.com".to_string(),
        "https://api.example.com:8080".to_string(),
        "redis://localhost:6379".to_string(),
        "mysql://db.example.com".to_string(),
        "postgres://db.example.com:5432".to_string(),
        "*://example.com:*".to_string(),
        "http://127.0.0.1:*".to_string(),
        "https://*.example.com".to_string(),
        "http://*".to_string(),
    ];
    assert!(validate_outbound_hosts(&hosts, &()).is_ok());
}

#[test]
fn test_build_config_validation() {
    // Test empty command
    let build = ComponentBuildConfig {
        command: String::new(),
        workdir: String::new(),
        watch: Vec::new(),
        environment: HashMap::new(),
    };
    assert!(build.validate().is_err());

    // Test valid build config
    let build = ComponentBuildConfig {
        command: "cargo build --target wasm32-wasip1 --release".to_string(),
        workdir: "/path/to/project".to_string(),
        watch: vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()],
        environment: HashMap::from([
            ("RUST_LOG".to_string(), "debug".to_string()),
            ("CARGO_BUILD_JOBS".to_string(), "4".to_string()),
        ]),
    };
    assert!(build.validate().is_ok());
}

#[test]
fn test_route_config_validation() {
    // Test empty path (which is actually valid for root route)
    let route = RouteConfig::Path(String::new());
    assert!(route.validate().is_ok());

    // Test various valid routes
    let valid_routes = vec![
        RouteConfig::Path("/".to_string()),
        RouteConfig::Path("/api".to_string()),
        RouteConfig::Path("/api/v1".to_string()),
        RouteConfig::Path("/api/...".to_string()),
        RouteConfig::Path("/users/:id".to_string()),
        RouteConfig::Path("/.well-known/oauth".to_string()),
        RouteConfig::Private { private: true },
    ];

    for route in valid_routes {
        assert!(route.validate().is_ok());
    }

    // Test invalid private route
    let route = RouteConfig::Private { private: false };
    assert!(route.validate().is_err());
}

#[test]
fn test_redis_trigger_validation() {
    // Test empty channel
    let trigger = RedisTrigger {
        address: Some("redis://localhost:6379".to_string()),
        channel: String::new(),
        component: "my-component".to_string(),
    };
    assert!(trigger.validate().is_err());

    // Test empty component
    let trigger = RedisTrigger {
        address: Some("redis://localhost:6379".to_string()),
        channel: "my-channel".to_string(),
        component: String::new(),
    };
    assert!(trigger.validate().is_err());

    // Test valid trigger
    let trigger = RedisTrigger {
        address: Some("redis://localhost:6379".to_string()),
        channel: "my-channel".to_string(),
        component: "my-component".to_string(),
    };
    assert!(trigger.validate().is_ok());

    // Test without address (should use default)
    let trigger = RedisTrigger {
        address: None,
        channel: "my-channel".to_string(),
        component: "my-component".to_string(),
    };
    assert!(trigger.validate().is_ok());
}

#[test]
fn test_http_trigger_validation() {
    // Test empty component
    let trigger = HttpTrigger {
        route: RouteConfig::Path("/test".to_string()),
        component: String::new(),
        executor: None,
    };
    assert!(trigger.validate().is_err());

    // Test valid triggers
    let valid_triggers = vec![
        HttpTrigger {
            route: RouteConfig::Path("/api".to_string()),
            component: "my-component".to_string(),
            executor: None,
        },
        HttpTrigger {
            route: RouteConfig::Path("/api".to_string()),
            component: "my-component".to_string(),
            executor: Some(ExecutorConfig::Spin),
        },
        HttpTrigger {
            route: RouteConfig::Path("/api".to_string()),
            component: "my-component".to_string(),
            executor: Some(ExecutorConfig::Wagi {
                argv: Some("${SCRIPT_NAME} ${ARGS}".to_string()),
                entrypoint: Some("_start".to_string()),
            }),
        },
    ];

    for trigger in valid_triggers {
        assert!(trigger.validate().is_ok());
    }
}

#[test]
fn test_full_config_validation() {
    // Create a complex configuration with multiple validation points
    let mut config = SpinConfig::new("complex-app".to_string());
    config.application.version = "1.2.3".to_string();
    config.application.description = "A complex application".to_string();
    config.application.authors = vec!["Author 1".to_string(), "Author 2".to_string()];

    // Add variables
    config.variables.insert(
        "api_key".to_string(),
        SpinVariable::Required { required: true },
    );
    config.variables.insert(
        "debug".to_string(),
        SpinVariable::Default {
            default: "false".to_string(),
        },
    );

    // Add components
    config.component.insert(
        "api-handler".to_string(),
        ComponentConfig {
            description: "Handles API requests".to_string(),
            source: ComponentSource::Local("api/handler.wasm".to_string()),
            files: vec![
                FileMount::Pattern("static/**/*".to_string()),
                FileMount::Mapping {
                    source: "config".to_string(),
                    destination: "/app/config".to_string(),
                },
            ],
            exclude_files: vec!["**/*.test.js".to_string()],
            allowed_outbound_hosts: vec![
                "https://api.example.com".to_string(),
                "redis://cache.example.com:6379".to_string(),
            ],
            key_value_stores: vec!["default".to_string()],
            environment: HashMap::from([
                ("NODE_ENV".to_string(), "production".to_string()),
                ("API_VERSION".to_string(), "v2".to_string()),
            ]),
            build: Some(ComponentBuildConfig {
                command: "npm run build".to_string(),
                workdir: "api".to_string(),
                watch: vec!["src/**/*.ts".to_string(), "package.json".to_string()],
                environment: HashMap::from([("BUILD_TARGET".to_string(), "wasm".to_string())]),
            }),
            variables: HashMap::from([
                ("api_key".to_string(), "{{ api_key }}".to_string()),
                ("debug_mode".to_string(), "{{ debug }}".to_string()),
            ]),
            dependencies_inherit_configuration: false,
            dependencies: HashMap::from([(
                "utils:logger".to_string(),
                ComponentDependency {
                    registry: "ghcr.io".to_string(),
                    package: "myorg:logger".to_string(),
                    version: "1.0.0".to_string(),
                },
            )]),
        },
    );

    // This should pass validation
    assert!(config.validate().is_ok());

    // Now break various parts and ensure validation fails

    // Break manifest version
    config.spin_manifest_version = 3;
    assert!(config.validate().is_err());
    config.spin_manifest_version = 2;

    // Break application name
    config.application.name = "123-invalid".to_string();
    assert!(config.validate().is_err());
    config.application.name = "complex-app".to_string();

    // Break version
    config.application.version = "1.2".to_string();
    assert!(config.validate().is_err());
    config.application.version = "1.2.3".to_string();

    // Break variable
    config.variables.insert(
        "broken".to_string(),
        SpinVariable::Required { required: false },
    );
    assert!(config.validate().is_err());
    config.variables.remove("broken");

    // Break component name
    config.component.insert(
        "invalid_component_name".to_string(),
        ComponentConfig {
            description: String::new(),
            source: ComponentSource::Local("test.wasm".to_string()),
            files: Vec::new(),
            exclude_files: Vec::new(),
            allowed_outbound_hosts: Vec::new(),
            key_value_stores: Vec::new(),
            environment: HashMap::new(),
            build: None,
            variables: HashMap::new(),
            dependencies_inherit_configuration: false,
            dependencies: HashMap::new(),
        },
    );
    assert!(config.validate().is_err());
    config.component.remove("invalid_component_name");

    // Should be valid again
    assert!(config.validate().is_ok());
}

#[test]
fn test_toml_generation_with_triggers() {
    let config = SpinConfig::new("test-app".to_string());

    let triggers = TriggerConfig {
        http: vec![
            HttpTrigger {
                route: RouteConfig::Path("/api".to_string()),
                component: "api-handler".to_string(),
                executor: None,
            },
            HttpTrigger {
                route: RouteConfig::Private { private: true },
                component: "internal".to_string(),
                executor: None,
            },
        ],
        redis: vec![RedisTrigger {
            address: Some("redis://localhost:6379".to_string()),
            channel: "events".to_string(),
            component: "event-handler".to_string(),
        }],
    };

    let toml = config.to_toml_string_with_triggers(&triggers).unwrap();

    // Verify the TOML contains the expected sections
    assert!(toml.contains("spin_manifest_version = 2"));
    assert!(toml.contains("[application]"));
    assert!(toml.contains("[[trigger.http]]"));
    assert!(toml.contains("route = \"/api\""));
    assert!(toml.contains("route = { private = true }"));
    assert!(toml.contains("[[trigger.redis]]"));
    assert!(toml.contains("channel = \"events\""));
}
