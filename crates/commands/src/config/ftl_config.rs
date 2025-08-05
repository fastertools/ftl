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

    /// OIDC configuration (optional)
    #[serde(default)]
    #[garde(dive)]
    pub oidc: Option<OidcConfig>,

    /// Tool definitions
    #[serde(default)]
    #[garde(custom(validate_tools))]
    pub tools: HashMap<String, ToolConfig>,

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

    /// Access control mode: "public" or "private"
    /// - public: No authentication required (default)
    /// - private: Authentication required
    #[serde(default = "default_access_control")]
    #[garde(custom(validate_access_control))]
    pub access_control: String,
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
            .context("FTL configuration validation failed")?;
        Ok(config)
    }

    /// Serialize FTL configuration to TOML string
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize FTL configuration")
    }

    /// Get the list of tool component names
    pub fn tool_components(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Determine if authentication is enabled
    pub fn is_auth_enabled(&self) -> bool {
        self.project.access_control == "private"
    }

    /// Determine the auth provider type
    pub fn auth_provider_type(&self) -> &str {
        if self.is_auth_enabled() {
            "jwt" // Always use JWT for both OIDC and built-in AuthKit
        } else {
            ""
        }
    }

    /// Get the issuer URL
    pub fn auth_issuer(&self) -> &str {
        if let Some(oidc) = &self.oidc {
            &oidc.issuer
        } else if self.is_auth_enabled() {
            // Use FTL's built-in AuthKit
            "https://divine-lion-50-staging.authkit.app"
        } else {
            ""
        }
    }

    /// Get the audience
    pub fn auth_audience(&self) -> &str {
        if let Some(oidc) = &self.oidc {
            &oidc.audience
        } else {
            ""
        }
    }

    /// Get required scopes
    pub fn auth_required_scopes(&self) -> &str {
        if let Some(oidc) = &self.oidc {
            &oidc.required_scopes
        } else {
            ""
        }
    }
}

impl ToolConfig {
    /// Get the path to the tool directory
    pub fn get_path(&self, tool_name: &str) -> String {
        self.path.clone().unwrap_or_else(|| tool_name.to_string())
    }
}

// Default value functions
fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_access_control() -> String {
    "public".to_string()
}

fn default_gateway() -> String {
    "ghcr.io/fastertools/mcp-gateway:0.0.10".to_string()
}

fn default_authorizer() -> String {
    "ghcr.io/fastertools/mcp-authorizer:0.0.10".to_string()
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
fn validate_tools(tools: &HashMap<String, ToolConfig>, _: &()) -> garde::Result {
    for (name, config) in tools {
        config
            .validate()
            .map_err(|e| garde::Error::new(e.to_string()))?;

        // Ensure tool name follows naming conventions
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(garde::Error::new(format!(
                "Tool name '{name}' contains invalid characters. Use only alphanumeric, dash, or underscore."
            )));
        }
    }
    Ok(())
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn validate_access_control(value: &str, _: &()) -> garde::Result {
    match value {
        "public" | "private" => Ok(()),
        _ => Err(garde::Error::new(format!(
            "Invalid access_control '{value}'. Must be 'public' or 'private'."
        ))),
    }
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
        assert_eq!(ftl_config.project.access_control, "public");
        assert!(!ftl_config.is_auth_enabled());
    }

    #[test]
    fn test_private_without_oidc() {
        let config = r#"
[project]
name = "test-project"
access_control = "private"
"#;
        let ftl_config = FtlConfig::parse(config).unwrap();
        assert!(ftl_config.is_auth_enabled());
        assert_eq!(
            ftl_config.auth_issuer(),
            "https://divine-lion-50-staging.authkit.app"
        );
        assert_eq!(ftl_config.auth_provider_type(), "jwt");
    }

    #[test]
    fn test_private_with_oidc() {
        let config = r#"
[project]
name = "test-project"
access_control = "private"

[oidc]
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
    fn test_invalid_access_control() {
        let config = r#"
[project]
name = "test-project"
access_control = "custom"
"#;
        let result = FtlConfig::parse(config);
        assert!(result.is_err());
        // The validation error message is in the general format
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("validation failed")
        );
    }
}
