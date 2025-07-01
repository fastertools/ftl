use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result};

use crate::{
    common::spin_installer::check_and_install_spin,
    language::{LanguageSupport, PackageManager},
    manifest::Manifest,
};

pub struct TypeScriptSupport;

impl TypeScriptSupport {
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
        let sdk_version = env!("FTL_SDK_TS_VERSION");

        let data = json!({
            "name": name,
            "description": description,
            "tool_name_class": tool_name_class,
            "sdk_version": sdk_version,
        });

        handlebars
            .render_template(template_str, &data)
            .map_err(|e| anyhow::anyhow!("Template rendering failed: {e}"))
    }
}

impl LanguageSupport for TypeScriptSupport {
    fn new_project(
        &self,
        name: &str,
        description: &str,
        _template: &str,
        path: &Path,
    ) -> Result<()> {
        // Get spin path using blocking runtime
        let spin_path = tokio::runtime::Handle::try_current()
            .ok()
            .and_then(|handle| {
                tokio::task::block_in_place(|| handle.block_on(check_and_install_spin()).ok())
            })
            .unwrap_or_else(|| {
                // If no runtime exists, create one
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                rt.block_on(check_and_install_spin())
                    .expect("Failed to install Spin")
            });

        // Use spin new to create the project with TypeScript template
        let output = Command::new(&spin_path)
            .args([
                "new",
                "-t",
                "http-ts",
                "-o",
                path.to_str().unwrap(),
                "--accept-defaults",
                name,
            ])
            .output()
            .context("Failed to run spin new")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to create TypeScript project with spin new:\n{}",
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
            include_str!("../templates/typescript/ftl.toml.hbs"),
            name,
            description,
        )?;
        fs::write(path.join("ftl.toml"), ftl_toml)?;

        // 2. Replace src/index.ts with MCP implementation
        let index_ts = self.render_template(
            include_str!("../templates/typescript/index.ts.hbs"),
            name,
            description,
        )?;
        fs::write(path.join("src/index.ts"), index_ts)?;

        // 3. Update package.json to include @ftl/sdk-js and TypeScript dependencies
        let package_json = self.render_template(
            include_str!("../templates/typescript/package.json.hbs"),
            name,
            description,
        )?;
        fs::write(path.join("package.json"), package_json)?;

        // 4. Replace webpack.config.js with TypeScript version
        let webpack_config = include_str!("../templates/typescript/webpack.config.js");
        fs::write(path.join("webpack.config.js"), webpack_config)?;

        // 5. Add tsconfig.json
        let tsconfig = include_str!("../templates/typescript/tsconfig.json");
        fs::write(path.join("tsconfig.json"), tsconfig)?;

        // 6. Create test directory and test file
        fs::create_dir_all(path.join("test"))?;
        let test_ts = self.render_template(
            include_str!("../templates/typescript/tool.test.ts.hbs"),
            name,
            description,
        )?;
        fs::write(path.join("test/tool.test.ts"), test_ts)?;

        // 7. Add vitest.config.ts
        let vitest_config = include_str!("../templates/typescript/vitest.config.ts");
        fs::write(path.join("vitest.config.ts"), vitest_config)?;

        Ok(())
    }

    fn build(&self, _manifest: &Manifest, path: &Path) -> Result<()> {
        // Get spin path using blocking runtime
        let spin_path = tokio::runtime::Handle::try_current()
            .ok()
            .and_then(|handle| {
                tokio::task::block_in_place(|| handle.block_on(check_and_install_spin()).ok())
            })
            .unwrap_or_else(|| {
                // If no runtime exists, create one
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                rt.block_on(check_and_install_spin())
                    .expect("Failed to install Spin")
            });

        // Run spin build with spin.toml from .ftl directory
        let spin_toml_path = path.join(".ftl/spin.toml");

        // Ensure the spin.toml exists
        if !spin_toml_path.exists() {
            let display = spin_toml_path.display();
            anyhow::bail!("spin.toml not found at: {display}");
        }

        // Convert to absolute path to avoid issues with relative paths
        let absolute_spin_toml = spin_toml_path
            .canonicalize()
            .context("Failed to resolve spin.toml path")?;

        let output = Command::new(&spin_path)
            .args(["build", "-f", absolute_spin_toml.to_str().unwrap()])
            .current_dir(path)
            .output()
            .context("Failed to run spin build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Build failed:\n{stderr}");
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

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{stdout}");
        Ok(())
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
                let version = version.trim();
                anyhow::bail!(
                    "Node.js version {version} is too old. Please install Node.js 18 or later."
                );
            }
        }

        Ok(())
    }
}
