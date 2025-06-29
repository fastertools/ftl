use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::language::{Language, LanguageSupport, PackageManager};
use crate::manifest::Manifest;
use crate::templates::Template;

pub struct JavaScriptSupport {
    package_manager: Option<PackageManager>,
}

impl JavaScriptSupport {
    pub fn new() -> Self {
        Self {
            package_manager: None,
        }
    }

    fn detect_package_manager(&mut self, path: &Path) -> PackageManager {
        if let Some(pm) = self.package_manager {
            return pm;
        }
        let pm = PackageManager::detect(path);
        self.package_manager = Some(pm);
        pm
    }

    fn render_template(&self, template_str: &str, name: &str, description: &str) -> Result<String> {
        use handlebars::Handlebars;
        use serde_json::json;
        
        let handlebars = Handlebars::new();
        
        // Convert name to PascalCase for class name
        let tool_name_class = name.split(&['-', '_'][..])
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();
        
        let data = json!({
            "name": name,
            "description": description,
            "tool_name_class": tool_name_class,
        });
        
        handlebars.render_template(template_str, &data)
            .map_err(|e| anyhow::anyhow!("Template rendering failed: {}", e))
    }

}

impl LanguageSupport for JavaScriptSupport {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn new_project(&self, name: &str, description: &str, _template: &str, path: &Path) -> Result<()> {
        // Use spin new to create the project
        let output = Command::new("spin")
            .args(&["new", "-t", "http-js", "-o", path.to_str().unwrap(), "--accept-defaults", name])
            .output()
            .context("Failed to run spin new. Is Spin installed?")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to create JavaScript project with spin new:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Overlay FTL-specific files
        
        // 1. Add ftl.toml
        let ftl_toml = self.render_template(include_str!("../templates/javascript/ftl.toml.hbs"), name, description)?;
        fs::write(path.join("ftl.toml"), ftl_toml)?;
        
        // 2. Replace src/index.js with MCP implementation
        let index_js = self.render_template(include_str!("../templates/javascript/index.js.hbs"), name, description)?;
        fs::write(path.join("src/index.js"), index_js)?;
        
        // 3. Update package.json to include @ftl/sdk-js
        let package_json = self.render_template(include_str!("../templates/javascript/package.json.hbs"), name, description)?;
        fs::write(path.join("package.json"), package_json)?;
        
        // 4. Replace webpack.config.js
        let webpack_config = include_str!("../templates/javascript/webpack.config.js");
        fs::write(path.join("webpack.config.js"), webpack_config)?;
        
        // 5. Create test directory and test file
        fs::create_dir_all(path.join("test"))?;
        let test_js = self.render_template(include_str!("../templates/javascript/tool.test.js.hbs"), name, description)?;
        fs::write(path.join("test/tool.test.js"), test_js)?;

        Ok(())
    }

    fn build(&self, _manifest: &Manifest, path: &Path) -> Result<()> {
        // Simply delegate to spin build
        let output = Command::new("spin")
            .args(&["build"])
            .current_dir(path)
            .output()
            .context("Failed to run spin build")?;

        if !output.status.success() {
            anyhow::bail!(
                "Build failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    fn test(&self, _manifest: &Manifest, path: &Path) -> Result<()> {
        let mut support = JavaScriptSupport::new();
        let pm = support.detect_package_manager(path);

        let test_cmd = pm.run_command("test");
        let mut cmd_parts = test_cmd.split_whitespace();
        let output = Command::new(cmd_parts.next().unwrap())
            .args(cmd_parts)
            .current_dir(path)
            .output()
            .context("Failed to run tests")?;

        if !output.status.success() {
            anyhow::bail!(
                "Tests failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    fn get_templates(&self) -> Vec<Template> {
        vec![Template {
            name: "default".to_string(),
            description: "Default JavaScript FTL tool template".to_string(),
            language: Language::JavaScript,
        }]
    }

    fn validate_environment(&self) -> Result<()> {
        // Check if Node.js is installed
        let output = Command::new("node")
            .arg("--version")
            .output()
            .context("Node.js is not installed. Please install Node.js from https://nodejs.org")?;

        let version = String::from_utf8_lossy(&output.stdout);
        let version_parts: Vec<&str> = version.trim().trim_start_matches('v').split('.').collect();
        
        if let Some(major) = version_parts.first().and_then(|v| v.parse::<u32>().ok()) {
            if major < 18 {
                anyhow::bail!(
                    "Node.js version {} is too old. Please install Node.js 18 or later.",
                    version.trim()
                );
            }
        }

        Ok(())
    }
}