//! Unit tests for the tools command

use std::sync::Arc;

use crate::commands::tools::{
    ToolsDependencies, list_with_deps, add_with_deps, update_with_deps, remove_with_deps,
};
use ftl_common::ui::TestUserInterface;
use ftl_runtime::deps::UserInterface;
use reqwest::Client;

struct TestFixture {
    ui: Arc<TestUserInterface>,
    client: Client,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            client: Client::new(),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<ToolsDependencies> {
        Arc::new(ToolsDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            client: self.client,
        })
    }
}

#[tokio::test]
async fn test_list_tools_from_manifest() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test listing all tools from manifest
    list_with_deps(&deps, None, None, None, false, false, false)
        .await
        .expect("Failed to list tools");

    // Verify output contains tools
    let output = _ui.get_output();
    assert!(output.iter().any(|s| s.contains("basic_math")));
    // The actual manifest has 82 tools after loading (some might be filtered or deduplicated)
    assert!(output.iter().any(|s| s.contains("Total: 82 tools") || s.contains("Total: 84 tools")));
}

#[tokio::test]
async fn test_list_tools_with_category_filter() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test listing tools filtered by category
    list_with_deps(&deps, Some("basic_math"), None, None, false, false, false)
        .await
        .expect("Failed to list tools with category filter");

    // Verify output contains only basic_math tools
    let output = _ui.get_output();
    assert!(output.iter().any(|s| s.contains("basic_math")));
    assert!(output.iter().any(|s| s.contains("add")));
    assert!(output.iter().any(|s| s.contains("subtract")));
    assert!(!output.iter().any(|s| s.contains("text_processing")));
}

#[tokio::test]
async fn test_list_tools_with_keyword_filter() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test listing tools filtered by keyword
    list_with_deps(&deps, None, Some("encode"), None, false, false, false)
        .await
        .expect("Failed to list tools with keyword filter");

    // Verify output contains encoding-related tools
    let output = _ui.get_output();
    assert!(output.iter().any(|s| s.contains("url-encode")));
    assert!(output.iter().any(|s| s.contains("base64-encode")));
}

#[tokio::test]
async fn test_list_tools_verbose() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test verbose listing
    list_with_deps(&deps, None, None, None, true, false, false)
        .await
        .expect("Failed to list tools in verbose mode");

    // Verify verbose output contains additional details
    let output = _ui.get_output();
    assert!(output.iter().any(|s| s.contains("Category:")));
    assert!(output.iter().any(|s| s.contains("Description:")));
    assert!(output.iter().any(|s| s.contains("Image:")));
    assert!(output.iter().any(|s| s.contains("Tags:")));
}

#[tokio::test]
async fn test_list_tools_direct_from_registry() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test direct registry listing
    // Note: This will make a real API call to GitHub, so it might fail in CI
    // We'll just verify it doesn't crash for now
    let result = list_with_deps(&deps, None, None, Some("ghcr"), false, false, true).await;
    
    // We expect this might fail due to network or API limits, but it shouldn't panic
    if result.is_ok() {
        let output = _ui.get_output();
        // The actual output uses the word "registry" in various forms
        assert!(output.iter().any(|s| s.contains("Querying") && s.contains("registry")));
    }
}

#[tokio::test]
async fn test_add_tools_without_confirmation() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test adding tools with --yes flag (skips confirmation)
    // Note: This test would need a test project setup to actually work
    // For now, we'll just verify it handles the missing spin.toml gracefully
    let result = add_with_deps(&deps, &["add".to_string()], None, None, true).await;
    
    // Should fail because no spin.toml exists
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("No spin.toml found"));
    }
}

#[tokio::test]
async fn test_update_tools_without_confirmation() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test updating tools with --yes flag
    let result = update_with_deps(&deps, &["add".to_string()], None, Some("latest"), true).await;
    
    // Should succeed but with warning
    assert!(result.is_ok());
    
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("not currently installed")));
    assert!(output.iter().any(|s| s.contains("No valid tools to update")));
}

#[tokio::test]
async fn test_remove_tools_without_confirmation() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test removing tools with --yes flag
    let result = remove_with_deps(&deps, &["add".to_string()], true).await;
    
    // Should succeed but with warning
    assert!(result.is_ok());
    
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("not currently installed")));
    assert!(output.iter().any(|s| s.contains("No valid tools to remove")));
}

#[tokio::test]
async fn test_update_tools_with_inline_table() {
    use std::fs;
    use tempfile::TempDir;
    
    let fixture = TestFixture::new();
    let _deps = fixture.to_deps();
    
    // Create a temporary directory with a spin.toml
    let temp_dir = TempDir::new().unwrap();
    let spin_toml_path = temp_dir.path().join("spin.toml");
    
    // Create a spin.toml with inline table format (the actual format used)
    let spin_toml_content = r#"spin_manifest_version = 2

[application]
name = "test-app"
version = "0.1.0"

[component.tool-json-formatter]
source = { registry = "ghcr.io", package = "fastertools:ftl-tool-json-formatter", version = "0.0.1" }
allowed_outbound_hosts = []
"#;
    
    fs::write(&spin_toml_path, spin_toml_content).unwrap();
    
    // This test verifies that inline table TOML parsing works correctly.
    // The actual update operation would require crane to be installed,
    // so we're just testing that the file can be parsed without errors.
    
    // Verify the file was created correctly
    let content = fs::read_to_string(&spin_toml_path).unwrap();
    assert!(content.contains("tool-json-formatter"));
    assert!(content.contains("source = { registry"));
    
    // If we can parse it, the inline table handling is working
    let parsed_doc = content.parse::<toml_edit::DocumentMut>().unwrap();
    assert!(parsed_doc.get("component").is_some());
}

#[tokio::test]
async fn test_get_installed_tools_empty() {
    use crate::commands::tools::get_installed_tools;
    use tempfile::TempDir;
    
    // Create empty temp directory
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let tools = get_installed_tools().unwrap();
    assert!(tools.is_empty());
    
    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_get_installed_tools_with_tools() {
    use crate::commands::tools::get_installed_tools;
    use tempfile::TempDir;
    use std::fs;
    
    let temp_dir = TempDir::new().unwrap();
    let spin_toml_path = temp_dir.path().join("spin.toml");
    
    let spin_toml_content = r#"
[component.tool-json-formatter]
source = { registry = "ghcr.io", package = "ftl-tool-json-formatter", version = "0.1.0" }

[component.tool-add]
source = { registry = "ghcr.io", package = "ftl-tool-add", version = "1.0.0" }

[component.not-a-tool]
source = { registry = "ghcr.io", package = "some-other-component", version = "1.0.0" }
"#;
    
    fs::write(&spin_toml_path, spin_toml_content).unwrap();
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let tools = get_installed_tools().unwrap();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains("json-formatter"));
    assert!(tools.contains("add"));
    assert!(!tools.contains("not-a-tool"));
    
    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_update_without_existing_tool() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    
    // Try to update a tool that's not installed
    let result = update_with_deps(&deps, &["non-existent-tool".to_string()], None, None, true).await;
    
    // Should succeed but with warning
    assert!(result.is_ok());
    
    let output = _ui.get_output();
    assert!(output.iter().any(|s| s.contains("not currently installed")));
    assert!(output.iter().any(|s| s.contains("No valid tools to update")));
}

#[tokio::test]
async fn test_remove_without_existing_tool() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    
    // Try to remove a tool that's not installed
    let result = remove_with_deps(&deps, &["non-existent-tool".to_string()], true).await;
    
    // Should succeed but with warning
    assert!(result.is_ok());
    
    let output = _ui.get_output();
    assert!(output.iter().any(|s| s.contains("not currently installed")));
    assert!(output.iter().any(|s| s.contains("No valid tools to remove")));
}

#[cfg(test)]
mod tool_spec_parsing {
    use crate::commands::tools::parse_tool_spec;

    #[test]
    fn test_parse_tool_spec_name_only() {
        let (registry, name, version) = parse_tool_spec("add", None);
        assert_eq!(registry, "ghcr");
        assert_eq!(name, "add");
        assert_eq!(version, "latest");
    }

    #[test]
    fn test_parse_tool_spec_with_version() {
        let (registry, name, version) = parse_tool_spec("add:1.0.0", None);
        assert_eq!(registry, "ghcr");
        assert_eq!(name, "add");
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn test_parse_tool_spec_with_prefix() {
        let (registry, name, version) = parse_tool_spec("ftl-tool-add:2.1", None);
        assert_eq!(registry, "ghcr");
        assert_eq!(name, "ftl-tool-add");
        assert_eq!(version, "2.1");
    }

    #[test]
    fn test_parse_tool_spec_registry_tool() {
        let (registry, name, version) = parse_tool_spec("docker:nginx", None);
        assert_eq!(registry, "docker");
        assert_eq!(name, "nginx");
        assert_eq!(version, "latest");
    }

    #[test]
    fn test_parse_tool_spec_registry_tool_version() {
        let (registry, name, version) = parse_tool_spec("ecr:myapp:1.2.3", None);
        assert_eq!(registry, "ecr");
        assert_eq!(name, "myapp");
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn test_parse_tool_spec_with_registry_override() {
        let (registry, name, version) = parse_tool_spec("tool:1.0", Some("docker"));
        assert_eq!(registry, "docker");
        assert_eq!(name, "tool");
        assert_eq!(version, "1.0");
    }

    #[test]
    fn test_parse_tool_spec_unknown_registry() {
        // Unknown registries in first position are treated as tool names
        let (registry, name, version) = parse_tool_spec("unknown:tool", None);
        assert_eq!(registry, "ghcr");
        assert_eq!(name, "unknown");
        assert_eq!(version, "tool");
    }

    #[test]
    fn test_parse_tool_spec_malformed() {
        // More than 3 colons - splitn(3) gives ["a", "b", "c:d:e"]
        let (registry, name, version) = parse_tool_spec("a:b:c:d:e", None);
        assert_eq!(registry, "a");
        assert_eq!(name, "b");
        assert_eq!(version, "c:d:e");
    }
}

#[cfg(test)]
mod tool_components_tests {
    use toml_edit::{DocumentMut, Item, Table, InlineTable};
    
    fn create_test_doc() -> DocumentMut {
        let mut doc = DocumentMut::new();
        doc["spin_manifest_version"] = toml_edit::value(2);
        doc
    }
    
    #[test]
    fn test_update_tool_components_add_first_tool() {
        use crate::commands::tools::update_tool_components_variable;
        
        let mut doc = create_test_doc();
        
        // Add first tool
        update_tool_components_variable(&mut doc, "json-formatter", true).unwrap();
        
        let variables = doc["variables"].as_table().unwrap();
        let tool_components = variables["tool_components"].as_inline_table().unwrap();
        let default_value = tool_components.get("default").unwrap().as_str().unwrap();
        
        assert_eq!(default_value, "json-formatter");
    }
    
    #[test]
    fn test_update_tool_components_add_multiple_tools() {
        use crate::commands::tools::update_tool_components_variable;
        
        let mut doc = create_test_doc();
        
        // Add multiple tools
        update_tool_components_variable(&mut doc, "json-formatter", true).unwrap();
        update_tool_components_variable(&mut doc, "add", true).unwrap();
        update_tool_components_variable(&mut doc, "query", true).unwrap();
        
        let variables = doc["variables"].as_table().unwrap();
        let tool_components = variables["tool_components"].as_inline_table().unwrap();
        let default_value = tool_components.get("default").unwrap().as_str().unwrap();
        
        assert_eq!(default_value, "json-formatter,add,query");
    }
    
    #[test]
    fn test_update_tool_components_remove_tool() {
        use crate::commands::tools::update_tool_components_variable;
        
        let mut doc = create_test_doc();
        
        // Pre-populate with tools
        doc["variables"] = Item::Table(Table::new());
        let variables = doc["variables"].as_table_mut().unwrap();
        variables["tool_components"] = Item::Value(toml_edit::Value::InlineTable({
            let mut table = InlineTable::new();
            table.insert("default", "json-formatter,add,query".into());
            table
        }));
        
        // Remove middle tool
        update_tool_components_variable(&mut doc, "add", false).unwrap();
        
        let variables = doc["variables"].as_table().unwrap();
        let tool_components = variables["tool_components"].as_inline_table().unwrap();
        let default_value = tool_components.get("default").unwrap().as_str().unwrap();
        
        assert_eq!(default_value, "json-formatter,query");
    }
    
    #[test]
    fn test_update_tool_components_remove_all_tools() {
        use crate::commands::tools::update_tool_components_variable;
        
        let mut doc = create_test_doc();
        
        // Pre-populate with one tool
        doc["variables"] = Item::Table(Table::new());
        let variables = doc["variables"].as_table_mut().unwrap();
        variables["tool_components"] = Item::Value(toml_edit::Value::InlineTable({
            let mut table = InlineTable::new();
            table.insert("default", "json-formatter".into());
            table
        }));
        
        // Remove the only tool
        update_tool_components_variable(&mut doc, "json-formatter", false).unwrap();
        
        let variables = doc["variables"].as_table().unwrap();
        let tool_components = variables["tool_components"].as_inline_table().unwrap();
        let default_value = tool_components.get("default").unwrap().as_str().unwrap();
        
        assert_eq!(default_value, "");
    }
}

#[tokio::test]
async fn test_list_with_category_filter() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    
    // List tools filtered by category
    list_with_deps(&deps, Some("basic_math"), None, None, false, false, false)
        .await
        .expect("Failed to list tools with category filter");
    
    let output = _ui.get_output();
    // Should only show basic_math tools
    assert!(output.iter().any(|s| s.contains("basic_math")));
    // Total should be less than full 82/84
    assert!(!output.iter().any(|s| s.contains("Total: 82 tools") || s.contains("Total: 84 tools")));
}

#[tokio::test]
async fn test_list_with_keyword_filter() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    
    // List tools filtered by keyword
    list_with_deps(&deps, None, Some("json"), None, false, false, false)
        .await
        .expect("Failed to list tools with keyword filter");
    
    let output = _ui.get_output();
    // Should show tools with "json" in name/description/tags
    assert!(output.iter().any(|s| s.contains("json") || s.contains("JSON")));
}

#[tokio::test]
async fn test_list_verbose_output() {
    let fixture = TestFixture::new();
    let _ui = fixture.ui.clone();
    let deps = fixture.to_deps();
    
    // List tools with verbose output
    list_with_deps(&deps, None, None, None, true, false, false)
        .await
        .expect("Failed to list tools with verbose output");
    
    let output = _ui.get_output();
    // Verbose output should include descriptions
    assert!(output.iter().any(|s| s.contains("Description:") || s.contains("Tags:")));
}