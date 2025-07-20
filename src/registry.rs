use anyhow::{Context, Result};
use std::env;

/// Trait for adapting different registry formats
pub trait RegistryAdapter {
    /// Get the full registry URL for a given image name
    fn get_registry_url(&self, image_name: &str) -> String;
    
    /// Get a human-readable name for this registry
    fn name(&self) -> &'static str;
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
        format!("ghcr.io/{}/{}:latest", self.organization, image_name)
    }
    
    fn name(&self) -> &'static str {
        "GitHub Container Registry (ghcr.io)"
    }
}

/// AWS Elastic Container Registry adapter
pub struct EcrAdapter {
    account_id: String,
    region: String,
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
        format!("{}.dkr.ecr.{}.amazonaws.com/{}:latest", 
            self.account_id, 
            self.region, 
            image_name
        )
    }
    
    fn name(&self) -> &'static str {
        "AWS Elastic Container Registry (ECR)"
    }
}

/// Get registry adapter based on registry name
pub fn get_registry_adapter(registry: Option<&str>) -> Result<Box<dyn RegistryAdapter>> {
    match registry {
        None | Some("ghcr") => Ok(Box::new(GhcrAdapter::new())),
        Some("ecr") => Ok(Box::new(EcrAdapter::from_env()?)),
        Some(other) => anyhow::bail!("Unsupported registry: {}. Supported: ghcr, ecr", other),
    }
}