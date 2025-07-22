use crate::formatters::spin::SpinFormatter;
use crate::registry::{get_registry_adapter, RegistryAdapter};
use anyhow::Result;
use reqwest::Client;

/// Golden file tests comparing generated spin.toml with known-good examples
#[cfg(test)]
mod golden_file_tests {
    use super::*;

    #[tokio::test]
    async fn test_ghcr_matches_ftl_mcp_demo_format() -> Result<()> {
        let client = Client::new();
        let adapter = get_registry_adapter(Some("ghcr"))?;
        
        // Extract components matching the golden file
        let components = adapter.get_registry_components(&client, "ftl-auth-gateway:0.0.6").await?;
        
        // Verify components match expected format from golden file
        assert_eq!(components.registry_domain, "ghcr.io");
        assert_eq!(components.package_name, "fastertools:ftl-auth-gateway");
        assert_eq!(components.version, "0.0.6");
        
        // Generate registry source in same format as golden file
        let source_json = SpinFormatter::format_registry_source(&components)?;
        assert_eq!(source_json["registry"], "ghcr.io");
        assert_eq!(source_json["package"], "fastertools:ftl-auth-gateway");
        assert_eq!(source_json["version"], "0.0.6");
        
        // Generate component section
        let component_toml = SpinFormatter::format_component_section(
            "mcp",
            &components,
            Some(&[
                "http://*.spin.internal".to_string(),
                "https://*.authkit.app".to_string(),
            ]),
            None,
        )?;
        
        // Verify the generated TOML contains expected elements
        assert!(component_toml.contains(r#"registry = "ghcr.io""#));
        assert!(component_toml.contains(r#"package = "fastertools:ftl-auth-gateway""#));
        assert!(component_toml.contains(r#"version = "0.0.6""#));
        assert!(component_toml.contains(r#""http://*.spin.internal""#));
        assert!(component_toml.contains(r#""https://*.authkit.app""#));
        
        // Parse and validate structure
        let parsed: toml::Value = toml::from_str(&component_toml)?;
        let mcp_component = &parsed["component"]["mcp"];
        assert!(mcp_component.get("source").is_some());
        assert!(mcp_component.get("allowed_outbound_hosts").is_some());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_complete_manifest_structure_matches_golden() -> Result<()> {
        let client = Client::new();
        let adapter = get_registry_adapter(Some("ghcr"))?;
        
        let components = adapter.get_registry_components(&client, "ftl-auth-gateway:0.0.6").await?;
        
        // Generate a minimal manifest similar to the golden file structure
        let manifest = SpinFormatter::generate_minimal_manifest(
            "ftl-mcp-demo",
            "0.1.0",
            "mcp",
            &components,
            None, // No HTTP trigger in the golden file
        )?;
        
        // Parse the generated manifest
        let parsed: toml::Value = toml::from_str(&manifest)?;
        
        // Validate structure matches golden file requirements
        assert_eq!(parsed["spin_manifest_version"].as_integer(), Some(2));
        
        let app = &parsed["application"];
        assert_eq!(app["name"].as_str(), Some("ftl-mcp-demo"));
        assert_eq!(app["version"].as_str(), Some("0.1.0"));
        
        let component = &parsed["component"]["mcp"];
        let source = &component["source"];
        assert_eq!(source["registry"].as_str(), Some("ghcr.io"));
        assert_eq!(source["package"].as_str(), Some("fastertools:ftl-auth-gateway"));
        assert_eq!(source["version"].as_str(), Some("0.0.6"));
        
        Ok(())
    }

    #[test]
    fn test_golden_file_format_compatibility() -> Result<()> {
        // Load the golden file
        let golden_content = include_str!("golden_files/ftl-mcp-demo.toml");
        
        // Parse the golden file
        let golden: toml::Value = toml::from_str(golden_content)?;
        
        // Extract source information from golden file
        let golden_component = &golden["component"]["mcp"];
        let golden_source = &golden_component["source"];
        
        // Verify our understanding of the format is correct
        assert_eq!(golden_source["registry"].as_str(), Some("ghcr.io"));
        assert_eq!(golden_source["package"].as_str(), Some("fastertools:ftl-auth-gateway"));
        assert_eq!(golden_source["version"].as_str(), Some("0.0.6"));
        
        // Verify colon separator is used (not slash)
        let package = golden_source["package"].as_str().unwrap();
        assert!(package.contains(':'));
        assert!(!package.contains('/'));
        
        // Verify allowed_outbound_hosts structure
        let hosts = golden_component["allowed_outbound_hosts"].as_array().unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].as_str(), Some("http://*.spin.internal"));
        assert_eq!(hosts[1].as_str(), Some("https://*.authkit.app"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_registry_to_golden_format_conversion() -> Result<()> {
        let client = Client::new();
        
        // Test multiple registry types can produce valid golden file format
        let test_cases = vec![
            ("ghcr", "fastertools:ftl-auth-gateway:0.0.6"),
            ("docker", "library/nginx:1.21.0"),
        ];
        
        for (registry_name, image) in test_cases {
            let adapter = get_registry_adapter(Some(registry_name))?;
            let components = adapter.get_registry_components(&client, image).await?;
            
            // Generate minimal manifest
            let manifest = SpinFormatter::generate_minimal_manifest(
                "test-app",
                "1.0.0",
                "main",
                &components,
                None,
            )?;
            
            // Parse and verify structure
            let parsed: toml::Value = toml::from_str(&manifest)?;
            
            // All manifests should have the same basic structure
            assert_eq!(parsed["spin_manifest_version"].as_integer(), Some(2));
            assert!(parsed["application"].is_table());
            assert!(parsed["component"]["main"]["source"].is_table());
            
            // Source should have required fields
            let source = &parsed["component"]["main"]["source"];
            assert!(source["registry"].is_str());
            assert!(source["package"].is_str());
            assert!(source["version"].is_str());
        }
        
        Ok(())
    }
}