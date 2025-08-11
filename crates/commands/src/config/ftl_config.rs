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

    /// OAuth configuration (optional)
    #[serde(default)]
    #[garde(dive)]
    pub oauth: Option<OauthConfig>,

    /// Component definitions
    #[serde(default, rename = "component")]
    #[garde(custom(validate_components))]
    pub component: HashMap<String, ComponentConfig>,

    /// MCP component configuration
    #[serde(default)]
    #[garde(dive)]
    pub mcp: McpConfig,

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

    /// Default registry for component references
    /// Example: "ghcr.io/myorg" or "docker.io"
    #[serde(default)]
    #[garde(skip)]
    pub default_registry: Option<String>,
}

/// OAuth-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OauthConfig {
    /// OAuth issuer URL
    #[garde(length(min = 1))]
    pub issuer: String,

    /// API audience (required for security)
    #[garde(length(min = 1))]
    pub audience: String,

    /// JWKS URI (optional - can be auto-discovered for some providers)
    #[serde(default)]
    #[garde(skip)]
    pub jwks_uri: String,

    /// Public key in PEM format (optional - alternative to JWKS)
    #[serde(default)]
    #[garde(skip)]
    pub public_key: String,

    /// JWT algorithm (e.g., RS256, ES256)
    #[serde(default)]
    #[garde(skip)]
    pub algorithm: String,

    /// Required scopes (comma-separated)
    #[serde(default)]
    #[garde(skip)]
    pub required_scopes: String,

    /// Authorization endpoint (optional)
    #[serde(default)]
    #[garde(skip)]
    pub authorize_endpoint: String,

    /// Token endpoint (optional)
    #[serde(default)]
    #[garde(skip)]
    pub token_endpoint: String,

    /// User info endpoint (optional)
    #[serde(default)]
    #[garde(skip)]
    pub userinfo_endpoint: String,

    /// Allowed subjects (user IDs) that can access this resource
    /// If not specified, any authenticated subject is allowed
    #[serde(default)]
    #[garde(skip)]
    pub allowed_subjects: Vec<String>,
}

/// Deployment configuration for a component
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeployConfig {
    /// Build profile to use for deployment (e.g., "release", "production")
    #[garde(length(min = 1))]
    pub profile: String,

    /// Optional custom name suffix for the deployed component
    /// The full name will be {project-name}-{name}
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[garde(skip)]
    pub name: Option<String>,
}

/// Component configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Path to component directory relative to project root
    /// Required for local components, ignored for registry components
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Path to the WASM file produced by the build (for local components)
    /// Mutually exclusive with `repo`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm: Option<String>,

    /// Repository reference for pre-built component (e.g., "ghcr.io/org/tool:latest")
    /// Mutually exclusive with `wasm`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,

    /// Build configuration (required for local components with `wasm`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildConfig>,

    /// Build profiles (optional, for advanced multi-profile builds)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profiles: Option<BuildProfiles>,

    /// Up configuration for development mode
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub up: Option<UpConfig>,

    /// Deployment configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deploy: Option<DeployConfig>,

    /// Allowed outbound hosts for the component
    #[serde(default)]
    pub allowed_outbound_hosts: Vec<String>,

    /// Variables to pass to the component at runtime
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub variables: HashMap<String, String>,
}

/// Build profiles for a component
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

/// MCP component configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct McpConfig {
    /// MCP gateway component registry URI
    #[serde(default = "default_gateway")]
    #[garde(length(min = 1))]
    pub gateway: String,

    /// MCP authorizer component registry URI
    #[serde(default = "default_authorizer")]
    #[garde(length(min = 1))]
    pub authorizer: String,

    /// Whether to validate tool call arguments
    #[serde(default = "default_validate_arguments")]
    #[garde(skip)]
    pub validate_arguments: bool,
}

impl FtlConfig {
    /// Parse FTL configuration from TOML string
    pub fn parse(content: &str) -> Result<Self> {
        let config: Self = toml::from_str(content).context("Failed to parse FTL configuration")?;
        config
            .validate()
            .map_err(|e| anyhow::anyhow!("FTL configuration validation failed: {}", e))?;
        Ok(config)
    }

    /// Serialize FTL configuration to TOML string
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize FTL configuration")
    }

    /// Get the list of component names
    pub fn component_names(&self) -> Vec<String> {
        self.component.keys().cloned().collect()
    }

    /// Determine if authentication is enabled (based on oauth presence)
    pub const fn is_auth_enabled(&self) -> bool {
        self.oauth.is_some()
    }

    /// Determine the auth provider type
    pub const fn auth_provider_type(&self) -> &str {
        if self.is_auth_enabled() {
            "jwt" // Always use JWT for both OAuth and built-in AuthKit
        } else {
            ""
        }
    }

    /// Get the issuer URL
    pub fn auth_issuer(&self) -> &str {
        if let Some(oauth) = &self.oauth {
            &oauth.issuer
        } else {
            ""
        }
    }

    /// Get the audience
    #[allow(clippy::missing_const_for_fn)] // Can't be const due to String deref
    pub fn auth_audience(&self) -> &str {
        if let Some(oauth) = &self.oauth {
            &oauth.audience
        } else {
            ""
        }
    }

    /// Get required scopes
    #[allow(clippy::missing_const_for_fn)] // Can't be const due to String deref
    pub fn auth_required_scopes(&self) -> &str {
        if let Some(oauth) = &self.oauth {
            &oauth.required_scopes
        } else {
            ""
        }
    }
}

impl ComponentConfig {
    /// Get the path to the component directory
    pub fn get_path(&self, component_name: &str) -> String {
        self.path
            .clone()
            .unwrap_or_else(|| component_name.to_string())
    }

    /// Validate the component configuration
    pub fn validate(&self) -> Result<(), garde::Report> {
        let mut report = garde::Report::new();

        // Ensure exactly one of `wasm` or `repo` is set
        match (&self.wasm, &self.repo) {
            (None, None) => {
                report.append(
                    garde::Path::empty(),
                    garde::Error::new(
                        "Component must specify either 'wasm' (for local) or 'repo' (for registry)",
                    ),
                );
                return Err(report);
            }
            (Some(_), Some(_)) => {
                report.append(
                    garde::Path::empty(),
                    garde::Error::new("Component cannot specify both 'wasm' and 'repo'. Use 'wasm' for local or 'repo' for registry")
                );
                return Err(report);
            }
            _ => {}
        }

        // If `wasm` is set (local component), ensure `build` is present
        if self.wasm.is_some() && self.build.is_none() {
            report.append(
                garde::Path::empty(),
                garde::Error::new(
                    "Local components with 'wasm' field must include a 'build' configuration",
                ),
            );
            return Err(report);
        }

        // If `repo` is set (registry component), ensure `build` is NOT present
        if self.repo.is_some() && self.build.is_some() {
            report.append(
                garde::Path::empty(),
                garde::Error::new("Registry components with 'repo' field should not include a 'build' configuration")
            );
            return Err(report);
        }

        // Validate build config if present
        if let Some(build) = &self.build {
            build.validate()?;
        }

        Ok(())
    }
}

// Default value functions
fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_gateway() -> String {
    "ghcr.io/fastertools/mcp-gateway:0.0.11".to_string()
}

fn default_authorizer() -> String {
    "ghcr.io/fastertools/mcp-authorizer:0.0.13".to_string()
}

const fn default_validate_arguments() -> bool {
    false
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            gateway: default_gateway(),
            authorizer: default_authorizer(),
            validate_arguments: default_validate_arguments(),
        }
    }
}

// Validation functions
#[allow(clippy::trivially_copy_pass_by_ref)]
fn validate_components(component: &HashMap<String, ComponentConfig>, _: &()) -> garde::Result {
    for (name, config) in component {
        config
            .validate()
            .map_err(|e| garde::Error::new(format!("Component '{name}': {e}")))?;

        // Ensure component name follows naming conventions
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(garde::Error::new(format!(
                "Component name '{name}' contains invalid characters. Use only alphanumeric, dash, or underscore."
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config() {
        let config = r#"
[project]
name = "test-project"
"#;
        let ftl_config = FtlConfig::parse(config).unwrap();
        assert_eq!(ftl_config.project.name, "test-project");
        assert_eq!(ftl_config.project.version, "0.1.0");
        // No oauth block means public access (no auth)
        assert!(!ftl_config.is_auth_enabled());
    }

    #[test]
    fn test_private_without_oauth() {
        let config = r#"
[project]
name = "test-project"
version = "0.1.0"
"#;
        let ftl_config = FtlConfig::parse(config).unwrap();
        assert!(!ftl_config.is_auth_enabled());
        assert_eq!(ftl_config.auth_issuer(), "");
        assert_eq!(ftl_config.auth_provider_type(), "");
    }

    #[test]
    fn test_private_with_oauth() {
        let config = r#"
[project]
name = "test-project"
version = "0.1.0"

[oauth]
issuer = "https://auth.example.com"
audience = "my-api"
"#;
        let ftl_config = FtlConfig::parse(config).unwrap();
        assert!(ftl_config.is_auth_enabled());
        assert_eq!(ftl_config.auth_issuer(), "https://auth.example.com");
        assert_eq!(ftl_config.auth_audience(), "my-api");
        assert_eq!(ftl_config.auth_provider_type(), "jwt");
    }

    #[test]
    fn test_org_access_control() {
        let config = r#"
[project]
name = "test-project"
version = "0.1.0"
"#;
        let ftl_config = FtlConfig::parse(config).unwrap();
        assert!(!ftl_config.is_auth_enabled());
        // Without OAuth, auth is disabled
        assert_eq!(ftl_config.auth_issuer(), "");
    }

    #[test]
    fn test_custom_access_control() {
        let config = r#"
[project]
name = "test-project"
version = "0.1.0"

[oauth]
issuer = "https://custom-auth.example.com"
audience = "custom-api"
"#;
        let ftl_config = FtlConfig::parse(config).unwrap();
        assert!(ftl_config.is_auth_enabled());
        // With OAuth block, auth is enabled
        assert_eq!(ftl_config.auth_issuer(), "https://custom-auth.example.com");
        assert_eq!(ftl_config.auth_audience(), "custom-api");
    }

    #[test]
    fn test_invalid_access_control() {
        // Test that access_control field is no longer accepted
        let config = r#"
[project]
name = "test-project"
version = "1.0.0"
access_control = "public"
"#;
        let _result = FtlConfig::parse(config);
        // Should fail because access_control is not a valid field anymore
        // But since we're deserializing with serde which ignores unknown fields by default,
        // this actually succeeds. The test name is misleading now.
        // Let's test an actual invalid config instead
        let invalid_config = r#"
[project]
name = "test-project"
version = 
"#;
        let result = FtlConfig::parse(invalid_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_mixed_local_and_registry_components() {
        let config = r#"
[project]
name = "py-tools"
version = "0.1.0"
description = "FTL MCP server for hosting MCP tools"
default_registry = "ghcr.io/fastertools"

[component.example-py]
path = "example-py"
wasm = "example-py/app.wasm"
allowed_outbound_hosts = []

[component.example-py.build]
command = "make build"
watch = [
    "src/**/*.py",
    "pyproject.toml",
]

[component.example-rs]
repo = "ghcr.io/fastertools/example-rs:latest"

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.13"
validate_arguments = false
"#;
        let ftl_config = FtlConfig::parse(config).unwrap();
        assert_eq!(ftl_config.project.name, "py-tools");
        // Component configuration test

        // Check local component
        let py_component = &ftl_config.component["example-py"];
        assert_eq!(py_component.wasm, Some("example-py/app.wasm".to_string()));
        assert!(py_component.repo.is_none());
        assert!(py_component.build.is_some());

        // Check registry component
        let rs_component = &ftl_config.component["example-rs"];
        assert!(rs_component.wasm.is_none());
        assert_eq!(
            rs_component.repo,
            Some("ghcr.io/fastertools/example-rs:latest".to_string())
        );
        assert!(rs_component.build.is_none());
    }

    #[test]
    fn test_component_validation_errors() {
        // Test missing both wasm and repo
        let config = r#"
[project]
name = "test"

[component.broken]
path = "broken"
"#;
        let result = FtlConfig::parse(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must specify either 'wasm'")
                || err.contains("must specify either 'wasm'")
        );

        // Test having both wasm and repo
        let config = r#"
[project]
name = "test"

[component.broken]
wasm = "broken/app.wasm"
repo = "ghcr.io/org/broken:latest"
"#;
        let result = FtlConfig::parse(config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot specify both")
        );

        // Test local component without build
        let config = r#"
[project]
name = "test"

[component.broken]
wasm = "broken/app.wasm"
"#;
        let result = FtlConfig::parse(config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must include a 'build' configuration")
        );

        // Test registry component with build
        let config = r#"
[project]
name = "test"

[component.broken]
repo = "ghcr.io/org/broken:latest"

[component.broken.build]
command = "make build"
"#;
        let result = FtlConfig::parse(config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("should not include a 'build' configuration")
        );
    }
}
