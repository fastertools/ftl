use anyhow::{Context, Result};
use std::env;
use reqwest::Client;

/// Trait for adapting different registry formats
pub trait RegistryAdapter {
    /// Get the full registry URL for a given image name
    fn get_registry_url(&self, image_name: &str) -> String;
    
    /// Get a human-readable name for this registry
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    
    /// Verify if an image exists in this registry
    async fn verify_image_exists(&self, client: &Client, image_name: &str) -> Result<bool>;
}

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
    
    fn name(&self) -> &'static str {
        "Custom Registry"
    }
    
    async fn verify_image_exists(&self, client: &Client, image_name: &str) -> Result<bool> {
        // For custom registries, try a HEAD request to the manifest
        let url = self.get_registry_url(image_name);
        
        // Try to construct a manifest URL (this is a best guess)
        let manifest_url = if url.contains("/manifests/") {
            url
        } else {
            // Convert from image URL to manifest URL (best effort)
            url.replace(":latest", "/manifests/latest")
               .replace(format!("{}:latest", image_name).as_str(), format!("{}/manifests/latest", image_name).as_str())
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
}