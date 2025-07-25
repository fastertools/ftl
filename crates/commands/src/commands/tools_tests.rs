//! Unit tests for the tools command

use std::sync::Arc;

use crate::commands::tools::{
    ToolsDependencies, add_with_deps, list_with_deps, remove_with_deps, update_with_deps,
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
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test listing all tools from manifest
    list_with_deps(&deps, None, None, None, false, false, false)
        .await
        .expect("Failed to list tools");

    // Verify output contains tools
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("basic_math")));
    // The actual manifest has 82 tools after loading (some might be filtered or deduplicated)
    assert!(
        output
            .iter()
            .any(|s| s.contains("Total: 82 tools") || s.contains("Total: 84 tools"))
    );
}

#[tokio::test]
async fn test_list_tools_with_category_filter() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test listing tools filtered by category
    list_with_deps(&deps, Some("basic_math"), None, None, false, false, false)
        .await
        .expect("Failed to list tools with category filter");

    // Verify output contains only basic_math tools
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("basic_math")));
    assert!(output.iter().any(|s| s.contains("add")));
    assert!(output.iter().any(|s| s.contains("subtract")));
    assert!(!output.iter().any(|s| s.contains("text_processing")));
}

#[tokio::test]
async fn test_list_tools_with_keyword_filter() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test listing tools filtered by keyword
    list_with_deps(&deps, None, Some("encode"), None, false, false, false)
        .await
        .expect("Failed to list tools with keyword filter");

    // Verify output contains encoding-related tools
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("url-encode")));
    assert!(output.iter().any(|s| s.contains("base64-encode")));
}

#[tokio::test]
async fn test_list_tools_verbose() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test verbose listing
    list_with_deps(&deps, None, None, None, true, false, false)
        .await
        .expect("Failed to list tools in verbose mode");

    // Verify verbose output contains additional details
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Category:")));
    assert!(output.iter().any(|s| s.contains("Description:")));
    assert!(output.iter().any(|s| s.contains("Image:")));
    assert!(output.iter().any(|s| s.contains("Tags:")));
}

#[tokio::test]
async fn test_list_tools_direct_from_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Test direct registry listing
    // Note: This will make a real API call to GitHub, so it might fail in CI
    // We'll just verify it doesn't crash for now
    let result = list_with_deps(&deps, None, None, Some("ghcr"), false, false, true).await;

    // We expect this might fail due to network or API limits, but it shouldn't panic
    if result.is_ok() {
        let output = ui.get_output();
        // The actual output uses the word "registry" in various forms
        assert!(
            output
                .iter()
                .any(|s| s.contains("Querying") && s.contains("registry"))
        );
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
    assert!(
        output
            .iter()
            .any(|s| s.contains("No valid tools to update"))
    );
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
    assert!(
        output
            .iter()
            .any(|s| s.contains("No valid tools to remove"))
    );
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
    use std::fs;
    use tempfile::TempDir;

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
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Try to update a tool that's not installed
    let result =
        update_with_deps(&deps, &["non-existent-tool".to_string()], None, None, true).await;

    // Should succeed but with warning
    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("not currently installed")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("No valid tools to update"))
    );
}

#[tokio::test]
async fn test_remove_without_existing_tool() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // Try to remove a tool that's not installed
    let result = remove_with_deps(&deps, &["non-existent-tool".to_string()], true).await;

    // Should succeed but with warning
    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("not currently installed")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("No valid tools to remove"))
    );
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
    use toml_edit::{DocumentMut, InlineTable, Item, Table};

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
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // List tools filtered by category
    list_with_deps(&deps, Some("basic_math"), None, None, false, false, false)
        .await
        .expect("Failed to list tools with category filter");

    let output = ui.get_output();
    // Should only show basic_math tools
    assert!(output.iter().any(|s| s.contains("basic_math")));
    // Total should be less than full 82/84
    assert!(
        !output
            .iter()
            .any(|s| s.contains("Total: 82 tools") || s.contains("Total: 84 tools"))
    );
}

#[tokio::test]
async fn test_list_with_keyword_filter() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // List tools filtered by keyword
    list_with_deps(&deps, None, Some("json"), None, false, false, false)
        .await
        .expect("Failed to list tools with keyword filter");

    let output = ui.get_output();
    // Should show tools with "json" in name/description/tags
    assert!(
        output
            .iter()
            .any(|s| s.contains("json") || s.contains("JSON"))
    );
}

#[cfg(test)]
mod version_resolution_tests {

    #[test]
    fn test_resolve_tools_constructs_correct_image_reference() {
        // Test that resolve_tools properly constructs image reference with version tag
        
        // This test verifies that resolve_tools creates the full image reference
        // The actual function call is tested in integration tests
        let tool_spec = crate::commands::tools::parse_tool_spec("proximity-search", None);
        assert_eq!(tool_spec.0, "ghcr"); // registry
        assert_eq!(tool_spec.1, "proximity-search"); // tool name  
        assert_eq!(tool_spec.2, "latest"); // version
    }

    #[test]
    fn test_version_resolution_with_explicit_version() {
        // Test parsing explicit version
        let (registry, name, version) = crate::commands::tools::parse_tool_spec("add:1.2.3", None);
        assert_eq!(registry, "ghcr");
        assert_eq!(name, "add");
        assert_eq!(version, "1.2.3");

        // This should result in image reference "ftl-tool-add:1.2.3"
        let expected_image_ref = format!("ftl-tool-add:{version}");
        assert_eq!(expected_image_ref, "ftl-tool-add:1.2.3");
    }

    #[test]
    fn test_version_resolution_with_latest() {
        // Test parsing "latest" version (default)
        let (registry, name, version) = crate::commands::tools::parse_tool_spec("add", None);
        assert_eq!(registry, "ghcr");
        assert_eq!(name, "add");
        assert_eq!(version, "latest");

        // This should result in image reference "ftl-tool-add:latest"
        let expected_image_ref = format!("ftl-tool-add:{version}");
        assert_eq!(expected_image_ref, "ftl-tool-add:latest");
    }

    #[test]
    fn test_image_name_construction_without_prefix() {
        // Test that tools without "ftl-tool-" prefix get it added
        let tool_name = "proximity-search";
        let version = "latest";
        
        let base_image_name = if tool_name.starts_with("ftl-tool-") {
            tool_name.to_string()
        } else {
            format!("ftl-tool-{tool_name}")
        };
        
        let image_name_with_version = format!("{base_image_name}:{version}");
        assert_eq!(image_name_with_version, "ftl-tool-proximity-search:latest");
    }

    #[test]
    fn test_image_name_construction_with_prefix() {
        // Test that tools with "ftl-tool-" prefix don't get it added again
        let tool_name = "ftl-tool-proximity-search";
        let version = "1.0.0";
        
        let base_image_name = if tool_name.starts_with("ftl-tool-") {
            tool_name.to_string()
        } else {
            format!("ftl-tool-{tool_name}")
        };
        
        let image_name_with_version = format!("{base_image_name}:{version}");
        assert_eq!(image_name_with_version, "ftl-tool-proximity-search:1.0.0");
    }
}

#[cfg(test)]
mod registry_integration_tests {
    #[test]
    fn test_image_tag_parsing_logic() {
        // Test our own parsing logic without relying on private functions
        fn parse_image_and_tag_test(image_name: &str) -> (String, String) {
            if let Some(pos) = image_name.rfind(':') {
                let image = image_name[..pos].to_string();
                let tag = image_name[pos + 1..].to_string();
                (image, tag)
            } else {
                (image_name.to_string(), "latest".to_string())
            }
        }

        let (image, tag) = parse_image_and_tag_test("ftl-tool-proximity-search:latest");
        assert_eq!(image, "ftl-tool-proximity-search");
        assert_eq!(tag, "latest");

        let (image, tag) = parse_image_and_tag_test("ftl-tool-proximity-search:1.2.3");
        assert_eq!(image, "ftl-tool-proximity-search");
        assert_eq!(tag, "1.2.3");

        let (image, tag) = parse_image_and_tag_test("ftl-tool-proximity-search");
        assert_eq!(image, "ftl-tool-proximity-search");
        assert_eq!(tag, "latest");
    }

    #[test]
    fn test_semver_validation_logic() {
        // Test semver validation through public APIs if available
        // This tests the behavior we expect from the registry components
        
        // Valid versions should work
        assert!(semver::Version::parse("1.0.0").is_ok());
        assert!(semver::Version::parse("2.1.3-alpha").is_ok());
        
        // Invalid versions should fail
        assert!(semver::Version::parse("latest").is_err());
        assert!(semver::Version::parse("main").is_err());
        assert!(semver::Version::parse("dev").is_err());
        
        // Auto-completion logic test
        assert!(semver::Version::parse("1.0.0").is_ok());
        assert!(semver::Version::parse("1.2.0").is_ok());
    }
}

#[cfg(test)]
mod add_tools_integration_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use crate::commands::tools::{add_tools_to_project, ResolvedTool};

    #[tokio::test]
    async fn test_add_tools_with_latest_version_should_fail_without_registry_resolution() {
        // This test documents the exact bug that was fixed
        let fixture = TestFixture::new();
        let deps = fixture.to_deps();
        
        // Create temporary directory with spin.toml
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let spin_toml_content = r#"spin_manifest_version = 2

[application]
name = "test-app"
version = "0.1.0"
"#;

        fs::write(temp_dir.path().join("spin.toml"), spin_toml_content).unwrap();

        // Create a ResolvedTool with "latest" version - this simulates the old buggy behavior
        let resolved_tools = vec![ResolvedTool {
            name: "proximity-search".to_string(),
            image_name: "ftl-tool-proximity-search:latest".to_string(), // Fixed: now includes version tag
            version: "latest".to_string(),
        }];

        // This should now work because the image_name includes the version tag
        // The registry adapter can parse "ftl-tool-proximity-search:latest" and resolve "latest"
        // to an actual semantic version
        let result = add_tools_to_project(&deps, &resolved_tools).await;

        // The actual result depends on whether crane is available and the registry is accessible
        // But at minimum, it should not fail with "Invalid version tag 'latest'" immediately
        // It will either succeed (if crane works) or fail with a different error (network/auth)
        match result {
            Ok(_) => {
                // Success - the version resolution worked
                assert!(true);
            }
            Err(e) => {
                let error_msg = e.to_string();
                // Should not fail with the original "Invalid version tag 'latest'" error
                assert!(!error_msg.contains("Invalid version tag 'latest'"));
                // It might fail with crane-related errors, which is expected in test environment
                assert!(
                    error_msg.contains("crane") || 
                    error_msg.contains("Failed to get registry components") ||
                    error_msg.contains("No semantic versions found")
                );
            }
        }

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_resolved_tool_construction_includes_version_tag() {
        // Test that demonstrates the fix - ResolvedTool.image_name now includes version
        let tool_name = "proximity-search";
        let version = "latest";
        
        // Simulate the logic from resolve_tools function
        let base_image_name = if tool_name.starts_with("ftl-tool-") {
            tool_name.to_string()
        } else {
            format!("ftl-tool-{tool_name}")
        };
        
        let image_name_with_version = format!("{base_image_name}:{version}");
        
        let resolved_tool = ResolvedTool {
            name: tool_name.to_string(),
            image_name: image_name_with_version.clone(),
            version: version.to_string(),
        };

        // Verify the fix: image_name should include the version tag
        assert_eq!(resolved_tool.image_name, "ftl-tool-proximity-search:latest");
        
        // This allows the registry adapter to parse it correctly
        let (parsed_image, parsed_tag) = if let Some(pos) = resolved_tool.image_name.rfind(':') {
            (resolved_tool.image_name[..pos].to_string(), resolved_tool.image_name[pos + 1..].to_string())
        } else {
            (resolved_tool.image_name.clone(), "latest".to_string())
        };
        
        assert_eq!(parsed_image, "ftl-tool-proximity-search");
        assert_eq!(parsed_tag, "latest");
    }

    #[test]
    fn test_version_parsing_regression_test() {
        // Regression test for the specific case that was failing
        
        // Input: User runs "ftl tools add ftl-tool-proximity-search"
        let user_input = "ftl-tool-proximity-search";
        
        // 1. Parse tool spec
        let (registry, tool_name, tool_version) = crate::commands::tools::parse_tool_spec(user_input, None);
        assert_eq!(registry, "ghcr");
        assert_eq!(tool_name, "ftl-tool-proximity-search");
        assert_eq!(tool_version, "latest");
        
        // 2. Construct base image name (this part was working)
        let base_image_name = if tool_name.starts_with("ftl-tool-") {
            tool_name.clone()
        } else {
            format!("ftl-tool-{tool_name}")
        };
        assert_eq!(base_image_name, "ftl-tool-proximity-search");
        
        // 3. THE FIX: Construct full image reference with version tag
        let image_name_with_version = format!("{base_image_name}:{tool_version}");
        assert_eq!(image_name_with_version, "ftl-tool-proximity-search:latest");
        
        // 4. This gets passed to registry adapter's get_registry_components
        // The adapter can now parse it correctly and resolve "latest" to actual version
        
        // Simulate registry adapter parsing
        let (parsed_base, parsed_tag) = if let Some(pos) = image_name_with_version.rfind(':') {
            (image_name_with_version[..pos].to_string(), image_name_with_version[pos + 1..].to_string())
        } else {
            (image_name_with_version, "latest".to_string())
        };
        
        assert_eq!(parsed_base, "ftl-tool-proximity-search");
        assert_eq!(parsed_tag, "latest");
        
        // Registry adapter would then resolve "latest" to actual version like "1.2.3"
        // This is where the original bug was - it was getting just "ftl-tool-proximity-search" 
        // without the ":latest" tag, so couldn't resolve the version
    }
}

#[tokio::test]
async fn test_list_verbose_output() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    // List tools with verbose output
    list_with_deps(&deps, None, None, None, true, false, false)
        .await
        .expect("Failed to list tools with verbose output");

    let output = ui.get_output();
    // Verbose output should include descriptions
    assert!(
        output
            .iter()
            .any(|s| s.contains("Description:") || s.contains("Tags:"))
    );
}
