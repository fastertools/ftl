use crate::registry::RegistryComponents;
use anyhow::Result;
use serde_json::Value as JsonValue;

/// Formatter for Spin manifest TOML table format
#[allow(dead_code)]
pub struct SpinFormatter;

#[allow(dead_code)]
impl SpinFormatter {
    /// Convert registry components to Spin manifest source table format
    pub fn format_registry_source(components: &RegistryComponents) -> Result<JsonValue> {
        Ok(serde_json::json!({
            "registry": components.registry_domain,
            "package": components.package_name,
            "version": components.version
        }))
    }
    
    /// Generate a complete component section for a Spin manifest  
    pub fn format_component_section(
        component_name: &str,
        components: &RegistryComponents,
        allowed_hosts: Option<&[String]>,
        environment: Option<&[(&str, &str)]>,
    ) -> Result<String> {
        let mut toml_content = String::new();
        
        toml_content.push_str(&format!("[component.{}]\n", component_name));
        
        // Add source as inline table
        toml_content.push_str(&format!(
            "source = {{ registry = \"{}\", package = \"{}\", version = \"{}\" }}\n",
            components.registry_domain,
            components.package_name,
            components.version
        ));
        
        // Add optional fields
        if let Some(hosts) = allowed_hosts {
            toml_content.push_str("allowed_outbound_hosts = [");
            for (i, host) in hosts.iter().enumerate() {
                if i > 0 { toml_content.push_str(", "); }
                toml_content.push_str(&format!("\"{}\"", host));
            }
            toml_content.push_str("]\n");
        }
        
        if let Some(env_vars) = environment {
            toml_content.push_str("\n[component.{}.environment]\n");
            for (key, val) in env_vars {
                toml_content.push_str(&format!("{} = \"{}\"\n", key, val));
            }
        }
        
        Ok(toml_content)
    }
    
    /// Generate a minimal complete Spin manifest with registry source
    pub fn generate_minimal_manifest(
        app_name: &str,
        app_version: &str,
        component_name: &str,
        components: &RegistryComponents,
        route: Option<&str>,
    ) -> Result<String> {
        let mut toml_content = String::new();
        
        // Manifest version
        toml_content.push_str("spin_manifest_version = 2\n\n");
        
        // Application section
        toml_content.push_str("[application]\n");
        toml_content.push_str(&format!("name = \"{}\"\n", app_name));
        toml_content.push_str(&format!("version = \"{}\"\n\n", app_version));
        
        // HTTP trigger (if route provided)
        if let Some(route_path) = route {
            toml_content.push_str("[[trigger.http]]\n");
            toml_content.push_str(&format!("route = \"{}\"\n", route_path));
            toml_content.push_str(&format!("component = \"{}\"\n\n", component_name));
        }
        
        // Component section with registry source
        toml_content.push_str(&format!("[component.{}]\n", component_name));
        toml_content.push_str(&format!(
            "source = {{ registry = \"{}\", package = \"{}\", version = \"{}\" }}\n",
            components.registry_domain,
            components.package_name,
            components.version
        ));
        
        Ok(toml_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_registry_source() {
        let components = RegistryComponents {
            registry_domain: "ghcr.io".to_string(),
            package_name: "fastertools:ftl-auth-gateway".to_string(),
            version: "0.0.6".to_string(),
        };
        
        let result = SpinFormatter::format_registry_source(&components).unwrap();
        let expected = serde_json::json!({
            "registry": "ghcr.io",
            "package": "fastertools:ftl-auth-gateway", 
            "version": "0.0.6"
        });
        
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_docker_hub_official_format() {
        let components = RegistryComponents {
            registry_domain: "docker.io".to_string(),
            package_name: "library/nginx".to_string(),
            version: "1.21.0".to_string(),
        };
        
        let result = SpinFormatter::format_registry_source(&components).unwrap();
        let expected = serde_json::json!({
            "registry": "docker.io",
            "package": "library/nginx",
            "version": "1.21.0"
        });
        
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_generate_minimal_manifest() {
        let components = RegistryComponents {
            registry_domain: "ghcr.io".to_string(),
            package_name: "fastertools:ftl-auth-gateway".to_string(),
            version: "0.0.6".to_string(),
        };
        
        let manifest = SpinFormatter::generate_minimal_manifest(
            "test-app",
            "0.1.0", 
            "mcp",
            &components,
            Some("/hello")
        ).unwrap();
        
        // Basic checks that the manifest contains expected sections
        assert!(manifest.contains("spin_manifest_version = 2"));
        assert!(manifest.contains(r#"name = "test-app""#));
        assert!(manifest.contains(r#"registry = "ghcr.io""#));
        assert!(manifest.contains(r#"package = "fastertools:ftl-auth-gateway""#));
        assert!(manifest.contains(r#"version = "0.0.6""#));
        assert!(manifest.contains(r#"route = "/hello""#));
    }
}