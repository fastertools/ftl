//! FTL configuration file format (ftl.toml)
//!
//! This module defines the simplified configuration format for FTL projects,
//! which gets transpiled to spin.toml when needed.

use anyhow::{Context, Result};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure for ftl.toml
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FtlConfig {
    /// Project metadata
    #[garde(dive)]
    pub project: ProjectConfig,

    /// Authorization configuration
    #[serde(default)]
    #[garde(dive)]
    pub auth: AuthConfig,

    /// Tool definitions
    #[serde(default)]
    #[garde(custom(validate_tools))]
    pub tools: HashMap<String, ToolConfig>,

    /// Gateway component configuration
    #[serde(default)]
    #[garde(dive)]
    pub gateway: GatewayConfig,

    /// Application-level variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
    pub variables: HashMap<String, ApplicationVariable>,
}

/// Application-level variable configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApplicationVariable {
    /// Variable with a default value
    Default {
        /// The default value for the variable
        default: String,
    },
    /// Required variable that must be provided at runtime
    Required {
        /// Whether the variable is required (should always be true)
        required: bool,
    },
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProjectConfig {
    /// Project name
    #[garde(length(min = 1))]
    #[garde(pattern(r"^[a-zA-Z][a-zA-Z0-9_-]*$"))]
    pub name: String,

    /// Project version
    #[serde(default = "default_version")]
    #[garde(length(min = 1))]
    pub version: String,

    /// Project description
    #[serde(default)]
    #[garde(skip)]
    pub description: String,

    /// Project authors
    #[serde(default)]
    #[garde(skip)]
    pub authors: Vec<String>,
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[garde(allow_unvalidated)]
pub struct AuthConfig {
    /// Whether authorization is enabled
    #[serde(default)]
    pub enabled: bool,

    /// `AuthKit` configuration (mutually exclusive with oidc)
    #[serde(default)]
    #[garde(dive)]
    pub authkit: Option<AuthKitConfig>,

    /// OIDC configuration (mutually exclusive with authkit)
    #[serde(default)]
    #[garde(dive)]
    pub oidc: Option<OidcConfig>,
}

/// AuthKit-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuthKitConfig {
    /// `AuthKit` issuer URL (e.g., `<https://my-tenant.authkit.app>`)
    #[garde(length(min = 1))]
    pub issuer: String,

    /// API audience
    #[serde(default)]
    #[garde(skip)]
    pub audience: String,
}

/// OIDC-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OidcConfig {
    /// OIDC issuer URL
    #[garde(length(min = 1))]
    pub issuer: String,

    /// API audience
    #[serde(default)]
    #[garde(skip)]
    pub audience: String,

    /// Provider name (e.g., "auth0", "okta")
    #[garde(length(min = 1))]
    pub provider_name: String,

    /// JWKS URI
    #[garde(length(min = 1))]
    pub jwks_uri: String,

    /// Authorization endpoint
    #[garde(length(min = 1))]
    pub authorize_endpoint: String,

    /// Token endpoint
    #[garde(length(min = 1))]
    pub token_endpoint: String,

    /// User info endpoint (optional)
    #[serde(default)]
    #[garde(skip)]
    pub userinfo_endpoint: String,

    /// Allowed domains (comma-separated)
    #[serde(default)]
    #[garde(skip)]
    pub allowed_domains: String,
}

/// Deployment configuration for a tool
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeployConfig {
    /// Build profile to use for deployment (e.g., "release", "production")
    #[garde(length(min = 1))]
    pub profile: String,

    /// Optional custom name suffix for the deployed tool
    /// The full name will be {project-name}-{name}
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub name: Option<String>,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ToolConfig {
    /// Path to tool directory relative to project root
    /// Defaults to the tool name if not specified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub path: Option<String>,

    /// Path to the WASM file produced by the build
    #[garde(length(min = 1))]
    pub wasm: String,

    /// Build configuration
    #[garde(dive)]
    pub build: BuildConfig,

    /// Build profiles (optional, for advanced multi-profile builds)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub profiles: Option<BuildProfiles>,

    /// Up configuration for development mode
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub up: Option<UpConfig>,

    /// Deployment configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub deploy: Option<DeployConfig>,

    /// Allowed outbound hosts for the tool
    #[serde(default)]
    #[garde(skip)]
    pub allowed_outbound_hosts: Vec<String>,

    /// Variables to pass to the tool at runtime
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
    pub variables: HashMap<String, String>,
}

/// Build profiles for a tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildProfiles {
    /// Build profiles mapped by name
    #[serde(flatten)]
    pub profiles: HashMap<String, BuildProfile>,
}

/// A single build profile
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BuildProfile {
    /// Build command to execute
    #[garde(length(min = 1))]
    pub command: String,

    /// Paths to watch for changes in development mode
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(skip)]
    pub watch: Vec<String>,

    /// Environment variables to set during build
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
    pub env: HashMap<String, String>,
}

/// Up configuration for development mode
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpConfig {
    /// Build profile to use for 'ftl up'
    #[garde(length(min = 1))]
    pub profile: String,
}

/// Build configuration for a tool
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BuildConfig {
    /// Build command to execute
    #[garde(length(min = 1))]
    pub command: String,

    /// Paths to watch for changes in development mode
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[garde(skip)]
    pub watch: Vec<String>,

    /// Environment variables to set during build
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
    pub env: HashMap<String, String>,
}

/// Gateway component configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct GatewayConfig {
    /// Gateway component version
    #[serde(default = "default_gateway_version")]
    pub version: String,

    /// MCP authorizer component version
    #[serde(default = "default_authorizer_version")]
    pub authorizer_version: String,

    /// Whether to validate tool arguments
    #[serde(default = "default_true")]
    pub validate_arguments: bool,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_gateway_version() -> String {
    "0.0.9".to_string()
}

fn default_authorizer_version() -> String {
    "0.0.9".to_string()
}

const fn default_true() -> bool {
    true
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            version: default_gateway_version(),
            authorizer_version: default_authorizer_version(),
            validate_arguments: default_true(),
        }
    }
}

// Custom validation functions for garde
#[allow(clippy::trivially_copy_pass_by_ref)]
fn validate_tools(tools: &HashMap<String, ToolConfig>, _ctx: &()) -> garde::Result {
    for (name, tool) in tools {
        // Validate tool names
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(garde::Error::new(format!(
                "Tool name '{name}' can only contain alphanumeric characters, hyphens, and underscores"
            )));
        }
        if !name.chars().next().is_some_and(char::is_alphabetic) {
            return Err(garde::Error::new(format!(
                "Tool name '{name}' must start with a letter"
            )));
        }

        // Validate each tool
        tool.validate()
            .map_err(|e| garde::Error::new(format!("Tool '{name}': {e}")))?;
    }
    Ok(())
}

impl AuthConfig {
    /// Get the provider type as a string
    pub const fn provider_type(&self) -> &str {
        if self.authkit.is_some() {
            "authkit"
        } else if self.oidc.is_some() {
            "oidc"
        } else {
            ""
        }
    }

    /// Get the issuer URL
    pub fn issuer(&self) -> &str {
        if let Some(authkit) = &self.authkit {
            &authkit.issuer
        } else if let Some(oidc) = &self.oidc {
            &oidc.issuer
        } else {
            ""
        }
    }

    /// Get the audience
    pub fn audience(&self) -> &str {
        if let Some(authkit) = &self.authkit {
            &authkit.audience
        } else if let Some(oidc) = &self.oidc {
            &oidc.audience
        } else {
            ""
        }
    }
}

impl ToolConfig {
    /// Get the effective path for the tool (uses tool name if path not specified)
    pub fn get_path(&self, tool_name: &str) -> String {
        self.path.clone().unwrap_or_else(|| tool_name.to_string())
    }

    /// Get the build command for a specific profile
    pub fn get_build_command(&self, profile: Option<&str>) -> &str {
        if let Some(profile_name) = profile {
            if let Some(profiles) = &self.profiles {
                if let Some(profile) = profiles.profiles.get(profile_name) {
                    return &profile.command;
                }
            }
        }
        // Fall back to default build command
        &self.build.command
    }

    /// Get the build configuration for a specific profile
    pub fn get_build_config(&self, profile: Option<&str>) -> BuildProfile {
        if let Some(profile_name) = profile {
            if let Some(profiles) = &self.profiles {
                if let Some(profile) = profiles.profiles.get(profile_name) {
                    return profile.clone();
                }
            }
        }
        // Fall back to default build config
        BuildProfile {
            command: self.build.command.clone(),
            watch: self.build.watch.clone(),
            env: self.build.env.clone(),
        }
    }

    /// Get watch paths for a specific profile
    pub fn get_watch_paths(&self, profile: Option<&str>) -> Vec<String> {
        if let Some(profile_name) = profile {
            if let Some(profiles) = &self.profiles {
                if let Some(profile) = profiles.profiles.get(profile_name) {
                    return profile.watch.clone();
                }
            }
        }
        // Fall back to default watch paths
        self.build.watch.clone()
    }

    /// Get the profile to use for 'ftl up'
    pub fn get_up_profile(&self) -> Option<&str> {
        self.up.as_ref().map(|up| up.profile.as_str())
    }
}

impl FtlConfig {
    /// Load FTL configuration from a TOML string
    pub fn parse(content: &str) -> Result<Self> {
        let config: Self = toml::from_str(content).context("Failed to parse ftl.toml")?;

        // Use garde validation
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("Validation error: {}", e))?;

        // Additional auth validation
        if config.auth.enabled {
            match (&config.auth.authkit, &config.auth.oidc) {
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either 'authkit' or 'oidc' configuration must be provided when auth is enabled"
                    ));
                }
                (Some(_), Some(_)) => {
                    return Err(anyhow::anyhow!(
                        "Only one of 'authkit' or 'oidc' can be configured, not both"
                    ));
                }
                _ => {} // One provider configured, which is correct
            }
        }

        Ok(config)
    }

    /// Load FTL configuration from a file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).context("Failed to read ftl.toml")?;
        Self::parse(&content)
    }

    /// Convert to TOML string
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize ftl.toml")
    }

    /// Get a list of all tool component names
    pub fn tool_components(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let content = r#"
[project]
name = "my-project"
"#;

        let config = FtlConfig::parse(content).unwrap();
        assert_eq!(config.project.name, "my-project");
        assert_eq!(config.project.version, "0.1.0");
        assert!(!config.auth.enabled);
    }

    #[test]
    fn test_validation_errors() {
        // Test invalid project name
        let content = r#"
[project]
name = "123-invalid"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Validation error"));

        // Test empty wasm path
        let content = r#"
[project]
name = "valid-name"

[tools.my-tool]
path = "my-tool"
wasm = ""

[tools.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());

        // Test auth validation - auth enabled but no provider configured
        let content = r#"
[project]
name = "valid-name"

[auth]
enabled = true
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Either 'authkit' or 'oidc' configuration must be provided")
        );
    }

    #[test]
    fn test_project_name_validation() {
        // Valid project names
        let valid_names = vec![
            "myproject",
            "my-project",
            "my_project",
            "MyProject",
            "project123",
            "a",
            "A",
        ];

        for name in valid_names {
            let content = format!(
                r#"
[project]
name = "{name}"
"#
            );
            let result = FtlConfig::parse(&content);
            assert!(result.is_ok(), "Project name '{name}' should be valid");
        }

        // Invalid project names
        let invalid_names = vec![
            "",           // empty
            "123project", // starts with number
            "-project",   // starts with hyphen
            "_project",   // starts with underscore
            "my project", // contains space
            "my.project", // contains dot
            "my@project", // contains special char
            "my/project", // contains slash
        ];

        for name in invalid_names {
            let content = format!(
                r#"
[project]
name = "{name}"
"#
            );
            let result = FtlConfig::parse(&content);
            assert!(result.is_err(), "Project name '{name}' should be invalid");
        }
    }

    #[test]
    fn test_tool_name_validation() {
        // Valid tool names
        let valid_names = vec![
            "mytool",
            "my-tool",
            "my_tool",
            "MyTool",
            "tool123",
            "tool-123_test",
        ];

        for name in valid_names {
            let content = format!(
                r#"
[project]
name = "test-project"

[tools.{name}]
path = "tool-path"
wasm = "tool-path/output.wasm"

[tools.{name}.build]
command = "cargo build --target wasm32-wasip1 --release"
"#
            );
            let result = FtlConfig::parse(&content);
            assert!(result.is_ok(), "Tool name '{name}' should be valid");
        }

        // Invalid tool names
        let invalid_names = vec![
            "123tool", // starts with number
            "-tool",   // starts with hyphen
            "_tool",   // starts with underscore
            "my tool", // contains space
            "my.tool", // contains dot
            "my@tool", // contains special char
            "my/tool", // contains slash
            "my$tool", // contains dollar sign
        ];

        for name in invalid_names {
            let content = format!(
                r#"
[project]
name = "test-project"

[tools."{name}"]
path = "tool-path"
wasm = "tool-path/output.wasm"

[tools."{name}".build]
command = "cargo build --target wasm32-wasip1 --release"
"#
            );
            let result = FtlConfig::parse(&content);
            assert!(result.is_err(), "Tool name '{name}' should be invalid");
        }
    }

    #[test]
    fn test_auth_validation() {
        // Test auth disabled - should pass with no provider config
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = false
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_ok());

        // Test auth enabled with authkit - should pass
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true

[auth.authkit]
issuer = "https://my-tenant.authkit.app"
audience = "my-api"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.auth.provider_type(), "authkit");
        assert_eq!(config.auth.issuer(), "https://my-tenant.authkit.app");
        assert_eq!(config.auth.audience(), "my-api");

        // Test auth enabled with oidc - should pass
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true

[auth.oidc]
issuer = "https://auth.example.com"
audience = "api"
provider_name = "okta"
jwks_uri = "https://auth.example.com/.well-known/jwks.json"
authorize_endpoint = "https://auth.example.com/authorize"
token_endpoint = "https://auth.example.com/token"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.auth.provider_type(), "oidc");
        assert_eq!(config.auth.issuer(), "https://auth.example.com");

        // Test auth enabled with no provider - should fail
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Either 'authkit' or 'oidc' configuration must be provided")
        );

        // Test auth enabled with both providers - should fail
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true

[auth.authkit]
issuer = "https://my-tenant.authkit.app"
audience = "my-api"

[auth.oidc]
issuer = "https://auth.example.com"
audience = "api"
provider_name = "okta"
jwks_uri = "https://auth.example.com/.well-known/jwks.json"
authorize_endpoint = "https://auth.example.com/authorize"
token_endpoint = "https://auth.example.com/token"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Only one of 'authkit' or 'oidc' can be configured")
        );

        // Test authkit with missing required field
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true

[auth.authkit]
issuer = ""
audience = "my-api"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_oidc_config_validation() {
        // Valid OIDC config
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true

[auth.oidc]
issuer = "https://example.com"
audience = "my-api"
provider_name = "okta"
jwks_uri = "https://example.com/.well-known/jwks.json"
authorize_endpoint = "https://example.com/oauth/authorize"
token_endpoint = "https://example.com/oauth/token"
userinfo_endpoint = "https://example.com/oauth/userinfo"
allowed_domains = "example.com,test.com"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.auth.oidc.is_some());
        let oidc = config.auth.oidc.as_ref().unwrap();
        assert_eq!(oidc.provider_name, "okta");
        assert_eq!(oidc.jwks_uri, "https://example.com/.well-known/jwks.json");
        assert_eq!(oidc.userinfo_endpoint, "https://example.com/oauth/userinfo");
        assert_eq!(oidc.allowed_domains, "example.com,test.com");
    }

    #[test]
    fn test_empty_required_fields() {
        // Test empty project name
        let content = r#"
[project]
name = ""
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());

        // Test empty tool path
        let content = r#"
[project]
name = "test-project"

[tools.mytool]
type = "rust"
path = ""
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_values() {
        let content = r#"
[project]
name = "test-project"
"#;
        let config = FtlConfig::parse(content).unwrap();

        // Check project defaults
        assert_eq!(config.project.version, "0.1.0");
        assert_eq!(config.project.description, "");
        assert_eq!(config.project.authors.len(), 0);

        // Check auth defaults
        assert!(!config.auth.enabled);
        assert_eq!(config.auth.provider_type(), "");

        // Check gateway defaults
        assert_eq!(config.gateway.version, "0.0.9");
        assert_eq!(config.gateway.authorizer_version, "0.0.9");
        assert!(config.gateway.validate_arguments);
    }

    #[test]
    fn test_parse_full_config() {
        let content = r#"
[project]
name = "my-project"
version = "1.0.0"
description = "My FTL project"
authors = ["John Doe <john@example.com>"]

[auth]
enabled = true

[auth.authkit]
issuer = "https://my-tenant.authkit.app"
audience = "mcp-api"

[tools.echo]
path = "echo-rs"
wasm = "echo-rs/target/wasm32-wasip1/release/echo_rs.wasm"
allowed_outbound_hosts = []

[tools.echo.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml"]

[tools.weather]
path = "weather-ts"
wasm = "weather-ts/dist/weather.wasm"
allowed_outbound_hosts = ["https://api.weather.com"]

[tools.weather.build]
command = "npm run build:custom"
watch = ["src/**/*.ts", "package.json"]

"#;

        let config = FtlConfig::parse(content).unwrap();
        assert_eq!(config.project.name, "my-project");
        assert_eq!(config.project.version, "1.0.0");
        assert!(config.auth.enabled);
        assert_eq!(config.auth.provider_type(), "authkit");
        assert_eq!(config.tools.len(), 2);
        assert_eq!(
            config.tools["echo"].build.command,
            "cargo build --target wasm32-wasip1 --release"
        );
        assert_eq!(
            config.tools["weather"].build.command,
            "npm run build:custom"
        );
    }

    #[test]
    fn test_tool_components_method() {
        let content = r#"
[project]
name = "test-project"

[tools.tool1]
path = "tool1"
wasm = "tool1/target/wasm32-wasip1/release/tool1.wasm"

[tools.tool1.build]
command = "cargo build --target wasm32-wasip1 --release"

[tools.tool2]
path = "tool2"
wasm = "tool2/dist/tool2.wasm"

[tools.tool2.build]
command = "npm run build"

[tools.tool3]
path = "tool3"
wasm = "tool3/dist/tool3.wasm"

[tools.tool3.build]
command = "npm run build"
"#;
        let config = FtlConfig::parse(content).unwrap();
        let components = config.tool_components();
        assert_eq!(components.len(), 3);
        assert!(components.contains(&"tool1".to_string()));
        assert!(components.contains(&"tool2".to_string()));
        assert!(components.contains(&"tool3".to_string()));
    }

    #[test]
    fn test_to_toml_string() {
        let config = FtlConfig {
            project: ProjectConfig {
                name: "test-project".to_string(),
                version: "1.0.0".to_string(),
                description: "Test description".to_string(),
                authors: vec!["Test Author <test@example.com>".to_string()],
            },
            auth: AuthConfig::default(),
            tools: HashMap::new(),
            gateway: GatewayConfig::default(),
            variables: HashMap::new(),
        };

        let toml_string = config.to_toml_string().unwrap();
        assert!(toml_string.contains("[project]"));
        assert!(toml_string.contains("name = \"test-project\""));
        assert!(toml_string.contains("version = \"1.0.0\""));
        assert!(toml_string.contains("description = \"Test description\""));
    }

    #[test]
    fn test_skip_empty_build_fields() {
        let mut tools = HashMap::new();
        tools.insert(
            "test-tool".to_string(),
            ToolConfig {
                path: Some("test-tool".to_string()),
                wasm: "test-tool/target/wasm32-wasip1/release/test_tool.wasm".to_string(),
                build: BuildConfig {
                    command: "cargo build --target wasm32-wasip1 --release".to_string(),
                    watch: vec!["src/**/*.rs".to_string()],
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

        let toml_string = config.to_toml_string().unwrap();
        println!("Generated TOML:\n{toml_string}");

        // Should NOT contain empty env section
        assert!(!toml_string.contains("[tools.test-tool.build.env]"));
        // Should contain watch array
        assert!(toml_string.contains("watch = ["));
    }

    #[test]
    fn test_application_variables() {
        let content = r#"
[project]
name = "test-project"

[variables]
api_token = { required = true }
api_url = { default = "https://api.example.com" }
debug = { default = "false" }

[tools.my-tool]
path = "my-tool"
wasm = "my-tool/target/wasm32-wasip1/release/my_tool.wasm"

[tools.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"

[tools.my-tool.variables]
token = "{{ api_token }}"
url = "{{ api_url }}"
debug_mode = "{{ debug }}"
static_var = "static-value"
"#;
        let config = FtlConfig::parse(content).unwrap();

        // Check application variables
        assert_eq!(config.variables.len(), 3);

        // Check required variable
        if let Some(ApplicationVariable::Required { required }) = config.variables.get("api_token")
        {
            assert!(required);
        } else {
            panic!("api_token should be a required variable");
        }

        // Check default variables
        if let Some(ApplicationVariable::Default { default }) = config.variables.get("api_url") {
            assert_eq!(default, "https://api.example.com");
        } else {
            panic!("api_url should have a default value");
        }

        // Check tool variables with templates
        let tool = &config.tools["my-tool"];
        assert_eq!(
            tool.variables.get("token"),
            Some(&"{{ api_token }}".to_string())
        );
        assert_eq!(
            tool.variables.get("url"),
            Some(&"{{ api_url }}".to_string())
        );
        assert_eq!(
            tool.variables.get("static_var"),
            Some(&"static-value".to_string())
        );
    }

    #[test]
    fn test_from_file() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("ftl.toml");

        let content = r#"
[project]
name = "file-test-project"
version = "2.0.0"
"#;

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let config = FtlConfig::from_file(&file_path).unwrap();
        assert_eq!(config.project.name, "file-test-project");
        assert_eq!(config.project.version, "2.0.0");
    }

    #[test]
    fn test_custom_validation_edge_cases() {
        // Test tool with environment variables
        let content = r#"
[project]
name = "test-project"

[tools.custom-build]
path = "custom"
wasm = "custom/dist/custom.wasm"

[tools.custom-build.build]
command = "npm run build:special"
env = { NODE_ENV = "production", CUSTOM_VAR = "value" }
"#;
        let config = FtlConfig::parse(content).unwrap();
        assert_eq!(
            config.tools["custom-build"].build.command,
            "npm run build:special"
        );
        assert_eq!(
            config.tools["custom-build"].build.env.get("NODE_ENV"),
            Some(&"production".to_string())
        );

        // Test tool with watch patterns
        let content = r#"
[project]
name = "test-project"

[tools.watch-tool]
path = "watch"
wasm = "watch/target/wasm32-wasip1/release/watch_tool.wasm"

[tools.watch-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["**/*.rs", "Cargo.toml"]
"#;
        let config = FtlConfig::parse(content).unwrap();
        assert_eq!(config.tools["watch-tool"].build.watch.len(), 2);

        // Test multiple validation errors at once
        let content = r#"
[project]
name = "123invalid"

[auth]
enabled = true
provider = ""

[tools."bad@name"]
path = ""
wasm = "output.wasm"

[tools."bad@name".build]
command = ""
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Should contain at least one validation error
        assert!(error_msg.contains("Validation error"));
    }

    #[test]
    fn test_build_configuration() {
        // Test minimal build config
        let content = r#"
[project]
name = "test-project"

[tools.minimal]
path = "minimal-tool"
wasm = "minimal-tool/output.wasm"

[tools.minimal.build]
command = "make build"
"#;
        let config = FtlConfig::parse(content).unwrap();
        assert_eq!(config.tools["minimal"].build.command, "make build");
        assert!(config.tools["minimal"].build.watch.is_empty());
        assert!(config.tools["minimal"].build.env.is_empty());

        // Test full build config
        let content = r#"
[project]
name = "test-project"

[tools.full]
path = "full-tool"
wasm = "full-tool/target/wasm32-wasip1/release/full_tool.wasm"

[tools.full.build]
command = "cargo build --release"
watch = ["src/**/*.rs", "Cargo.toml", "build.rs"]

[tools.full.build.env]
RUSTFLAGS = "-C target-cpu=native"
CARGO_BUILD_JOBS = "4"
"#;
        let config = FtlConfig::parse(content).unwrap();
        let build = &config.tools["full"].build;
        assert_eq!(build.command, "cargo build --release");
        assert_eq!(build.watch.len(), 3);
        assert_eq!(build.watch[2], "build.rs");
        assert_eq!(
            build.env.get("RUSTFLAGS"),
            Some(&"-C target-cpu=native".to_string())
        );
        assert_eq!(build.env.get("CARGO_BUILD_JOBS"), Some(&"4".to_string()));

        // Test missing build section validation
        let content = r#"
[project]
name = "test-project"

[tools.no-build]
path = "tool"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        // The error will be a TOML parse error about missing required field
    }

    #[test]
    fn test_build_profiles() {
        // Test tool with multiple build profiles
        let content = r#"
[project]
name = "test-project"

[tools.myapp]
wasm = "myapp/target/wasm32-wasip1/release/myapp.wasm"

[tools.myapp.build]
command = "cargo build --target wasm32-wasip1"

[tools.myapp.profiles.dev]
command = "cargo build --target wasm32-wasip1"
watch = ["src/**/*.rs", "Cargo.toml"]
env = { RUST_LOG = "debug" }

[tools.myapp.profiles.release]
command = "cargo build --target wasm32-wasip1 --release"
env = { RUST_LOG = "warn" }

[tools.myapp.profiles.production]
command = "cargo build --target wasm32-wasip1 --release"
env = { RUST_LOG = "error", RUST_BACKTRACE = "1" }

[tools.myapp.up]
profile = "dev"

[tools.myapp.deploy]
profile = "production"
"#;
        let config = FtlConfig::parse(content).unwrap();

        let tool = &config.tools["myapp"];

        // Check profiles exist
        assert!(tool.profiles.is_some());
        let profiles = tool.profiles.as_ref().unwrap();
        assert_eq!(profiles.profiles.len(), 3);

        // Check dev profile
        let dev = &profiles.profiles["dev"];
        assert_eq!(dev.command, "cargo build --target wasm32-wasip1");
        assert_eq!(dev.watch.len(), 2);
        assert_eq!(dev.env.get("RUST_LOG"), Some(&"debug".to_string()));

        // Check release profile
        let release = &profiles.profiles["release"];
        assert_eq!(
            release.command,
            "cargo build --target wasm32-wasip1 --release"
        );
        assert_eq!(release.env.get("RUST_LOG"), Some(&"warn".to_string()));

        // Check production profile
        let prod = &profiles.profiles["production"];
        assert_eq!(prod.env.get("RUST_BACKTRACE"), Some(&"1".to_string()));

        // Check up configuration
        assert_eq!(tool.get_up_profile(), Some("dev"));

        // Check deploy configuration
        assert_eq!(tool.deploy.as_ref().unwrap().profile, "production");

        // Test getting build commands for different profiles
        assert_eq!(
            tool.get_build_command(Some("dev")),
            "cargo build --target wasm32-wasip1"
        );
        assert_eq!(
            tool.get_build_command(Some("release")),
            "cargo build --target wasm32-wasip1 --release"
        );
        assert_eq!(
            tool.get_build_command(None),
            "cargo build --target wasm32-wasip1"
        ); // default
    }

    #[test]
    fn test_deploy_configuration() {
        // Test tool with deploy config
        let content = r#"
[project]
name = "test-project"

[tools.calc]
path = "calc"
wasm = "calc/target/wasm32-wasip1/release/calc.wasm"

[tools.calc.build]
command = "cargo build --target wasm32-wasip1 --release"

[tools.calc.deploy]
profile = "release"
name = "calculator"

[tools.weather]
path = "weather"
wasm = "weather/dist/weather.wasm"

[tools.weather.build]
command = "npm run build"

[tools.weather.deploy]
profile = "production"
"#;
        let config = FtlConfig::parse(content).unwrap();

        // Check calc tool with custom name
        assert_eq!(
            config.tools["calc"].deploy.as_ref().unwrap().profile,
            "release"
        );
        assert_eq!(
            config.tools["calc"].deploy.as_ref().unwrap().name,
            Some("calculator".to_string())
        );

        // Check weather tool without custom name
        assert_eq!(
            config.tools["weather"].deploy.as_ref().unwrap().profile,
            "production"
        );
        assert!(
            config.tools["weather"]
                .deploy
                .as_ref()
                .unwrap()
                .name
                .is_none()
        );
    }
}
