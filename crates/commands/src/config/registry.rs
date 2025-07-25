use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt;

/// Registry type enumeration for different container registries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RegistryType {
    /// GitHub Container Registry (ghcr.io)
    Ghcr,
    /// Docker Hub registry (docker.io)
    Docker,
    /// AWS Elastic Container Registry
    Ecr,
    /// Custom registry with user-defined URL pattern
    Custom,
}

impl fmt::Display for RegistryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ghcr => write!(f, "ghcr"),
            Self::Docker => write!(f, "docker"),
            Self::Ecr => write!(f, "ecr"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for RegistryType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ghcr" => Ok(Self::Ghcr),
            "docker" => Ok(Self::Docker),
            "ecr" => Ok(Self::Ecr),
            "custom" => Ok(Self::Custom),
            _ => anyhow::bail!("Unknown registry type: {}", s),
        }
    }
}

/// Configuration for a container registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Human-readable name for this registry configuration
    pub name: String,
    #[serde(rename = "type")]
    /// Type of registry (GHCR, Docker Hub, ECR, or Custom)
    pub registry_type: RegistryType,
    /// Whether this registry is enabled for use
    pub enabled: bool,
    /// Priority order for registry selection (lower = higher priority)
    pub priority: u32,
    #[serde(default)]
    /// Registry-specific configuration as JSON
    pub config: JsonValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional display URL for this registry
    pub display_url: Option<String>,
}

impl RegistryConfig {
    /// Create a new GHCR registry configuration
    pub fn new_ghcr(name: String, organization: &str) -> Self {
        Self {
            name,
            registry_type: RegistryType::Ghcr,
            enabled: true,
            priority: 1,
            config: serde_json::json!({
                "organization": organization
            }),
            display_url: Some(format!("https://github.com/orgs/{organization}/packages")),
        }
    }

    /// Create a new Docker Hub registry configuration
    pub fn new_docker(name: String) -> Self {
        Self {
            name,
            registry_type: RegistryType::Docker,
            enabled: true,
            priority: 2,
            config: serde_json::json!({}),
            display_url: Some("https://hub.docker.com".to_string()),
        }
    }

    /// Create a new ECR registry configuration
    pub fn new_ecr(name: String, account_id: Option<String>, region: Option<String>) -> Self {
        let mut config = serde_json::json!({});

        if let Some(account) = account_id {
            config["account_id"] = serde_json::Value::String(account);
        }

        if let Some(region) = region {
            config["region"] = serde_json::Value::String(region);
        }

        Self {
            name,
            registry_type: RegistryType::Ecr,
            enabled: true,
            priority: 3,
            config,
            display_url: None, // ECR URLs are account-specific
        }
    }

    /// Create a new custom registry configuration
    pub fn new_custom(name: String, url_pattern: &str, auth_type: Option<String>) -> Self {
        let mut config = serde_json::json!({
            "url_pattern": url_pattern
        });

        if let Some(auth) = auth_type {
            config["auth_type"] = serde_json::Value::String(auth);
        }

        Self {
            name,
            registry_type: RegistryType::Custom,
            enabled: true,
            priority: 4,
            config,
            display_url: None, // Custom registries define their own URLs
        }
    }

    /// Get a configuration value as a string
    pub fn get_config_str(&self, key: &str) -> Option<String> {
        self.config
            .get(key)
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
    }

    /// Check if this registry requires authentication
    #[allow(dead_code)]
    pub fn requires_auth(&self) -> bool {
        match self.registry_type {
            RegistryType::Ghcr | RegistryType::Docker => false, // Public repos don't require auth
            RegistryType::Ecr => true,                          // ECR always requires auth
            RegistryType::Custom => {
                // Check auth_type field
                self.get_config_str("auth_type")
                    .is_some_and(|auth| auth != "none")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_type_parsing() {
        assert_eq!("ghcr".parse::<RegistryType>().unwrap(), RegistryType::Ghcr);
        assert_eq!(
            "docker".parse::<RegistryType>().unwrap(),
            RegistryType::Docker
        );
        assert_eq!("ecr".parse::<RegistryType>().unwrap(), RegistryType::Ecr);
        assert_eq!(
            "custom".parse::<RegistryType>().unwrap(),
            RegistryType::Custom
        );

        assert!("unknown".parse::<RegistryType>().is_err());
    }

    #[test]
    fn test_registry_type_display() {
        assert_eq!(RegistryType::Ghcr.to_string(), "ghcr");
        assert_eq!(RegistryType::Docker.to_string(), "docker");
        assert_eq!(RegistryType::Ecr.to_string(), "ecr");
        assert_eq!(RegistryType::Custom.to_string(), "custom");
    }

    #[test]
    fn test_new_registry_configs() {
        let ghcr = RegistryConfig::new_ghcr("my-ghcr".to_string(), "myorg");
        assert_eq!(ghcr.name, "my-ghcr");
        assert_eq!(ghcr.registry_type, RegistryType::Ghcr);
        assert_eq!(
            ghcr.get_config_str("organization"),
            Some("myorg".to_string())
        );

        let docker = RegistryConfig::new_docker("my-docker".to_string());
        assert_eq!(docker.name, "my-docker");
        assert_eq!(docker.registry_type, RegistryType::Docker);

        let ecr = RegistryConfig::new_ecr(
            "my-ecr".to_string(),
            Some("123456".to_string()),
            Some("us-west-2".to_string()),
        );
        assert_eq!(ecr.name, "my-ecr");
        assert_eq!(ecr.registry_type, RegistryType::Ecr);
        assert_eq!(ecr.get_config_str("account_id"), Some("123456".to_string()));
        assert_eq!(ecr.get_config_str("region"), Some("us-west-2".to_string()));

        let custom = RegistryConfig::new_custom(
            "my-custom".to_string(),
            "registry.example.com/{image_name}:latest",
            Some("bearer".to_string()),
        );
        assert_eq!(custom.name, "my-custom");
        assert_eq!(custom.registry_type, RegistryType::Custom);
        assert_eq!(
            custom.get_config_str("url_pattern"),
            Some("registry.example.com/{image_name}:latest".to_string())
        );
        assert_eq!(
            custom.get_config_str("auth_type"),
            Some("bearer".to_string())
        );
    }

    #[test]
    fn test_requires_auth() {
        let ghcr = RegistryConfig::new_ghcr("ghcr".to_string(), "org");
        assert!(!ghcr.requires_auth());

        let docker = RegistryConfig::new_docker("docker".to_string());
        assert!(!docker.requires_auth());

        let ecr = RegistryConfig::new_ecr("ecr".to_string(), None, None);
        assert!(ecr.requires_auth());

        let custom_no_auth = RegistryConfig::new_custom(
            "custom".to_string(),
            "registry.example.com/{image_name}",
            Some("none".to_string()),
        );
        assert!(!custom_no_auth.requires_auth());

        let custom_with_auth = RegistryConfig::new_custom(
            "custom".to_string(),
            "registry.example.com/{image_name}",
            Some("bearer".to_string()),
        );
        assert!(custom_with_auth.requires_auth());
    }
}
