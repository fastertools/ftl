use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RegistryType {
    Ghcr,
    Docker,
    Ecr,
    Custom,
}

impl fmt::Display for RegistryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryType::Ghcr => write!(f, "ghcr"),
            RegistryType::Docker => write!(f, "docker"),
            RegistryType::Ecr => write!(f, "ecr"),
            RegistryType::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for RegistryType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ghcr" => Ok(RegistryType::Ghcr),
            "docker" => Ok(RegistryType::Docker),
            "ecr" => Ok(RegistryType::Ecr),
            "custom" => Ok(RegistryType::Custom),
            _ => anyhow::bail!("Unknown registry type: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub registry_type: RegistryType,
    pub enabled: bool,
    pub priority: u32,
    #[serde(default)]
    pub config: JsonValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_url: Option<String>,
}

impl RegistryConfig {
    /// Create a new GHCR registry configuration
    pub fn new_ghcr(name: String, organization: String) -> Self {
        Self {
            name,
            registry_type: RegistryType::Ghcr,
            enabled: true,
            priority: 1,
            config: serde_json::json!({
                "organization": organization
            }),
            display_url: Some(format!("https://github.com/orgs/{}/packages", organization)),
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
    pub fn new_custom(name: String, url_pattern: String, auth_type: Option<String>) -> Self {
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
        self.config.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Check if this registry requires authentication
    #[allow(dead_code)]
    pub fn requires_auth(&self) -> bool {
        match self.registry_type {
            RegistryType::Ghcr => false, // Public GHCR repos don't require auth
            RegistryType::Docker => false, // Public Docker Hub doesn't require auth
            RegistryType::Ecr => true, // ECR always requires auth
            RegistryType::Custom => {
                // Check auth_type field
                self.get_config_str("auth_type")
                    .map(|auth| auth != "none")
                    .unwrap_or(false)
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
        assert_eq!("docker".parse::<RegistryType>().unwrap(), RegistryType::Docker);
        assert_eq!("ecr".parse::<RegistryType>().unwrap(), RegistryType::Ecr);
        assert_eq!("custom".parse::<RegistryType>().unwrap(), RegistryType::Custom);
        
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
        let ghcr = RegistryConfig::new_ghcr("my-ghcr".to_string(), "myorg".to_string());
        assert_eq!(ghcr.name, "my-ghcr");
        assert_eq!(ghcr.registry_type, RegistryType::Ghcr);
        assert_eq!(ghcr.get_config_str("organization"), Some("myorg".to_string()));

        let docker = RegistryConfig::new_docker("my-docker".to_string());
        assert_eq!(docker.name, "my-docker");
        assert_eq!(docker.registry_type, RegistryType::Docker);

        let ecr = RegistryConfig::new_ecr(
            "my-ecr".to_string(),
            Some("123456".to_string()),
            Some("us-west-2".to_string())
        );
        assert_eq!(ecr.name, "my-ecr");
        assert_eq!(ecr.registry_type, RegistryType::Ecr);
        assert_eq!(ecr.get_config_str("account_id"), Some("123456".to_string()));
        assert_eq!(ecr.get_config_str("region"), Some("us-west-2".to_string()));

        let custom = RegistryConfig::new_custom(
            "my-custom".to_string(),
            "registry.example.com/{image_name}:latest".to_string(),
            Some("bearer".to_string())
        );
        assert_eq!(custom.name, "my-custom");
        assert_eq!(custom.registry_type, RegistryType::Custom);
        assert_eq!(
            custom.get_config_str("url_pattern"),
            Some("registry.example.com/{image_name}:latest".to_string())
        );
        assert_eq!(custom.get_config_str("auth_type"), Some("bearer".to_string()));
    }

    #[test]
    fn test_requires_auth() {
        let ghcr = RegistryConfig::new_ghcr("ghcr".to_string(), "org".to_string());
        assert!(!ghcr.requires_auth());

        let docker = RegistryConfig::new_docker("docker".to_string());
        assert!(!docker.requires_auth());

        let ecr = RegistryConfig::new_ecr("ecr".to_string(), None, None);
        assert!(ecr.requires_auth());

        let custom_no_auth = RegistryConfig::new_custom(
            "custom".to_string(),
            "registry.example.com/{image_name}".to_string(),
            Some("none".to_string())
        );
        assert!(!custom_no_auth.requires_auth());

        let custom_with_auth = RegistryConfig::new_custom(
            "custom".to_string(),
            "registry.example.com/{image_name}".to_string(),
            Some("bearer".to_string())
        );
        assert!(custom_with_auth.requires_auth());
    }
}