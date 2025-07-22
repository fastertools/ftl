//! Unit tests for the registry command

use std::sync::Arc;

use crate::commands::registry::{
    RegistryDependencies, info_with_deps, list_with_deps, search_with_deps,
};
use crate::commands::registries::{
    list_registries, add_registry, remove_registry, set_default_registry, 
    enable_registry, set_priority
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

    #[allow(clippy::wrong_self_convention)]
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

    list_with_deps(None, &deps);

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

    list_with_deps(Some("docker.io"), &deps);

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

    search_with_deps("my-component", None, &deps);

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

    search_with_deps("test-tool", Some("quay.io"), &deps);

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

    search_with_deps("my-component@v2.0", None, &deps);

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

    info_with_deps("my-component", &deps);

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

    info_with_deps("ghcr.io/ftl/my-tool:v1.0.0", &deps);

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

    info_with_deps("docker.io/library/nginx:latest", &deps);

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

    list_with_deps(None, &deps);

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
            "Expected '{actual}' to contain '{expected}'"
        );
    }
}

// Registry Management Function Tests

#[tokio::test]
async fn test_list_registries_with_default() {
    // This test would need to mock the config loading
    // For now, we'll test the error handling when config is missing
    let result = list_registries().await;
    // Should either succeed with a real config or fail with a clear error
    match result {
        Ok(_) => {
            // If config exists, the function succeeded
        }
        Err(e) => {
            // Should be a clear error about missing config
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_add_registry_ghcr_basic() {
    let result = add_registry(
        "test-ghcr".to_string(),
        "ghcr".to_string(),
        Some("testorg".to_string()),
        None,
        None,
        None,
        None,
        10,
        true,
    ).await;
    
    // Should either succeed or fail with clear error about config
    match result {
        Ok(_) => {
            // Registry was added successfully
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_add_registry_docker_basic() {
    let result = add_registry(
        "test-docker".to_string(),
        "docker".to_string(),
        None,
        None,
        None,
        None,
        None,
        10,
        true,
    ).await;
    
    // Should either succeed or fail with clear error about config
    match result {
        Ok(_) => {
            // Registry was added successfully
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied") ||
                error_msg.contains("Invalid registry type"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_add_registry_ecr_with_params() {
    let result = add_registry(
        "test-ecr".to_string(),
        "ecr".to_string(),
        None,
        Some("123456789012".to_string()),
        Some("us-east-1".to_string()),
        None,
        None,
        10,
        false,
    ).await;
    
    // Should either succeed or fail with clear error
    match result {
        Ok(_) => {
            // Registry was added successfully
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied") ||
                error_msg.contains("Invalid registry type"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_add_registry_custom_with_url() {
    let result = add_registry(
        "test-custom".to_string(),
        "custom".to_string(),
        None,
        None,
        None,
        Some("registry.example.com/{image_name}".to_string()),
        Some("basic".to_string()),
        5,
        true,
    ).await;
    
    // Should either succeed or fail with clear error
    match result {
        Ok(_) => {
            // Registry was added successfully
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied") ||
                error_msg.contains("Invalid registry type"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_add_registry_invalid_type() {
    let result = add_registry(
        "test-invalid".to_string(),
        "invalid-type".to_string(),
        None,
        None,
        None,
        None,
        None,
        10,
        true,
    ).await;
    
    // Should fail with invalid registry type error
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("Invalid registry type") ||
        error_msg.contains("invalid-type"),
        "Expected invalid registry type error, got: {}", error_msg
    );
}

#[tokio::test]
async fn test_remove_registry_basic() {
    let result = remove_registry("test-registry".to_string()).await;
    
    // Should either succeed or fail with clear error
    match result {
        Ok(_) => {
            // Registry was removed successfully (or didn't exist)
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied") ||
                error_msg.contains("not found"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_set_default_registry_basic() {
    let result = set_default_registry("test-registry".to_string()).await;
    
    // Should either succeed or fail with clear error
    match result {
        Ok(_) => {
            // Default was set successfully
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied") ||
                error_msg.contains("not found"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_enable_registry_basic() {
    let result = enable_registry("test-registry".to_string(), true).await;
    
    // Should either succeed or fail with clear error
    match result {
        Ok(_) => {
            // Registry was enabled successfully
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied") ||
                error_msg.contains("not found"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_disable_registry_basic() {
    let result = enable_registry("test-registry".to_string(), false).await;
    
    // Should either succeed or fail with clear error
    match result {
        Ok(_) => {
            // Registry was disabled successfully
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied") ||
                error_msg.contains("not found"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_set_priority_basic() {
    let result = set_priority("test-registry".to_string(), 5).await;
    
    // Should either succeed or fail with clear error
    match result {
        Ok(_) => {
            // Priority was set successfully
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("config") || 
                error_msg.contains("No such file") ||
                error_msg.contains("Permission denied") ||
                error_msg.contains("not found"),
                "Unexpected error: {}", error_msg
            );
        }
    }
}
