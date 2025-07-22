use crate::formatters::spin::SpinFormatter;
use crate::registry::{get_registry_adapter, RegistryAdapter};
use anyhow::Result;
use reqwest::Client;

/// Integration tests for registry → spin.toml workflow
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_hub_to_spin_manifest() -> Result<()> {
        let client = Client::new();
        let adapter = get_registry_adapter(Some("docker"))?;
        
        // Test official image
        let components = adapter.get_registry_components(&client, "nginx:1.21.0").await?;
        assert_eq!(components.registry_domain, "docker.io");
        assert_eq!(components.package_name, "library/nginx");
        assert_eq!(components.version, "1.21.0");
        
        // Generate spin.toml
        let manifest = SpinFormatter::generate_minimal_manifest(
            "test-app",
            "0.1.0",
            "web",
            &components,
            Some("/hello")
        )?;
        
        // Validate TOML structure
        assert!(manifest.contains("spin_manifest_version = 2"));
        assert!(manifest.contains(r#"name = "test-app""#));
        assert!(manifest.contains(r#"registry = "docker.io""#));
        assert!(manifest.contains(r#"package = "library/nginx""#));
        assert!(manifest.contains(r#"version = "1.21.0""#));
        assert!(manifest.contains(r#"route = "/hello""#));
        assert!(manifest.contains(r#"component = "web""#));
        
        // Test parsing with toml crate
        let parsed: toml::Value = toml::from_str(&manifest)?;
        assert_eq!(parsed["spin_manifest_version"].as_integer(), Some(2));
        assert_eq!(parsed["application"]["name"].as_str(), Some("test-app"));
        
        Ok(())
    }

    #[tokio::test] 
    async fn test_ghcr_to_spin_manifest() -> Result<()> {
        let client = Client::new();
        let adapter = get_registry_adapter(Some("ghcr"))?;
        
        // Test GHCR with colon separator
        let components = adapter.get_registry_components(&client, "ftl-auth-gateway:0.0.6").await?;
        assert_eq!(components.registry_domain, "ghcr.io");
        assert_eq!(components.package_name, "fastertools:ftl-auth-gateway");
        assert_eq!(components.version, "0.0.6");
        
        // Generate spin.toml
        let manifest = SpinFormatter::generate_minimal_manifest(
            "ftl-mcp-demo",
            "0.1.0",
            "mcp",
            &components,
            None // No HTTP route
        )?;
        
        // Validate TOML structure matches expected format
        assert!(manifest.contains(r#"registry = "ghcr.io""#));
        assert!(manifest.contains(r#"package = "fastertools:ftl-auth-gateway""#));
        assert!(manifest.contains(r#"version = "0.0.6""#));
        
        // Ensure colon separator is preserved
        assert!(!manifest.contains("fastertools/ftl-auth-gateway"));
        assert!(manifest.contains("fastertools:ftl-auth-gateway"));
        
        // Test parsing
        let parsed: toml::Value = toml::from_str(&manifest)?;
        let source = &parsed["component"]["mcp"]["source"];
        assert_eq!(source["registry"].as_str(), Some("ghcr.io"));
        assert_eq!(source["package"].as_str(), Some("fastertools:ftl-auth-gateway"));
        assert_eq!(source["version"].as_str(), Some("0.0.6"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_ecr_to_spin_manifest() -> Result<()> {
        use crate::registry::EcrAdapter;
        
        let client = Client::new();
        let adapter = EcrAdapter::new("123456789".to_string(), "us-west-2".to_string());
        
        let components = adapter.get_registry_components(&client, "my-tool:2.1.0").await?;
        assert_eq!(components.registry_domain, "123456789.dkr.ecr.us-west-2.amazonaws.com");
        assert_eq!(components.package_name, "my-tool");
        assert_eq!(components.version, "2.1.0");
        
        // Generate spin.toml
        let manifest = SpinFormatter::generate_minimal_manifest(
            "ecr-app",
            "1.0.0",
            "worker",
            &components,
            Some("/api/work")
        )?;
        
        // Validate ECR domain format
        assert!(manifest.contains(r#"registry = "123456789.dkr.ecr.us-west-2.amazonaws.com""#));
        assert!(manifest.contains(r#"package = "my-tool""#));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_semver_validation_in_manifest() -> Result<()> {
        let client = Client::new();
        let adapter = get_registry_adapter(Some("docker"))?;
        
        // Test various semver formats
        let test_cases = vec![
            ("nginx:1", "1.0.0"),
            ("nginx:1.2", "1.2.0"), 
            ("nginx:1.2.3", "1.2.3"),
            ("nginx:v1.2.3", "1.2.3"),
            ("nginx:2.0.1-alpha", "2.0.1-alpha"),
        ];
        
        for (image, expected_version) in test_cases {
            let components = adapter.get_registry_components(&client, image).await?;
            assert_eq!(components.version, expected_version, "Failed for image: {}", image);
            
            // Verify it generates valid TOML
            let manifest = SpinFormatter::generate_minimal_manifest(
                "test",
                "0.1.0", 
                "comp",
                &components,
                None
            )?;
            
            // Should parse without error
            let _parsed: toml::Value = toml::from_str(&manifest)?;
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_semver_rejection() {
        let client = Client::new();
        let adapter = get_registry_adapter(Some("docker")).unwrap();
        
        // These should fail semver validation
        let invalid_tags = vec!["latest", "main", "stable", "dev"];
        
        for tag in invalid_tags {
            let result = adapter.get_registry_components(&client, &format!("nginx:{}", tag)).await;
            assert!(result.is_err(), "Expected error for tag: {}", tag);
        }
    }

    #[tokio::test] 
    async fn test_spin_manifest_structure_validation() -> Result<()> {
        let client = Client::new();
        let adapter = get_registry_adapter(Some("ghcr"))?;
        
        let components = adapter.get_registry_components(&client, "test-app:1.0.0").await?;
        
        let manifest = SpinFormatter::generate_minimal_manifest(
            "validation-test",
            "0.1.0",
            "main",
            &components,
            Some("/api/v1")
        )?;
        
        // Parse and validate complete structure
        let parsed: toml::Value = toml::from_str(&manifest)?;
        
        // Required top-level fields
        assert!(parsed.get("spin_manifest_version").is_some());
        assert!(parsed.get("application").is_some());
        assert!(parsed.get("trigger").is_some());
        assert!(parsed.get("component").is_some());
        
        // Application section
        let app = &parsed["application"];
        assert!(app.get("name").is_some());
        assert!(app.get("version").is_some());
        
        // Trigger section
        let trigger = &parsed["trigger"]["http"];
        assert!(trigger.is_array());
        let http_trigger = &trigger.as_array().unwrap()[0];
        assert!(http_trigger.get("route").is_some());
        assert!(http_trigger.get("component").is_some());
        
        // Component section
        let component = &parsed["component"]["main"];
        assert!(component.get("source").is_some());
        let source = &component["source"];
        assert!(source.get("registry").is_some());
        assert!(source.get("package").is_some());
        assert!(source.get("version").is_some());
        
        Ok(())
    }
}