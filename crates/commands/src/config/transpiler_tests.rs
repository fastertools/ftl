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
            deployment: DeploymentConfig::default(),
            gateway: GatewayConfig::default(),
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
            tool_type: "rust".to_string(),
            path: "echo-tool".to_string(),
            build: None,
            allowed_hosts: vec![],
            watch: vec![],
        });
        tools.insert("weather".to_string(), ToolConfig {
            tool_type: "typescript".to_string(),
            path: "weather-ts".to_string(),
            build: Some("npm run build:custom".to_string()),
            allowed_hosts: vec!["https://api.weather.com".to_string()],
            watch: vec!["src/**/*.ts".to_string()],
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
            deployment: DeploymentConfig::default(),
            gateway: GatewayConfig::default(),
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
            deployment: DeploymentConfig::default(),
            gateway: GatewayConfig::default(),
        };
        
        let result = transpile_ftl_to_spin(&config).unwrap();
        
        // Check auth configuration
        assert!(result.contains("auth_enabled = { default = \"true\" }"));
        assert!(result.contains("auth_provider_type = { default = \"authkit\" }"));
        assert!(result.contains("auth_provider_issuer = { default = \"https://my-tenant.authkit.app\" }"));
        assert!(result.contains("auth_provider_audience = { default = \"mcp-api\" }"));
    }
    
    #[test]
    fn test_transpile_prebuilt_tool() {
        let mut tools = HashMap::new();
        tools.insert("tool-calculator".to_string(), ToolConfig {
            tool_type: "prebuilt".to_string(),
            path: "tool-calculator".to_string(),
            build: None,
            allowed_hosts: vec![],
            watch: vec![],
        });
        
        let config = FtlConfig {
            project: ProjectConfig {
                name: "prebuilt-test".to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: vec![],
            },
            auth: AuthConfig::default(),
            tools,
            deployment: DeploymentConfig::default(),
            gateway: GatewayConfig::default(),
        };
        
        let result = transpile_ftl_to_spin(&config).unwrap();
        
        // Check prebuilt tool uses registry source
        assert!(result.contains("[component.tool-calculator]"));
        assert!(result.contains("source = { registry = \"ghcr.io\", package = \"fastertools:calculator\", version = \"latest\" }"));
}