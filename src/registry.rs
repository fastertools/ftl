use anyhow::{Context, Result};
use std::env;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Registry components for Spin manifest generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistryComponents {
    /// Registry domain (e.g., "docker.io", "ghcr.io")
    pub registry_domain: String,
    /// Package name (e.g., "library/nginx", "fastertools:ftl-tool-add")
    pub package_name: String,
    /// Semantic version (e.g., "1.0.0", "2.1.3-alpha")
    pub version: String,
}

/// Trait for adapting different registry formats
pub trait RegistryAdapter {
    /// Get the full registry URL for a given image name
    fn get_registry_url(&self, image_name: &str) -> String;
    
    /// Get registry components for Spin manifest generation
    async fn get_registry_components(&self, client: &Client, image_name: &str) -> Result<RegistryComponents>;
    
    /// Get a human-readable name for this registry
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    
    /// Verify if an image exists in this registry
    async fn verify_image_exists(&self, client: &Client, image_name: &str) -> Result<bool>;
}

/// Parse image name and tag, defaulting to "latest" if no tag specified
fn parse_image_and_tag(image_name: &str) -> (String, String) {
    if let Some(pos) = image_name.rfind(':') {
        let image = image_name[..pos].to_string();
        let tag = image_name[pos + 1..].to_string();
        (image, tag)
    } else {
        (image_name.to_string(), "latest".to_string())
    }
}

/// Validate and normalize a semantic version
fn validate_and_normalize_semver(version: &str) -> Result<String> {
    use semver::Version;
    
    // Remove common prefixes
    let clean_version = version.trim_start_matches('v');
    
    // Check for common non-semver tags first
    match clean_version {
        "latest" | "main" | "master" | "stable" | "dev" | "edge" => {
            anyhow::bail!("Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)", version)
        }
        _ => {}
    }
    
    // Try to parse as semver
    match Version::parse(clean_version) {
        Ok(v) => Ok(v.to_string()),
        Err(_) => {
            // If it's not valid semver, try to make it valid (only if it looks like a number)
            if clean_version.chars().all(|c| c.is_numeric() || c == '.') {
                if clean_version.matches('.').count() == 0 {
                    // Single number like "1" -> "1.0.0"
                    Ok(format!("{}.0.0", clean_version))
                } else if clean_version.matches('.').count() == 1 {
                    // Two numbers like "1.2" -> "1.2.0"
                    Ok(format!("{}.0", clean_version))
                } else {
                    anyhow::bail!("Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)", version)
                }
            } else {
                anyhow::bail!("Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)", version)
            }
        }
    }
}

/// Resolve "latest" tag to actual semantic version from registry

/// Docker Hub adapter
pub struct DockerHubAdapter;

impl RegistryAdapter for DockerHubAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        // Docker Hub patterns:
        // Official images: docker.io/library/image:tag
        // User images: docker.io/username/image:tag
        
        if image_name.contains('/') {
            // User/organization image
            format!("docker.io/{}", image_name)
        } else {
            // Official image - use library namespace
            format!("docker.io/library/{}", image_name)
        }
    }
    
    async fn get_registry_components(&self, _client: &Client, image_name: &str) -> Result<RegistryComponents> {
        let (image_without_tag, tag) = parse_image_and_tag(image_name);
        
        let package_name = if image_without_tag.contains('/') {
            image_without_tag.to_string()
        } else {
            format!("library/{}", image_without_tag)
        };
        
        let version = if tag == "latest" {
            // For Spin manifests, we need an actual semver, not "latest"
            // In production, this should resolve to actual version from registry
            // For now, return error to force explicit semver usage
            anyhow::bail!("Invalid version tag '{}'. Spin manifests require explicit semantic versions (e.g., 1.0.0)", tag)
        } else {
            validate_and_normalize_semver(&tag)?
        };
        
        Ok(RegistryComponents {
            registry_domain: "docker.io".to_string(),
            package_name,
            version,
        })
    }
    
    fn name(&self) -> &'static str {
        "Docker Hub"
    }
    
    async fn verify_image_exists(&self, client: &Client, image_name: &str) -> Result<bool> {
        // Docker Hub Registry API v2
        let repo_name = if image_name.contains('/') {
            image_name.to_string()
        } else {
            format!("library/{}", image_name)
        };
        
        let url = format!("https://registry-1.docker.io/v2/{}/manifests/latest", repo_name);
        
        let response = client
            .head(&url)
            .header("Accept", "application/vnd.docker.distribution.manifest.v2+json")
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }
}

/// GitHub Container Registry adapter
pub struct GhcrAdapter {
    organization: String,
}

impl GhcrAdapter {
    pub fn new() -> Self {
        Self {
            organization: "fastertools".to_string(),
        }
    }
}

impl RegistryAdapter for GhcrAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        format!("ghcr.io/{}/{}", self.organization, image_name)
    }
    
    async fn get_registry_components(&self, _client: &Client, image_name: &str) -> Result<RegistryComponents> {
        let (image_without_tag, tag) = parse_image_and_tag(image_name);
        
        // GHCR uses colon separator for Spin manifests: "org:repo" not "org/repo"
        let package_name = format!("{}:{}", self.organization, image_without_tag);
        
        let version = if tag == "latest" {
            anyhow::bail!("Invalid version tag '{}'. Spin manifests require explicit semantic versions (e.g., 1.0.0)", tag)
        } else {
            validate_and_normalize_semver(&tag)?
        };
        
        Ok(RegistryComponents {
            registry_domain: "ghcr.io".to_string(),
            package_name,
            version,
        })
    }
    
    fn name(&self) -> &'static str {
        "GitHub Container Registry (ghcr.io)"
    }
    
    async fn verify_image_exists(&self, client: &Client, image_name: &str) -> Result<bool> {
        // GHCR follows OCI Registry API v2
        let url = format!("https://ghcr.io/v2/{}/{}/manifests/latest", self.organization, image_name);
        
        let response = client
            .head(&url)
            .header("Accept", "application/vnd.oci.image.manifest.v1+json,application/vnd.docker.distribution.manifest.v2+json")
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }
}

/// AWS Elastic Container Registry adapter
pub struct EcrAdapter {
    account_id: String,
    region: String,
}

/// Custom registry adapter
pub struct CustomAdapter {
    url_pattern: String,
}

impl EcrAdapter {
    pub fn from_env() -> Result<Self> {
        let account_id = env::var("AWS_ACCOUNT_ID")
            .context("AWS_ACCOUNT_ID environment variable required for ECR")?;
        let region = env::var("AWS_REGION")
            .or_else(|_| env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| "us-east-1".to_string());
        
        Ok(Self {
            account_id,
            region,
        })
    }
    
    pub fn new(account_id: String, region: String) -> Self {
        Self {
            account_id,
            region,
        }
    }
}

impl RegistryAdapter for EcrAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        format!("{}.dkr.ecr.{}.amazonaws.com/{}", 
            self.account_id, 
            self.region, 
            image_name
        )
    }
    
    async fn get_registry_components(&self, _client: &Client, image_name: &str) -> Result<RegistryComponents> {
        let (image_without_tag, tag) = parse_image_and_tag(image_name);
        
        let registry_domain = format!("{}.dkr.ecr.{}.amazonaws.com", self.account_id, self.region);
        let package_name = image_without_tag;
        
        let version = if tag == "latest" {
            anyhow::bail!("Invalid version tag '{}'. Spin manifests require explicit semantic versions (e.g., 1.0.0)", tag)
        } else {
            validate_and_normalize_semver(&tag)?
        };
        
        Ok(RegistryComponents {
            registry_domain,
            package_name,
            version,
        })
    }
    
    fn name(&self) -> &'static str {
        "AWS Elastic Container Registry (ECR)"
    }
    
    async fn verify_image_exists(&self, _client: &Client, _image_name: &str) -> Result<bool> {
        // ECR uses AWS authentication - for now, return true (assume exists)
        // TODO: Implement proper ECR API authentication and verification
        Ok(true)
    }
}

impl RegistryAdapter for CustomAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        self.url_pattern.replace("{image_name}", image_name)
    }
    
    async fn get_registry_components(&self, _client: &Client, image_name: &str) -> Result<RegistryComponents> {
        let (image_without_tag, tag) = parse_image_and_tag(image_name);
        
        // For custom registries, try to extract registry domain from pattern
        let registry_domain = if let Some(start) = self.url_pattern.find("://") {
            let after_scheme = &self.url_pattern[start + 3..];
            if let Some(end) = after_scheme.find('/') {
                after_scheme[..end].to_string()
            } else {
                after_scheme.to_string()
            }
        } else {
            // Fallback: assume pattern starts with domain
            if let Some(end) = self.url_pattern.find('/') {
                self.url_pattern[..end].to_string()
            } else {
                "github.com/fastertools".to_string()
            }
        };
        
        let package_name = image_without_tag;
        
        let version = if tag == "latest" {
            anyhow::bail!("Invalid version tag '{}'. Spin manifests require explicit semantic versions (e.g., 1.0.0)", tag)
        } else {
            validate_and_normalize_semver(&tag)?
        };
        
        Ok(RegistryComponents {
            registry_domain,
            package_name,
            version,
        })
    }
    
    fn name(&self) -> &'static str {
        "Custom Registry"
    }
    
    async fn verify_image_exists(&self, client: &Client, image_name: &str) -> Result<bool> {
        // For custom registries, try a HEAD request to the manifest
        let url = self.get_registry_url(image_name);
        
        // Try to construct a manifest URL
        let manifest_url = if url.contains("/manifests/") {
            url
        } else {
            // Parse image and tag properly
            let (_image_without_tag, tag) = parse_image_and_tag(image_name);
            
            // Convert from image URL to manifest URL
            // Replace the tag portion with /manifests/tag
            if let Some(tag_pos) = url.rfind(&format!(":{}", tag)) {
                format!("{}/manifests/{}", &url[..tag_pos], tag)
            } else {
                // Fallback: append /manifests/tag to the URL
                format!("{}/manifests/{}", url, tag)
            }
        };
        
        let response = client
            .head(&manifest_url)
            .header("Accept", "application/vnd.oci.image.manifest.v1+json,application/vnd.docker.distribution.manifest.v2+json")
            .send()
            .await?;
            
        Ok(response.status().is_success())
    }
}

/// Enum holding different registry adapter types
pub enum ConcreteRegistryAdapter {
    DockerHub(DockerHubAdapter),
    Ghcr(GhcrAdapter),
    Ecr(EcrAdapter),
    Custom(CustomAdapter),
}

impl RegistryAdapter for ConcreteRegistryAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        match self {
            ConcreteRegistryAdapter::DockerHub(adapter) => adapter.get_registry_url(image_name),
            ConcreteRegistryAdapter::Ghcr(adapter) => adapter.get_registry_url(image_name),
            ConcreteRegistryAdapter::Ecr(adapter) => adapter.get_registry_url(image_name),
            ConcreteRegistryAdapter::Custom(adapter) => adapter.get_registry_url(image_name),
        }
    }
    
    async fn get_registry_components(&self, client: &Client, image_name: &str) -> Result<RegistryComponents> {
        match self {
            ConcreteRegistryAdapter::DockerHub(adapter) => adapter.get_registry_components(client, image_name).await,
            ConcreteRegistryAdapter::Ghcr(adapter) => adapter.get_registry_components(client, image_name).await,
            ConcreteRegistryAdapter::Ecr(adapter) => adapter.get_registry_components(client, image_name).await,
            ConcreteRegistryAdapter::Custom(adapter) => adapter.get_registry_components(client, image_name).await,
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            ConcreteRegistryAdapter::DockerHub(adapter) => adapter.name(),
            ConcreteRegistryAdapter::Ghcr(adapter) => adapter.name(),
            ConcreteRegistryAdapter::Ecr(adapter) => adapter.name(),
            ConcreteRegistryAdapter::Custom(adapter) => adapter.name(),
        }
    }
    
    async fn verify_image_exists(&self, client: &Client, image_name: &str) -> Result<bool> {
        match self {
            ConcreteRegistryAdapter::DockerHub(adapter) => adapter.verify_image_exists(client, image_name).await,
            ConcreteRegistryAdapter::Ghcr(adapter) => adapter.verify_image_exists(client, image_name).await,
            ConcreteRegistryAdapter::Ecr(adapter) => adapter.verify_image_exists(client, image_name).await,
            ConcreteRegistryAdapter::Custom(adapter) => adapter.verify_image_exists(client, image_name).await,
        }
    }
}

/// Get registry adapter based on registry name
pub fn get_registry_adapter(registry: Option<&str>) -> Result<ConcreteRegistryAdapter> {
    use crate::config::FtlConfig;
    
    // Try to load config for custom registries
    if let Some(reg_name) = registry {
        if let Ok(config) = FtlConfig::load() {
            if let Some(reg_config) = config.get_registry(reg_name) {
                use crate::config::registry::RegistryType;
                
                match reg_config.registry_type {
                    RegistryType::Ghcr => {
                        let org = reg_config.get_config_str("organization")
                            .unwrap_or_else(|| "fastertools".to_string());
                        return Ok(ConcreteRegistryAdapter::Ghcr(GhcrAdapter { organization: org }));
                    }
                    RegistryType::Docker => {
                        return Ok(ConcreteRegistryAdapter::DockerHub(DockerHubAdapter));
                    }
                    RegistryType::Ecr => {
                        if let (Some(account), Some(region)) = 
                            (reg_config.get_config_str("account_id"), 
                             reg_config.get_config_str("region")) {
                            return Ok(ConcreteRegistryAdapter::Ecr(EcrAdapter::new(account, region)));
                        } else {
                            return Ok(ConcreteRegistryAdapter::Ecr(EcrAdapter::from_env()?));
                        }
                    }
                    RegistryType::Custom => {
                        if let Some(pattern) = reg_config.get_config_str("url_pattern") {
                            return Ok(ConcreteRegistryAdapter::Custom(CustomAdapter { url_pattern: pattern }));
                        }
                    }
                }
            }
        }
    }
    
    // Fallback to original behavior for backward compatibility
    match registry {
        None | Some("ghcr") => Ok(ConcreteRegistryAdapter::Ghcr(GhcrAdapter::new())),
        Some("docker") => Ok(ConcreteRegistryAdapter::DockerHub(DockerHubAdapter)),
        Some("ecr") => Ok(ConcreteRegistryAdapter::Ecr(EcrAdapter::from_env()?)),
        Some(other) => anyhow::bail!("Unsupported registry: {}. Supported: ghcr, docker, ecr, or configure custom registry with 'ftl registries add'", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_hub_official_images() {
        let adapter = DockerHubAdapter;
        assert_eq!(adapter.get_registry_url("nginx"), "docker.io/library/nginx");
        assert_eq!(adapter.get_registry_url("ubuntu"), "docker.io/library/ubuntu");
        assert_eq!(adapter.get_registry_url("redis"), "docker.io/library/redis");
    }

    #[test]
    fn test_docker_hub_user_images() {
        let adapter = DockerHubAdapter;
        assert_eq!(adapter.get_registry_url("username/myapp"), "docker.io/username/myapp");
        assert_eq!(adapter.get_registry_url("org/tool"), "docker.io/org/tool");
    }

    #[test]
    fn test_ghcr_adapter() {
        let adapter = GhcrAdapter::new();
        assert_eq!(adapter.get_registry_url("ftl-tool-add"), "ghcr.io/fastertools/ftl-tool-add");
    }

    #[test]
    fn test_registry_factory() {
        let docker_adapter = get_registry_adapter(Some("docker")).unwrap();
        assert_eq!(docker_adapter.name(), "Docker Hub");
        
        let ghcr_adapter = get_registry_adapter(Some("ghcr")).unwrap();
        assert_eq!(ghcr_adapter.name(), "GitHub Container Registry (ghcr.io)");
        
        let default_adapter = get_registry_adapter(None).unwrap();
        assert_eq!(default_adapter.name(), "GitHub Container Registry (ghcr.io)");
    }

    #[test]
    fn test_parse_image_and_tag() {
        assert_eq!(parse_image_and_tag("nginx"), ("nginx".to_string(), "latest".to_string()));
        assert_eq!(parse_image_and_tag("nginx:1.21"), ("nginx".to_string(), "1.21".to_string()));
        assert_eq!(parse_image_and_tag("user/app:v1.0"), ("user/app".to_string(), "v1.0".to_string()));
        assert_eq!(parse_image_and_tag("registry.io/org/app:1.0.0"), ("registry.io/org/app".to_string(), "1.0.0".to_string()));
    }

    #[test]
    fn test_validate_and_normalize_semver() {
        // Valid semantic versions
        assert_eq!(validate_and_normalize_semver("1.0.0").unwrap(), "1.0.0");
        assert_eq!(validate_and_normalize_semver("v1.0.0").unwrap(), "1.0.0");
        assert_eq!(validate_and_normalize_semver("2.1.3-alpha").unwrap(), "2.1.3-alpha");
        assert_eq!(validate_and_normalize_semver("0.0.6").unwrap(), "0.0.6");
        
        // Auto-completion of versions
        assert_eq!(validate_and_normalize_semver("1").unwrap(), "1.0.0");
        assert_eq!(validate_and_normalize_semver("1.2").unwrap(), "1.2.0");
        assert_eq!(validate_and_normalize_semver("v1.2").unwrap(), "1.2.0");
        
        // Invalid versions should return error
        assert!(validate_and_normalize_semver("latest").is_err());
        assert!(validate_and_normalize_semver("main").is_err());
        assert!(validate_and_normalize_semver("invalid.version.format.too.many").is_err());
    }

    #[tokio::test]
    async fn test_docker_hub_registry_components() {
        let adapter = DockerHubAdapter;
        let client = reqwest::Client::new();
        
        // Test official image without tag - should error for "latest"
        let result = adapter.get_registry_components(&client, "nginx").await;
        assert!(result.is_err()); // "latest" should be rejected
        
        // Test user image with tag
        let components = adapter.get_registry_components(&client, "user/app:1.2.0").await.unwrap();
        assert_eq!(components.registry_domain, "docker.io");
        assert_eq!(components.package_name, "user/app");
        assert_eq!(components.version, "1.2.0");
        
        // Test version normalization
        let components = adapter.get_registry_components(&client, "nginx:v1.21").await.unwrap();
        assert_eq!(components.version, "1.21.0");
    }

    #[tokio::test]
    async fn test_ghcr_registry_components() {
        let adapter = GhcrAdapter::new();
        let client = reqwest::Client::new();
        
        // Test GHCR with colon separator
        let components = adapter.get_registry_components(&client, "ftl-auth-gateway:0.0.6").await.unwrap();
        assert_eq!(components.registry_domain, "ghcr.io");
        assert_eq!(components.package_name, "fastertools:ftl-auth-gateway");
        assert_eq!(components.version, "0.0.6");
        
        // Test without tag (should error for "latest")
        let result = adapter.get_registry_components(&client, "my-tool").await;
        assert!(result.is_err()); // "latest" should be rejected
    }

    #[tokio::test]
    async fn test_ecr_registry_components() {
        let adapter = EcrAdapter::new("123456".to_string(), "us-east-1".to_string());
        let client = reqwest::Client::new();
        
        let components = adapter.get_registry_components(&client, "my-app:1.0.0").await.unwrap();
        assert_eq!(components.registry_domain, "123456.dkr.ecr.us-east-1.amazonaws.com");
        assert_eq!(components.package_name, "my-app");
        assert_eq!(components.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_custom_registry_components() {
        let adapter = CustomAdapter {
            url_pattern: "registry.example.com/{image_name}".to_string(),
        };
        let client = reqwest::Client::new();
        
        let components = adapter.get_registry_components(&client, "tool:v2.0.0").await.unwrap();
        assert_eq!(components.registry_domain, "registry.example.com");
        assert_eq!(components.package_name, "tool");
        assert_eq!(components.version, "2.0.0");
    }

    #[tokio::test]
    async fn test_custom_registry_with_port() {
        let adapter = CustomAdapter {
            url_pattern: "https://registry.company.com:5000/v2/{image_name}".to_string(),
        };
        let client = reqwest::Client::new();
        
        let components = adapter.get_registry_components(&client, "myapp:1.2.3").await.unwrap();
        assert_eq!(components.registry_domain, "registry.company.com:5000");
        assert_eq!(components.package_name, "myapp");
        assert_eq!(components.version, "1.2.3");
    }
}