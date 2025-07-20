//! Unit tests for the registry command

use std::sync::Arc;

use crate::commands::registry::{
    RegistryDependencies, info_with_deps, list_with_deps, search_with_deps,
};
use crate::deps::UserInterface;
use crate::ui::TestUserInterface;

struct TestFixture {
    ui: Arc<TestUserInterface>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
        }
    }

    fn to_deps(self) -> Arc<RegistryDependencies> {
        Arc::new(RegistryDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
        })
    }
}

#[tokio::test]
async fn test_list_default_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = list_with_deps(None, deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Listing components from ghcr.io"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Registry listing not yet implemented"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("GitHub Container Registry"))
    );
    assert!(output.iter().any(|s| s.contains("Docker Hub")));
}

#[tokio::test]
async fn test_list_custom_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = list_with_deps(Some("docker.io".to_string()), deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Listing components from docker.io"))
    );
}

#[tokio::test]
async fn test_search_default_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = search_with_deps("my-component".to_string(), None, deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Searching for 'my-component' in ghcr.io"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Registry search not yet implemented"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("https://github.com/search?q=mcp+my-component"))
    );
}

#[tokio::test]
async fn test_search_custom_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = search_with_deps("test-tool".to_string(), Some("quay.io".to_string()), deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Searching for 'test-tool' in quay.io"))
    );
}

#[tokio::test]
async fn test_search_with_special_characters() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = search_with_deps("my-component@v2.0".to_string(), None, deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Searching for 'my-component@v2.0'"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("https://github.com/search?q=mcp+my-component@v2.0"))
    );
}

#[tokio::test]
async fn test_info_simple_component() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = info_with_deps("my-component".to_string(), deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Getting info for component: my-component"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Registry info not yet implemented"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Component reference formats:"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("ghcr.io/username/component:version"))
    );
}

#[tokio::test]
async fn test_info_full_component_reference() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = info_with_deps("ghcr.io/ftl/my-tool:v1.0.0".to_string(), deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Getting info for component: ghcr.io/ftl/my-tool:v1.0.0"))
    );
}

#[tokio::test]
async fn test_info_docker_hub_reference() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = info_with_deps("docker.io/library/nginx:latest".to_string(), deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Getting info for component: docker.io/library/nginx:latest"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("docker.io/username/component:version"))
    );
}

#[tokio::test]
async fn test_list_output_completeness() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = list_with_deps(None, deps).await;
    assert!(result.is_ok());

    // Verify all expected output lines are present
    let output = ui.get_output();
    let expected_lines = [
        "→ Listing components from ghcr.io",
        "",
        "! Registry listing not yet implemented",
        "",
        "For now, you can browse components at:",
        "  - GitHub Container Registry: https://github.com/orgs/YOUR_ORG/packages",
        "  - Docker Hub: https://hub.docker.com/",
    ];

    // Verify exact line count
    assert_eq!(output.len(), expected_lines.len());

    // Verify each line
    for (actual, expected) in output.iter().zip(expected_lines.iter()) {
        assert!(
            actual.contains(expected),
            "Expected '{}' to contain '{}'",
            actual,
            expected
        );
    }
}
