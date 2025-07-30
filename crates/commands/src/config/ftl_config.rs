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

    /// Authentication configuration
    #[serde(default)]
    #[garde(dive)]
    #[garde(custom(validate_auth_config))]
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

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[garde(allow_unvalidated)]
pub struct AuthConfig {
    /// Whether authentication is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Authentication provider type ("authkit" or "oidc")
    #[serde(default)]
    pub provider: String,

    /// Provider issuer URL
    #[serde(default)]
    pub issuer: String,

    /// API audience
    #[serde(default)]
    pub audience: String,

    /// OIDC-specific settings
    #[serde(flatten)]
    #[garde(dive)]
    pub oidc: Option<OidcConfig>,
}

/// OIDC-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OidcConfig {
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

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ToolConfig {
    /// Path to tool directory relative to project root
    #[garde(length(min = 1))]
    pub path: String,

    /// Build configuration
    #[garde(dive)]
    pub build: BuildConfig,

    /// Allowed outbound hosts for the tool
    #[serde(default)]
    #[garde(skip)]
    pub allowed_outbound_hosts: Vec<String>,

    /// Variables to pass to the tool at runtime
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[garde(skip)]
    pub variables: HashMap<String, String>,
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
fn validate_auth_config(auth: &AuthConfig, _ctx: &()) -> garde::Result {
    if auth.enabled {
        if auth.provider.is_empty() {
            return Err(garde::Error::new(
                "provider must be specified when auth is enabled",
            ));
        }
        if auth.issuer.is_empty() {
            return Err(garde::Error::new(
                "issuer must be specified when auth is enabled",
            ));
        }
        // audience is now optional
    }
    Ok(())
}

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

impl ToolConfig {
    /// Get the build command
    pub fn get_build_command(&self) -> &str {
        &self.build.command
    }

    /// Get watch paths
    pub fn get_watch_paths(&self) -> &[String] {
        &self.build.watch
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

        // Test empty tool path
        let content = r#"
[project]
name = "valid-name"

[tools.my-tool]
path = ""

[tools.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());

        // Test auth validation
        let content = r#"
[project]
name = "valid-name"

[auth]
enabled = true
provider = ""
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("provider must be specified")
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
        // Test auth disabled - should pass with empty fields
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = false
provider = ""
issuer = ""
audience = ""
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_ok());

        // Test auth enabled with all fields - should pass
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true
provider = "authkit"
issuer = "https://example.com"
audience = "my-api"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_ok());

        // Test auth enabled with missing provider
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true
provider = ""
issuer = "https://example.com"
audience = "my-api"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("provider must be specified")
        );

        // Test auth enabled with missing issuer
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true
provider = "authkit"
issuer = ""
audience = "my-api"
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("issuer must be specified")
        );

        // Test auth enabled with empty audience - should now pass
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true
provider = "authkit"
issuer = "https://example.com"
audience = ""
"#;
        let result = FtlConfig::parse(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_oidc_config_validation() {
        // Valid OIDC config
        let content = r#"
[project]
name = "test-project"

[auth]
enabled = true
provider = "oidc"
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
        let oidc = config.auth.oidc.unwrap();
        assert_eq!(oidc.provider_name, "okta");
        assert_eq!(oidc.jwks_uri, "https://example.com/.well-known/jwks.json");
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
        assert_eq!(config.auth.provider, "");

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
provider = "authkit"
issuer = "https://my-tenant.authkit.app"
audience = "mcp-api"

[tools.echo]
path = "echo-rs"
allowed_outbound_hosts = []

[tools.echo.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml"]

[tools.weather]
path = "weather-ts"
allowed_outbound_hosts = ["https://api.weather.com"]

[tools.weather.build]
command = "npm run build:custom"
watch = ["src/**/*.ts", "package.json"]

"#;

        let config = FtlConfig::parse(content).unwrap();
        assert_eq!(config.project.name, "my-project");
        assert_eq!(config.project.version, "1.0.0");
        assert!(config.auth.enabled);
        assert_eq!(config.auth.provider, "authkit");
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

[tools.tool1.build]
command = "cargo build --target wasm32-wasip1 --release"

[tools.tool2]
path = "tool2"

[tools.tool2.build]
command = "npm run build"

[tools.tool3]
path = "tool3"

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
                path: "test-tool".to_string(),
                build: BuildConfig {
                    command: "cargo build --target wasm32-wasip1 --release".to_string(),
                    watch: vec!["src/**/*.rs".to_string()],
                    env: HashMap::new(),
                },
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
}
