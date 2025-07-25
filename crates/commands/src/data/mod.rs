use serde::{Deserialize, Serialize};

/// Represents a pre-built FTL tool with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique name identifier for the tool
    pub name: String,
    /// Category this tool belongs to (e.g., `basic_math`, `text_processing`)
    pub category: String,
    /// Human-readable description of what the tool does
    pub description: String,
    /// Container image name (without registry prefix)
    pub image_name: String,
    /// Searchable tags for tool discovery
    pub tags: Vec<String>,
}

/// Container for the complete tools manifest with all available tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsManifest {
    /// List of all available tools
    pub tools: Vec<Tool>,
}

impl ToolsManifest {
    /// Load tools manifest from TOML content
    pub fn from_toml(content: &str) -> anyhow::Result<Self> {
        let manifest: Self = toml::from_str(content)?;
        Ok(manifest)
    }

    /// Get all tools
    pub fn get_tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Get tools by category
    pub fn get_tools_by_category(&self, category: &str) -> Vec<&Tool> {
        self.tools
            .iter()
            .filter(|tool| tool.category == category)
            .collect()
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .tools
            .iter()
            .map(|tool| tool.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    /// Find tool by name
    pub fn find_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|tool| tool.name == name)
    }

    /// Search tools by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Tool> {
        self.tools
            .iter()
            .filter(|tool| tool.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get tools count
    pub fn count(&self) -> usize {
        self.tools.len()
    }
}

impl Tool {
    /// Get full registry image name with default registry
    pub fn get_image_ref(&self, registry: &str) -> String {
        format!("{}/{}", registry, self.image_name)
    }

    /// Check if tool has specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_manifest_parsing() {
        let toml_content = r#"
[[tools]]
name = "add"
category = "basic_math"
description = "Add two numbers together"
image_name = "ftl-tool-add"
tags = ["math", "arithmetic", "basic"]

[[tools]]
name = "uppercase"
category = "text_processing"
description = "Convert text to uppercase"
image_name = "ftl-tool-uppercase"
tags = ["text", "string", "formatting"]
"#;

        let manifest = ToolsManifest::from_toml(toml_content).unwrap();
        assert_eq!(manifest.count(), 2);

        let add_tool = manifest.find_tool("add").unwrap();
        assert_eq!(add_tool.category, "basic_math");
        assert_eq!(add_tool.description, "Add two numbers together");
        assert!(add_tool.has_tag("math"));

        let categories = manifest.get_categories();
        assert!(categories.contains(&"basic_math".to_string()));
        assert!(categories.contains(&"text_processing".to_string()));

        let math_tools = manifest.get_tools_by_category("basic_math");
        assert_eq!(math_tools.len(), 1);
        assert_eq!(math_tools[0].name, "add");
    }

    #[test]
    fn test_tool_image_ref() {
        let tool = Tool {
            name: "add".to_string(),
            category: "basic_math".to_string(),
            description: "Add two numbers".to_string(),
            image_name: "ftl-tool-add".to_string(),
            tags: vec!["math".to_string()],
        };

        assert_eq!(tool.get_image_ref("docker.io"), "docker.io/ftl-tool-add");
        assert_eq!(
            tool.get_image_ref("ghcr.io/fastertools"),
            "ghcr.io/fastertools/ftl-tool-add"
        );
    }

    #[test]
    fn test_search_by_tag() {
        let toml_content = r#"
[[tools]]
name = "add"
category = "basic_math"
description = "Add two numbers together"
image_name = "ftl-tool-add"
tags = ["math", "arithmetic", "basic"]

[[tools]]
name = "multiply"
category = "basic_math"
description = "Multiply two numbers"
image_name = "ftl-tool-multiply"
tags = ["math", "arithmetic", "basic"]

[[tools]]
name = "uppercase"
category = "text_processing"
description = "Convert text to uppercase"
image_name = "ftl-tool-uppercase"
tags = ["text", "string", "formatting"]
"#;

        let manifest = ToolsManifest::from_toml(toml_content).unwrap();
        let math_tools = manifest.search_by_tag("math");
        assert_eq!(math_tools.len(), 2);

        let text_tools = manifest.search_by_tag("text");
        assert_eq!(text_tools.len(), 1);
        assert_eq!(text_tools[0].name, "uppercase");
    }
}
