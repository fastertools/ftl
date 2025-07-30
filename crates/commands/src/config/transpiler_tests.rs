//! Tests for ftl.toml to spin.toml transpiler

use super::*;
use crate::config::ftl_config::*;
use std::collections::HashMap;

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
            gateway: GatewayConfig::default(),
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
        assert!(result.contains("[component.mcp]"));
        assert!(result.contains("[component.ftl-mcp-gateway]"));
    }
    
    #[test]
    fn test_transpile_with_tools() {
        let mut tools = HashMap::new();
        tools.insert("echo-tool".to_string(), ToolConfig {
            path: "echo-tool".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                workdir: None,
                watch: vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()],
                env: HashMap::new(),
            },
            allowed_outbound_hosts: vec![],
            variables: HashMap::new(),
        });
        tools.insert("weather".to_string(), ToolConfig {
            path: "weather-ts".to_string(),
            build: BuildConfig {
                command: "npm run build:custom".to_string(),
                workdir: None,
                watch: vec!["src/**/*.ts".to_string()],
                env: HashMap::new(),
            },
            allowed_outbound_hosts: vec!["https://api.weather.com".to_string()],
            variables: HashMap::new(),
        });
        
        let config = FtlConfig {
            project: ProjectConfig {
                name: "test-project".to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: vec!["Test Author <test@example.com>".to_string()],
            },
            auth: AuthConfig::default(),
            tools,
            gateway: GatewayConfig::default(),
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
    }
    
    #[test]
    fn test_transpile_with_variables() {
        let mut tools = HashMap::new();
        let mut variables = HashMap::new();
        variables.insert("API_KEY".to_string(), "test-key".to_string());
        variables.insert("DEBUG".to_string(), "true".to_string());
        
        tools.insert("api-tool".to_string(), ToolConfig {
            path: "api-tool".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                workdir: None,
                watch: vec![],
                env: HashMap::new(),
            },
            allowed_outbound_hosts: vec![],
            variables,
        });
        
        let config = FtlConfig {
            project: ProjectConfig {
                name: "test-project".to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: vec![],
            },
            auth: AuthConfig::default(),
            tools,
            gateway: GatewayConfig::default(),
            variables: HashMap::new(),
        };
        
        let result = transpile_ftl_to_spin(&config).unwrap();
        
        println!("Generated TOML with variables:\n{result}");
        
        // Check that variables are included in the component
        assert!(result.contains("[component.api-tool.variables]"));
        assert!(result.contains("API_KEY = \"test-key\""));
        assert!(result.contains("DEBUG = \"true\""));
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
                provider: "authkit".to_string(),
                issuer: "https://my-tenant.authkit.app".to_string(),
                audience: "mcp-api".to_string(),
                oidc: None,
            },
            tools: HashMap::new(),
            gateway: GatewayConfig::default(),
            variables: HashMap::new(),
        };
        
        let result = transpile_ftl_to_spin(&config).unwrap();
        
        // Check auth configuration
        assert!(result.contains("auth_enabled = { default = \"true\" }"));
        assert!(result.contains("auth_provider_type = { default = \"authkit\" }"));
        assert!(result.contains("auth_provider_issuer = { default = \"https://my-tenant.authkit.app\" }"));
        assert!(result.contains("auth_provider_audience = { default = \"mcp-api\" }"));
    }
    
    #[test]
    fn test_transpile_with_application_variables() {
        let mut app_vars = HashMap::new();
        app_vars.insert("api_token".to_string(), ApplicationVariable::Required { required: true });
        app_vars.insert("api_url".to_string(), ApplicationVariable::Default { default: "https://api.example.com".to_string() });
        
        let mut tools = HashMap::new();
        let mut tool_vars = HashMap::new();
        tool_vars.insert("token".to_string(), "{{ api_token }}".to_string());
        tool_vars.insert("url".to_string(), "{{ api_url }}".to_string());
        tool_vars.insert("version".to_string(), "v1".to_string());
        
        tools.insert("api-consumer".to_string(), ToolConfig {
            path: "api-consumer".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                workdir: None,
                watch: vec![],
                env: HashMap::new(),
            },
            allowed_outbound_hosts: vec!["{{ api_url }}".to_string()],
            variables: tool_vars,
        });
        
        let config = FtlConfig {
            project: ProjectConfig {
                name: "test-project".to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: vec![],
            },
            auth: AuthConfig::default(),
            tools,
            gateway: GatewayConfig::default(),
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
    }
    
    #[test]
    fn test_transpile_simple_variable_reference() {
        // Test the exact scenario we just tested - a simple app variable with tool reference
        let mut app_vars = HashMap::new();
        app_vars.insert("foo_user".to_string(), ApplicationVariable::Default { default: "foo guy".to_string() });
        
        let mut tools = HashMap::new();
        let mut tool_vars = HashMap::new();
        tool_vars.insert("user".to_string(), "{{ foo_user }}".to_string());
        
        tools.insert("hello".to_string(), ToolConfig {
            path: "hello".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                workdir: None,
                watch: vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()],
                env: HashMap::new(),
            },
            allowed_outbound_hosts: vec![],
            variables: tool_vars,
        });
        
        let config = FtlConfig {
            project: ProjectConfig {
                name: "vartest".to_string(),
                version: "0.1.0".to_string(),
                description: "FTL MCP server for hosting MCP tools".to_string(),
                authors: vec![],
            },
            auth: AuthConfig::default(),
            tools,
            gateway: GatewayConfig::default(),
            variables: app_vars,
        };
        
        let result = transpile_ftl_to_spin(&config).unwrap();
        
        // Check application variable
        assert!(result.contains("[variables]"));
        assert!(result.contains("foo_user = { default = \"foo guy\" }"));
        
        // Check component variable reference
        assert!(result.contains("[component.hello.variables]"));
        assert!(result.contains("user = \"{{ foo_user }}\""));
    }
    
    #[test]
    fn test_transpile_multiple_variable_types() {
        // Test mixing required and default variables
        let mut app_vars = HashMap::new();
        app_vars.insert("required_secret".to_string(), ApplicationVariable::Required { required: true });
        app_vars.insert("optional_setting".to_string(), ApplicationVariable::Default { default: "default_value".to_string() });
        app_vars.insert("another_required".to_string(), ApplicationVariable::Required { required: true });
        
        let config = FtlConfig {
            project: ProjectConfig {
                name: "test-project".to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: vec![],
            },
            auth: AuthConfig::default(),
            tools: HashMap::new(),
            gateway: GatewayConfig::default(),
            variables: app_vars,
        };
        
        let result = transpile_ftl_to_spin(&config).unwrap();
        
        // All variables should be in the [variables] section
        assert!(result.contains("required_secret = { required = true }"));
        assert!(result.contains("optional_setting = { default = \"default_value\" }"));
        assert!(result.contains("another_required = { required = true }"));
    }
    
    #[test]
    fn test_transpile_empty_variables() {
        // Test that empty variables are handled correctly
        let config = FtlConfig {
            project: ProjectConfig {
                name: "test-project".to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: vec![],
            },
            auth: AuthConfig::default(),
            tools: HashMap::new(),
            gateway: GatewayConfig::default(),
            variables: HashMap::new(),
        };
        
        let result = transpile_ftl_to_spin(&config).unwrap();
        
        // Should still have [variables] section
        assert!(result.contains("[variables]"));
        // Should have default auth variables but no custom application variables
        assert!(result.contains("auth_enabled = { default = \"false\" }"));
        // Should not have any custom application variables
        assert!(!result.contains("foo_user"));
        assert!(!result.contains("api_token"));
    }
    
    #[test]
    fn test_transpile_variable_edge_cases() {
        // Test edge cases like special characters in variable names/values
        let mut app_vars = HashMap::new();
        app_vars.insert("MULTI_WORD_VAR".to_string(), ApplicationVariable::Default { default: "value with spaces".to_string() });
        app_vars.insert("var-with-dashes".to_string(), ApplicationVariable::Default { default: "dash-value".to_string() });
        app_vars.insert("var_with_underscores".to_string(), ApplicationVariable::Required { required: true });
        
        let mut tools = HashMap::new();
        let mut tool_vars = HashMap::new();
        // Test that tool can reference these variables
        tool_vars.insert("config1".to_string(), "{{ MULTI_WORD_VAR }}".to_string());
        tool_vars.insert("config2".to_string(), "{{ var-with-dashes }}".to_string());
        tool_vars.insert("config3".to_string(), "{{ var_with_underscores }}".to_string());
        tool_vars.insert("literal".to_string(), "not a template".to_string());
        
        tools.insert("edge-case-tool".to_string(), ToolConfig {
            path: "edge-tool".to_string(),
            build: BuildConfig {
                command: "cargo build --target wasm32-wasip1 --release".to_string(),
                workdir: None,
                watch: vec![],
                env: HashMap::new(),
            },
            allowed_outbound_hosts: vec![],
            variables: tool_vars,
        });
        
        let config = FtlConfig {
            project: ProjectConfig {
                name: "test-project".to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: vec![],
            },
            auth: AuthConfig::default(),
            tools,
            gateway: GatewayConfig::default(),
            variables: app_vars,
        };
        
        let result = transpile_ftl_to_spin(&config).unwrap();
        
        // Check app variables
        assert!(result.contains("MULTI_WORD_VAR = { default = \"value with spaces\" }"));
        assert!(result.contains("var-with-dashes = { default = \"dash-value\" }"));
        assert!(result.contains("var_with_underscores = { required = true }"));
        
        // Check component variables
        assert!(result.contains("config1 = \"{{ MULTI_WORD_VAR }}\""));
        assert!(result.contains("config2 = \"{{ var-with-dashes }}\""));
        assert!(result.contains("config3 = \"{{ var_with_underscores }}\""));
        assert!(result.contains("literal = \"not a template\""));
    }
