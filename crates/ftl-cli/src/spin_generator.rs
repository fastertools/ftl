use std::path::Path;

use anyhow::Result;

use crate::{
    language::Language,
    manifest::{ToolManifest, ToolkitManifest},
};

pub struct SpinConfig {
    pub content: String,
}

impl SpinConfig {
    pub fn from_tool(manifest: &ToolManifest, wasm_path: &Path) -> Result<Self> {
        // Build command based on language and profile
        let build_command = match manifest.tool.language {
            Language::Rust => {
                // Note: Cargo uses "dev" as the profile name but "debug" as the directory name
                let cargo_profile = if manifest.build.profile == "debug" {
                    "dev"
                } else {
                    &manifest.build.profile
                };
                format!(
                    "cargo build --target wasm32-wasip1 --profile {}{}",
                    cargo_profile,
                    if manifest.build.features.is_empty() {
                        String::new()
                    } else {
                        format!(" --features {}", manifest.build.features.join(","))
                    }
                )
            }
            Language::JavaScript => {
                // For JS/TS, we use npm run build which is defined in package.json
                "npm run build".to_string()
            }
        };

        let content = format!(
            r#"spin_manifest_version = 2

[application]
name = "{}"
version = "{}"
description = "{}"
authors = ["FTL Tool"]

[[trigger.http]]
route = "/mcp/..."
component = "{}"

[component.{}]
source = "{}"
allowed_outbound_hosts = {:?}
build.command = "{}"
"#,
            manifest.tool.name,
            manifest.tool.version,
            manifest.tool.description,
            manifest.tool.name,
            manifest.tool.name,
            wasm_path.display(),
            manifest.runtime.allowed_hosts,
            build_command
        );

        Ok(Self { content })
    }

    pub fn from_toolkit(
        manifest: &ToolkitManifest,
        tool_paths: &[(String, String)],
    ) -> Result<Self> {
        let mut content = format!(
            r#"spin_manifest_version = 2

[application]
name = "{}"
version = "{}"
description = "{}"
authors = ["FTL Toolkit"]
"#,
            manifest.toolkit.name, manifest.toolkit.version, manifest.toolkit.description
        );

        // Add triggers and components for each tool
        for tool in &manifest.tools {
            content.push_str(&format!(
                r#"
[[trigger.http]]
route = "{}/..."
component = "{}"
"#,
                tool.route, tool.name
            ));
        }

        // Add component definitions
        for (tool_name, wasm_path) in tool_paths {
            let allowed_hosts = manifest
                .tools
                .iter()
                .find(|t| t.name == *tool_name)
                .map(|_| Vec::<String>::new())
                .unwrap_or_default();

            content.push_str(&format!(
                r#"
[component.{tool_name}]
source = "{wasm_path}"
allowed_outbound_hosts = {allowed_hosts:?}
"#
            ));
        }

        Ok(Self { content })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        std::fs::write(path, &self.content)?;
        Ok(())
    }
}

pub fn generate_development_config(tool_name: &str, port: u16, wasm_path: &Path) -> Result<String> {
    Ok(format!(
        r#"# Development configuration for {} on port {}
spin_manifest_version = 2

[application]
name = "{}-dev"
version = "0.0.0"
description = "Development server for {}"
authors = ["FTL Developer"]

[[trigger.http]]
route = "/mcp/..."
component = "{}"

[component.{}]
source = "{}"
allowed_outbound_hosts = []
build.command = "cargo build --target wasm32-wasip1 --release"
build.watch = ["src/**/*.rs", "Cargo.toml", "ftl.toml"]
"#,
        tool_name,
        port,
        tool_name,
        tool_name,
        tool_name,
        tool_name,
        wasm_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        language::Language,
        manifest::{BuildConfig, OptimizationConfig, RuntimeConfig, ToolConfig, ToolManifest},
    };

    #[test]
    fn test_tool_spin_generation() {
        let manifest = ToolManifest {
            tool: ToolConfig {
                name: "test_tool".to_string(),
                version: "1.0.0".to_string(),
                description: "Test tool".to_string(),
                language: Language::Rust,
            },
            build: BuildConfig::default(),
            optimization: OptimizationConfig::default(),
            runtime: RuntimeConfig::default(),
        };

        let wasm_path = Path::new("test.wasm");
        let config = SpinConfig::from_tool(&manifest, wasm_path).unwrap();

        assert!(config.content.contains("test_tool"));
        assert!(config.content.contains("test.wasm"));
    }
}
