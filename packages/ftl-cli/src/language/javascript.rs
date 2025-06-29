use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result};

use crate::{
    language::{Language, LanguageSupport, PackageManager},
    manifest::Manifest,
    templates::Template,
};

pub struct JavaScriptSupport;

impl JavaScriptSupport {
    pub fn new() -> Self {
        Self
    }

    fn render_template(&self, template_str: &str, name: &str, description: &str) -> Result<String> {
        use handlebars::Handlebars;
        use serde_json::json;

        let handlebars = Handlebars::new();

        // Convert name to PascalCase for class name
        let tool_name_class = name
            .split(&['-', '_'][..])
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        // Get the SDK version from compile-time constant
        let sdk_version = env!("FTL_SDK_JS_VERSION");

        let data = json!({
            "name": name,
            "description": description,
            "tool_name_class": tool_name_class,
            "sdk_version": sdk_version,
        });

        handlebars
            .render_template(template_str, &data)
            .map_err(|e| anyhow::anyhow!("Template rendering failed: {}", e))
    }
}

impl LanguageSupport for JavaScriptSupport {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn new_project(
        &self,
        name: &str,
        description: &str,
        _template: &str,
        path: &Path,
    ) -> Result<()> {
        // Use spin new to create the project
        let output = Command::new("spin")
            .args([
                "new",
                "-t",
                "http-js",
                "-o",
                path.to_str().unwrap(),
                "--accept-defaults",
                name,
            ])
            .output()
            .context("Failed to run spin new. Is Spin installed?")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to create JavaScript project with spin new:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Move spin.toml to .ftl directory
        let spin_toml_src = path.join("spin.toml");
        let ftl_dir = path.join(".ftl");
        fs::create_dir_all(&ftl_dir)?;
        let spin_toml_dest = ftl_dir.join("spin.toml");

        if spin_toml_src.exists() {
            fs::rename(&spin_toml_src, &spin_toml_dest)
                .context("Failed to move spin.toml to .ftl directory")?;
        }

        // Overlay FTL-specific files

        // 1. Add ftl.toml
        let ftl_toml = self.render_template(
            include_str!("../templates/javascript/ftl.toml.hbs"),
            name,
            description,
        )?;
        fs::write(path.join("ftl.toml"), ftl_toml)?;

        // 2. Replace src/index.js with MCP implementation
        let index_js = self.render_template(
            include_str!("../templates/javascript/index.js.hbs"),
            name,
            description,
        )?;
        fs::write(path.join("src/index.js"), index_js)?;

        // 3. Update package.json to include @ftl/sdk-js
        let package_json = self.render_template(
            include_str!("../templates/javascript/package.json.hbs"),
            name,
            description,
        )?;
        fs::write(path.join("package.json"), package_json)?;

        // 4. Replace webpack.config.js
        let webpack_config = include_str!("../templates/javascript/webpack.config.js");
        fs::write(path.join("webpack.config.js"), webpack_config)?;

        // 5. Create test directory and test file
        fs::create_dir_all(path.join("test"))?;
        let test_js = self.render_template(
            include_str!("../templates/javascript/tool.test.js.hbs"),
            name,
            description,
        )?;
        fs::write(path.join("test/tool.test.js"), test_js)?;

        // 6. Add vitest.config.js
        let vitest_config = include_str!("../templates/javascript/vitest.config.js");
        fs::write(path.join("vitest.config.js"), vitest_config)?;

        Ok(())
    }

    fn build(&self, _manifest: &Manifest, path: &Path) -> Result<()> {
        // Run spin build with spin.toml from .ftl directory
        let spin_toml_path = path.join(".ftl/spin.toml");
        let output = Command::new("spin")
            .args(["build", "-f", spin_toml_path.to_str().unwrap()])
            .current_dir(path)
            .output()
            .context("Failed to run spin build")?;

        if !output.status.success() {
            anyhow::bail!("Build failed:\n{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn test(&self, _manifest: &Manifest, path: &Path) -> Result<()> {
        let pm = PackageManager::detect(path);
        let test_cmd = pm.run_command("test");
        let mut cmd_parts = test_cmd.split_whitespace();
        let output = Command::new(cmd_parts.next().unwrap())
            .args(cmd_parts)
            .current_dir(path)
            .output()
            .context("Failed to run tests")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            if !stdout.is_empty() {
                println!("\nOutput:\n{stdout}");
            }
            if !stderr.is_empty() {
                println!("\nErrors:\n{stderr}");
            }

            anyhow::bail!("Tests failed");
        }

        println!("{}", String::from_utf8_lossy(&output.stdout));
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

        if let Some(major) = version_parts.first().and_then(|v| v.parse::<u32>().ok())
            && major < 18
        {
            anyhow::bail!(
                "Node.js version {} is too old. Please install Node.js 18 or later.",
                version.trim()
            );
        }

        Ok(())
    }
}
