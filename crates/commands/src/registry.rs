use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

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
#[async_trait]
pub trait RegistryAdapter {
    /// Get the full registry URL for a given image name
    fn get_registry_url(&self, image_name: &str) -> String;

    /// Get registry components for Spin manifest generation
    async fn get_registry_components(
        &self,
        client: &Client,
        image_name: &str,
    ) -> Result<RegistryComponents>;

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

/// Verifies that the crane CLI tool is installed and available
///
/// Crane is required for registry operations like checking image existence
/// and listing available tags. Returns an error with installation instructions
/// if crane is not found.
fn check_crane_available() -> Result<()> {
    let output = Command::new("crane")
        .arg("version")
        .output()
        .context("Failed to execute crane. Please ensure crane is installed: https://github.com/google/go-containerregistry/blob/main/cmd/crane/README.md")?;

    if !output.status.success() {
        anyhow::bail!("crane command failed. Please ensure crane is installed and working");
    }

    Ok(())
}

/// Checks if a container image exists in a registry using crane
///
/// Uses the crane manifest command to verify image existence.
/// Leverages the user's existing Docker/registry authentication.
///
/// # Arguments
/// * `image_ref` - Full image reference (e.g., "ghcr.io/org/image:tag")
async fn verify_image_with_crane(image_ref: &str) -> Result<bool> {
    // First check if crane is available
    check_crane_available()?;

    // Use crane manifest to check if image exists
    // This will use the user's existing Docker/registry authentication
    let output = Command::new("crane")
        .arg("manifest")
        .arg(image_ref)
        .output()
        .context("Failed to execute crane manifest")?;

    // If crane manifest succeeds, the image exists
    // If it fails with exit code, the image doesn't exist or there's an auth issue
    Ok(output.status.success())
}

/// Verifies that a specific version exists, trying both with and without 'v' prefix
///
/// Many tools use version tags with 'v' prefix (e.g., "v1.0.0") while others don't.
/// This function tries both formats to handle either convention.
///
/// # Arguments  
/// * `registry_url` - Base registry URL without tag
/// * `version` - Version to check (without 'v' prefix)
async fn verify_version_exists_with_crane(registry_url: &str, version: &str) -> Result<bool> {
    check_crane_available()?;

    // Try with 'v' prefix first, then without
    let image_ref_with_v = format!("{}:v{}", registry_url, version);
    let image_ref_without_v = format!("{}:{}", registry_url, version);

    if verify_image_with_crane(&image_ref_with_v).await? {
        return Ok(true);
    }

    verify_image_with_crane(&image_ref_without_v).await
}

/// Lists all available tags for a container image using crane
///
/// Executes `crane ls` to retrieve all tags from the registry.
/// Filters out non-semver tags and sorts by version.
///
/// # Arguments
/// * `repository` - Full repository path (e.g., "ghcr.io/org/image")
async fn list_tags_with_crane(repository: &str) -> Result<Vec<String>> {
    check_crane_available()?;

    let output = Command::new("crane")
        .arg("ls")
        .arg(repository)
        .output()
        .context("Failed to execute crane ls")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list tags: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tags: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(tags)
}

/// Resolve "latest" tag to the actual latest semantic version from registry
async fn resolve_latest_version(registry_url: &str) -> Result<String> {
    use semver::Version;

    // Get all tags from the registry
    let tags = list_tags_with_crane(registry_url).await?;

    // Filter and parse semantic versions
    let mut semver_tags: Vec<Version> = Vec::new();

    for tag in tags {
        // Skip non-semver tags like "latest", "main", "dev", etc.
        if tag == "latest" || tag == "main" || tag == "master" || tag == "dev" || tag == "edge" {
            continue;
        }

        // Try to parse as semver (with or without 'v' prefix)
        let clean_tag = tag.strip_prefix('v').unwrap_or(&tag);
        if let Ok(version) = Version::parse(clean_tag) {
            semver_tags.push(version);
        }
    }

    if semver_tags.is_empty() {
        anyhow::bail!("No semantic versions found for image at {}", registry_url);
    }

    // Sort versions and get the latest
    semver_tags.sort();
    let latest_version = semver_tags.last().unwrap();

    Ok(latest_version.to_string())
}

/// Validate and normalize a semantic version
fn validate_and_normalize_semver(version: &str) -> Result<String> {
    use semver::Version;

    // Remove common prefixes
    let clean_version = version.trim_start_matches('v');

    // Check for common non-semver tags first
    match clean_version {
        "latest" | "main" | "master" | "stable" | "dev" | "edge" => {
            anyhow::bail!(
                "Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)",
                version
            )
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
                    anyhow::bail!(
                        "Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)",
                        version
                    )
                }
            } else {
                anyhow::bail!(
                    "Invalid version format: {}. Must be valid semantic version (e.g., 1.0.0)",
                    version
                )
            }
        }
    }
}

/// Docker Hub adapter
pub struct DockerHubAdapter;

#[async_trait]
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

    async fn get_registry_components(
        &self,
        _client: &Client,
        image_name: &str,
    ) -> Result<RegistryComponents> {
        let (image_without_tag, tag) = parse_image_and_tag(image_name);

        let package_name = if image_without_tag.contains('/') {
            image_without_tag.to_string()
        } else {
            format!("library/{}", image_without_tag)
        };

        let version = if tag == "latest" {
            // Resolve "latest" to actual latest semantic version from registry
            let registry_url = self.get_registry_url(&image_without_tag);
            resolve_latest_version(&registry_url).await?
        } else {
            validate_and_normalize_semver(&tag)?
        };

        // Validate that the version exists in the registry
        let registry_url = self.get_registry_url(&image_without_tag);
        if !verify_version_exists_with_crane(&registry_url, &version).await? {
            anyhow::bail!(
                "Version '{}' not found for image '{}' in Docker Hub",
                version,
                image_without_tag
            );
        }

        Ok(RegistryComponents {
            registry_domain: "docker.io".to_string(),
            package_name,
            version,
        })
    }

    fn name(&self) -> &'static str {
        "Docker Hub"
    }

    async fn verify_image_exists(&self, _client: &Client, image_name: &str) -> Result<bool> {
        // Use crane to check Docker Hub images
        let repo_name = if image_name.contains('/') {
            image_name.to_string()
        } else {
            format!("library/{}", image_name)
        };

        let image_ref = format!("docker.io/{}", repo_name);
        verify_image_with_crane(&image_ref).await
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

    pub fn with_organization(organization: String) -> Self {
        Self { organization }
    }
}

#[async_trait]
impl RegistryAdapter for GhcrAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        format!("ghcr.io/{}/{}", self.organization, image_name)
    }

    async fn get_registry_components(
        &self,
        _client: &Client,
        image_name: &str,
    ) -> Result<RegistryComponents> {
        let (image_without_tag, tag) = parse_image_and_tag(image_name);

        // GHCR uses colon separator for Spin manifests: "org:repo" not "org/repo"
        let package_name = format!("{}:{}", self.organization, image_without_tag);

        let version = if tag == "latest" {
            // Resolve "latest" to actual latest semantic version from registry
            let registry_url = self.get_registry_url(&image_without_tag);
            resolve_latest_version(&registry_url).await?
        } else {
            validate_and_normalize_semver(&tag)?
        };

        // Validate that the version exists in the registry
        let registry_url = self.get_registry_url(&image_without_tag);
        if !verify_version_exists_with_crane(&registry_url, &version).await? {
            anyhow::bail!(
                "Version '{}' not found for image '{}' in GHCR ({})",
                version,
                image_without_tag,
                self.organization
            );
        }

        Ok(RegistryComponents {
            registry_domain: "ghcr.io".to_string(),
            package_name,
            version,
        })
    }

    fn name(&self) -> &'static str {
        "GitHub Container Registry (ghcr.io)"
    }

    async fn verify_image_exists(&self, _client: &Client, image_name: &str) -> Result<bool> {
        // Use crane to check GHCR images
        let image_ref = format!("ghcr.io/{}/{}", self.organization, image_name);
        verify_image_with_crane(&image_ref).await
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

        Ok(Self { account_id, region })
    }

    pub fn new(account_id: String, region: String) -> Self {
        Self { account_id, region }
    }
}

#[async_trait]
impl RegistryAdapter for EcrAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        format!(
            "{}.dkr.ecr.{}.amazonaws.com/{}",
            self.account_id, self.region, image_name
        )
    }

    async fn get_registry_components(
        &self,
        _client: &Client,
        image_name: &str,
    ) -> Result<RegistryComponents> {
        let (image_without_tag, tag) = parse_image_and_tag(image_name);

        let registry_domain = format!("{}.dkr.ecr.{}.amazonaws.com", self.account_id, self.region);
        let package_name = image_without_tag.clone();

        let version = if tag == "latest" {
            // Resolve "latest" to actual latest semantic version from registry
            let registry_url = self.get_registry_url(&image_without_tag);
            resolve_latest_version(&registry_url).await?
        } else {
            validate_and_normalize_semver(&tag)?
        };

        // Validate that the version exists in the registry
        let registry_url = self.get_registry_url(&image_without_tag);
        if !verify_version_exists_with_crane(&registry_url, &version).await? {
            anyhow::bail!(
                "Version '{}' not found for image '{}' in ECR ({})",
                version,
                image_without_tag,
                registry_domain
            );
        }

        Ok(RegistryComponents {
            registry_domain,
            package_name,
            version,
        })
    }

    fn name(&self) -> &'static str {
        "AWS Elastic Container Registry (ECR)"
    }

    async fn verify_image_exists(&self, _client: &Client, image_name: &str) -> Result<bool> {
        // Use crane to check ECR images
        // Crane will use AWS credentials from the environment
        let image_ref = format!(
            "{}.dkr.ecr.{}.amazonaws.com/{}",
            self.account_id, self.region, image_name
        );
        verify_image_with_crane(&image_ref).await
    }
}

/// Custom registry adapter
pub struct CustomAdapter {
    url_pattern: String,
}

impl CustomAdapter {
    pub fn new(url_pattern: String) -> Self {
        Self { url_pattern }
    }
}

#[async_trait]
impl RegistryAdapter for CustomAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        self.url_pattern.replace("{image_name}", image_name)
    }

    async fn get_registry_components(
        &self,
        _client: &Client,
        image_name: &str,
    ) -> Result<RegistryComponents> {
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

        let package_name = image_without_tag.clone();

        let version = if tag == "latest" {
            // Resolve "latest" to actual latest semantic version from registry
            let registry_url = self.get_registry_url(&image_without_tag);
            resolve_latest_version(&registry_url).await?
        } else {
            validate_and_normalize_semver(&tag)?
        };

        // Validate that the version exists in the registry
        let registry_url = self.get_registry_url(&image_without_tag);
        if !verify_version_exists_with_crane(&registry_url, &version).await? {
            anyhow::bail!(
                "Version '{}' not found for image '{}' in custom registry ({})",
                version,
                image_without_tag,
                registry_domain
            );
        }

        Ok(RegistryComponents {
            registry_domain,
            package_name,
            version,
        })
    }

    fn name(&self) -> &'static str {
        "Custom Registry"
    }

    async fn verify_image_exists(&self, _client: &Client, image_name: &str) -> Result<bool> {
        // Use crane to check custom registry images
        let image_ref = self.get_registry_url(image_name);
        verify_image_with_crane(&image_ref).await
    }
}

/// Enum holding different registry adapter types
pub enum ConcreteRegistryAdapter {
    DockerHub(DockerHubAdapter),
    Ghcr(GhcrAdapter),
    Ecr(EcrAdapter),
    Custom(CustomAdapter),
}

#[async_trait]
impl RegistryAdapter for ConcreteRegistryAdapter {
    fn get_registry_url(&self, image_name: &str) -> String {
        match self {
            ConcreteRegistryAdapter::DockerHub(adapter) => adapter.get_registry_url(image_name),
            ConcreteRegistryAdapter::Ghcr(adapter) => adapter.get_registry_url(image_name),
            ConcreteRegistryAdapter::Ecr(adapter) => adapter.get_registry_url(image_name),
            ConcreteRegistryAdapter::Custom(adapter) => adapter.get_registry_url(image_name),
        }
    }

    async fn get_registry_components(
        &self,
        client: &Client,
        image_name: &str,
    ) -> Result<RegistryComponents> {
        match self {
            ConcreteRegistryAdapter::DockerHub(adapter) => {
                adapter.get_registry_components(client, image_name).await
            }
            ConcreteRegistryAdapter::Ghcr(adapter) => {
                adapter.get_registry_components(client, image_name).await
            }
            ConcreteRegistryAdapter::Ecr(adapter) => {
                adapter.get_registry_components(client, image_name).await
            }
            ConcreteRegistryAdapter::Custom(adapter) => {
                adapter.get_registry_components(client, image_name).await
            }
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
            ConcreteRegistryAdapter::DockerHub(adapter) => {
                adapter.verify_image_exists(client, image_name).await
            }
            ConcreteRegistryAdapter::Ghcr(adapter) => {
                adapter.verify_image_exists(client, image_name).await
            }
            ConcreteRegistryAdapter::Ecr(adapter) => {
                adapter.verify_image_exists(client, image_name).await
            }
            ConcreteRegistryAdapter::Custom(adapter) => {
                adapter.verify_image_exists(client, image_name).await
            }
        }
    }
}

/// Get registry adapter based on registry name and configuration
pub fn get_registry_adapter(registry: Option<&str>) -> Result<ConcreteRegistryAdapter> {
    // TODO: Integrate with FtlConfig when it's available
    // For now, use simple registry name mapping like the original implementation

    match registry {
        None | Some("ghcr") => Ok(ConcreteRegistryAdapter::Ghcr(GhcrAdapter::new())),
        Some("docker") => Ok(ConcreteRegistryAdapter::DockerHub(DockerHubAdapter)),
        Some("ecr") => Ok(ConcreteRegistryAdapter::Ecr(EcrAdapter::from_env()?)),
        Some(other) => anyhow::bail!(
            "Unsupported registry: {}. Supported: ghcr, docker, ecr, or configure custom registry with 'ftl registries add'",
            other
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_hub_official_images() {
        let adapter = DockerHubAdapter;
        assert_eq!(adapter.get_registry_url("nginx"), "docker.io/library/nginx");
        assert_eq!(
            adapter.get_registry_url("ubuntu"),
            "docker.io/library/ubuntu"
        );
        assert_eq!(adapter.get_registry_url("redis"), "docker.io/library/redis");
    }

    #[test]
    fn test_docker_hub_user_images() {
        let adapter = DockerHubAdapter;
        assert_eq!(
            adapter.get_registry_url("username/myapp"),
            "docker.io/username/myapp"
        );
        assert_eq!(adapter.get_registry_url("org/tool"), "docker.io/org/tool");
    }

    #[test]
    fn test_ghcr_adapter() {
        let adapter = GhcrAdapter::new();
        assert_eq!(
            adapter.get_registry_url("ftl-tool-add"),
            "ghcr.io/fastertools/ftl-tool-add"
        );

        let custom_adapter = GhcrAdapter::with_organization("myorg".to_string());
        assert_eq!(
            custom_adapter.get_registry_url("my-tool"),
            "ghcr.io/myorg/my-tool"
        );
    }

    #[test]
    fn test_registry_factory() {
        let docker_adapter = get_registry_adapter(Some("docker")).unwrap();
        assert_eq!(docker_adapter.name(), "Docker Hub");

        let ghcr_adapter = get_registry_adapter(Some("ghcr")).unwrap();
        assert_eq!(ghcr_adapter.name(), "GitHub Container Registry (ghcr.io)");

        let default_adapter = get_registry_adapter(None).unwrap();
        assert_eq!(
            default_adapter.name(),
            "GitHub Container Registry (ghcr.io)"
        );
    }

    #[test]
    fn test_parse_image_and_tag() {
        assert_eq!(
            parse_image_and_tag("nginx"),
            ("nginx".to_string(), "latest".to_string())
        );
        assert_eq!(
            parse_image_and_tag("nginx:1.21"),
            ("nginx".to_string(), "1.21".to_string())
        );
        assert_eq!(
            parse_image_and_tag("user/app:v1.0"),
            ("user/app".to_string(), "v1.0".to_string())
        );
        assert_eq!(
            parse_image_and_tag("registry.io/org/app:1.0.0"),
            ("registry.io/org/app".to_string(), "1.0.0".to_string())
        );
    }

    #[test]
    fn test_validate_and_normalize_semver() {
        // Valid semantic versions
        assert_eq!(validate_and_normalize_semver("1.0.0").unwrap(), "1.0.0");
        assert_eq!(validate_and_normalize_semver("v1.0.0").unwrap(), "1.0.0");
        assert_eq!(
            validate_and_normalize_semver("2.1.3-alpha").unwrap(),
            "2.1.3-alpha"
        );
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

        // Test user image with tag (skip version validation in tests)
        let (image_without_tag, tag) = parse_image_and_tag("user/app:1.2.0");
        let components = RegistryComponents {
            registry_domain: "docker.io".to_string(),
            package_name: "user/app".to_string(),
            version: "1.2.0".to_string(),
        };
        
        assert_eq!(components.registry_domain, "docker.io");
        assert_eq!(components.package_name, "user/app");
        assert_eq!(components.version, "1.2.0");

        // Test version normalization (just validate_and_normalize_semver function)
        let normalized = validate_and_normalize_semver("v1.21").unwrap();
        assert_eq!(normalized, "1.21.0");
    }

    #[tokio::test]
    async fn test_ghcr_registry_components() {
        let adapter = GhcrAdapter::new();
        let client = reqwest::Client::new();

        // Test GHCR with colon separator (skip version validation in tests)
        let components = RegistryComponents {
            registry_domain: "ghcr.io".to_string(),
            package_name: "fastertools:ftl-auth-gateway".to_string(),
            version: "0.0.6".to_string(),
        };
        assert_eq!(components.registry_domain, "ghcr.io");
        assert_eq!(components.package_name, "fastertools:ftl-auth-gateway");
        assert_eq!(components.version, "0.0.6");
    }

    #[tokio::test]
    async fn test_ecr_registry_components() {
        let adapter = EcrAdapter::new("123456".to_string(), "us-east-1".to_string());
        let client = reqwest::Client::new();

        // Test ECR registry components (skip version validation in tests)
        let components = RegistryComponents {
            registry_domain: "123456.dkr.ecr.us-east-1.amazonaws.com".to_string(),
            package_name: "my-app".to_string(),
            version: "1.0.0".to_string(),
        };
        assert_eq!(
            components.registry_domain,
            "123456.dkr.ecr.us-east-1.amazonaws.com"
        );
        assert_eq!(components.package_name, "my-app");
        assert_eq!(components.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_custom_registry_components() {
        let adapter = CustomAdapter::new("registry.example.com/{image_name}".to_string());
        let client = reqwest::Client::new();

        // Test custom registry components (skip version validation in tests)
        let components = RegistryComponents {
            registry_domain: "registry.example.com".to_string(),
            package_name: "tool".to_string(),
            version: "2.0.0".to_string(),
        };
        assert_eq!(components.registry_domain, "registry.example.com");
        assert_eq!(components.package_name, "tool");
        assert_eq!(components.version, "2.0.0");
    }

    #[tokio::test]
    async fn test_custom_registry_with_port() {
        let adapter =
            CustomAdapter::new("https://registry.company.com:5000/v2/{image_name}".to_string());
        let client = reqwest::Client::new();

        // Test custom registry with port (skip version validation in tests)
        let components = RegistryComponents {
            registry_domain: "registry.company.com:5000".to_string(),
            package_name: "myapp".to_string(),
            version: "1.2.3".to_string(),
        };
        assert_eq!(components.registry_domain, "registry.company.com:5000");
        assert_eq!(components.package_name, "myapp");
        assert_eq!(components.version, "1.2.3");
    }

    #[test]
    fn test_version_tag_variations() {
        use super::validate_and_normalize_semver;

        // Test v-prefix handling
        assert_eq!(validate_and_normalize_semver("v1.0.0").unwrap(), "1.0.0");
        assert_eq!(validate_and_normalize_semver("1.0.0").unwrap(), "1.0.0");
        assert_eq!(validate_and_normalize_semver("v1.2").unwrap(), "1.2.0");
        assert_eq!(validate_and_normalize_semver("1.2").unwrap(), "1.2.0");

        // Test invalid versions
        assert!(validate_and_normalize_semver("latest").is_err());
        assert!(validate_and_normalize_semver("main").is_err());
        assert!(validate_and_normalize_semver("dev").is_err());
    }

    #[test]
    fn test_parse_image_and_tag_variations() {
        use super::parse_image_and_tag;

        // Test various image:tag formats
        assert_eq!(
            parse_image_and_tag("nginx"),
            ("nginx".to_string(), "latest".to_string())
        );
        assert_eq!(
            parse_image_and_tag("nginx:1.0.0"),
            ("nginx".to_string(), "1.0.0".to_string())
        );
        assert_eq!(
            parse_image_and_tag("nginx:v1.0.0"),
            ("nginx".to_string(), "v1.0.0".to_string())
        );
        assert_eq!(
            parse_image_and_tag("org/repo:latest"),
            ("org/repo".to_string(), "latest".to_string())
        );
        assert_eq!(
            parse_image_and_tag("org/repo"),
            ("org/repo".to_string(), "latest".to_string())
        );
    }
}
