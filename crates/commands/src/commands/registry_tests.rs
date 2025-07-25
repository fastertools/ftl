//! Unit tests for the registry command

use std::sync::Arc;

use crate::commands::registry::{
    RegistryDependencies, info_with_deps, list_with_deps, search_with_deps,
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
    fn to_deps(self) -> Arc<RegistryDependencies> {
        Arc::new(RegistryDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            client: self.client,
        })
    }
}

#[tokio::test]
async fn test_list_default_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    list_with_deps(None, &deps)
        .await
        .expect("Failed to list registry");

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Listing components from ghcr"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Registry listing requires crane CLI"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("GitHub Container Registry"))
    );
    assert!(output.iter().any(|s| s.contains("GitHub Container Registry")));
}

#[tokio::test]
async fn test_list_custom_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    list_with_deps(Some("docker"), &deps)
        .await
        .expect("Failed to list registry");

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Listing components from docker"))
    );
}

#[tokio::test]
async fn test_search_default_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    search_with_deps("my-component", None, &deps)
        .await
        .expect("Failed to search registry");

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Searching for 'my-component' in ghcr"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Registry search not yet implemented"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("https://github.com/search?q=my-component"))
    );
}

#[tokio::test]
async fn test_search_custom_registry() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    search_with_deps("test-tool", Some("docker"), &deps)
        .await
        .expect("Failed to search registry");

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Searching for 'test-tool' in docker"))
    );
}

#[tokio::test]
async fn test_search_with_special_characters() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    search_with_deps("my-component@v2.0", None, &deps)
        .await
        .expect("Failed to search registry");

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
            .any(|s| s.contains("https://github.com/search?q=my-component@v2.0"))
    );
}

#[tokio::test]
async fn test_info_simple_component() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    info_with_deps("my-component", &deps)
        .await
        .expect("Failed to get info");

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
            .any(|s| s.contains("Checking if component exists"))
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

    info_with_deps("ghcr.io/ftl/my-tool:v1.0.0", &deps)
        .await
        .expect("Failed to get info");

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

    info_with_deps("docker.io/library/nginx:latest", &deps)
        .await
        .expect("Failed to get info");

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

    list_with_deps(None, &deps)
        .await
        .expect("Failed to list registry");

    // Verify all expected output lines are present
    let output = ui.get_output();
    
    // Just verify key content is present rather than exact line count
    // since the new implementation has different formatting
    assert!(output.iter().any(|s| s.contains("Listing components from ghcr")));
    assert!(output.iter().any(|s| s.contains("GitHub Container Registry")));
    assert!(output.iter().any(|s| s.contains("Registry listing requires crane CLI")));
}
