//! Unit tests for the registry command

use std::path::Path;
use std::sync::Arc;

use crate::commands::registry::{
    RegistryDependencies, list_with_deps, remove_default_registry, set_default_registry,
};
use crate::test_helpers::*;
use ftl_common::ui::TestUserInterface;
use ftl_runtime::deps::{FileSystem, UserInterface};
use mockall::predicate::*;

struct TestFixture {
    ui: Arc<TestUserInterface>,
    file_system: MockFileSystemMock,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            file_system: MockFileSystemMock::new(),
        }
    }

    fn mock_ftl_toml_exists(&mut self, exists: bool) {
        self.file_system
            .expect_exists()
            .with(eq(Path::new("ftl.toml")))
            .times(1)
            .returning(move |_| exists);
    }

    fn mock_read_ftl_toml(&mut self, content: &'static str) {
        self.file_system
            .expect_read_to_string()
            .with(eq(Path::new("ftl.toml")))
            .times(1)
            .returning(move |_| Ok(content.to_string()));
    }

    fn mock_write_ftl_toml(&mut self) {
        self.file_system
            .expect_write_string()
            .with(eq(Path::new("ftl.toml")), always())
            .times(1)
            .returning(|_, _| Ok(()));
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<RegistryDependencies> {
        Arc::new(RegistryDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            file_system: Arc::new(self.file_system) as Arc<dyn FileSystem>,
        })
    }
}

#[tokio::test]
async fn test_list_no_ftl_toml() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(false);

    let deps = fixture.to_deps();
    let result = list_with_deps(&deps);

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("No ftl.toml found")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Available registry types:"))
    );
    assert!(output.iter().any(|s| s.contains("ghcr.io")));
    assert!(output.iter().any(|s| s.contains("docker.io")));
    assert!(output.iter().any(|s| s.contains("docker login")));
}

#[tokio::test]
async fn test_list_with_default_registry() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(true);
    fixture.mock_read_ftl_toml(
        r#"
[project]
name = "my-project"
default_registry = "ghcr.io/myorg"
"#,
    );

    let deps = fixture.to_deps();
    let result = list_with_deps(&deps);

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Current registry configuration:"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Default registry: ghcr.io/myorg"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Available registry types:"))
    );
}

#[tokio::test]
async fn test_list_without_default_registry() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(true);
    fixture.mock_read_ftl_toml(
        r#"
[project]
name = "my-project"
"#,
    );

    let deps = fixture.to_deps();
    let result = list_with_deps(&deps);

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("No default registry configured"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Available registry types:"))
    );
}

#[tokio::test]
async fn test_set_default_registry_no_ftl_toml() {
    let mut fixture = TestFixture::new();

    fixture.mock_ftl_toml_exists(false);

    let deps = fixture.to_deps();
    let result = set_default_registry(&deps, "ghcr.io/myorg");

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No ftl.toml found")
    );
}

#[tokio::test]
async fn test_set_default_registry_success() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(true);
    fixture.mock_read_ftl_toml(
        r#"
[project]
name = "my-project"
"#,
    );
    fixture.mock_write_ftl_toml();

    let deps = fixture.to_deps();
    let result = set_default_registry(&deps, "ghcr.io/myorg");

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("✓ Default registry set to: ghcr.io/myorg"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("You can now use short component names"))
    );
}

#[tokio::test]
async fn test_set_default_registry_update_existing() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(true);
    fixture.mock_read_ftl_toml(
        r#"
[project]
name = "my-project"
default_registry = "docker.io/oldorg"
"#,
    );
    fixture.mock_write_ftl_toml();

    let deps = fixture.to_deps();
    let result = set_default_registry(&deps, "ghcr.io/neworg");

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("✓ Default registry set to: ghcr.io/neworg"))
    );
}

#[tokio::test]
async fn test_set_default_registry_no_project_section() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(true);
    fixture.mock_read_ftl_toml(
        r#"
[component]
name = "my-component"
"#,
    );
    fixture.mock_write_ftl_toml();

    let deps = fixture.to_deps();
    let result = set_default_registry(&deps, "ghcr.io/myorg");

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("✓ Default registry set to: ghcr.io/myorg"))
    );
}

#[tokio::test]
async fn test_remove_default_registry_no_ftl_toml() {
    let mut fixture = TestFixture::new();

    fixture.mock_ftl_toml_exists(false);

    let deps = fixture.to_deps();
    let result = remove_default_registry(&deps);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No ftl.toml found")
    );
}

#[tokio::test]
async fn test_remove_default_registry_success() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(true);
    fixture.mock_read_ftl_toml(
        r#"
[project]
name = "my-project"
default_registry = "ghcr.io/myorg"
"#,
    );
    fixture.mock_write_ftl_toml();

    let deps = fixture.to_deps();
    let result = remove_default_registry(&deps);

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("✓ Default registry removed"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Components will now require full registry URLs"))
    );
}

#[tokio::test]
async fn test_remove_default_registry_no_registry_set() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(true);
    fixture.mock_read_ftl_toml(
        r#"
[project]
name = "my-project"
"#,
    );
    fixture.mock_write_ftl_toml();

    let deps = fixture.to_deps();
    let result = remove_default_registry(&deps);

    assert!(result.is_ok());

    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("✓ Default registry removed"))
    );
}

#[tokio::test]
async fn test_list_output_completeness() {
    let mut fixture = TestFixture::new();
    let ui = fixture.ui.clone();

    fixture.mock_ftl_toml_exists(true);
    fixture.mock_read_ftl_toml(
        r#"
[project]
name = "test-project"
default_registry = "docker.io/mycompany"
"#,
    );

    let deps = fixture.to_deps();
    let result = list_with_deps(&deps);

    assert!(result.is_ok());

    let output = ui.get_output();

    // Verify all important information is displayed
    assert!(
        output
            .iter()
            .any(|s| s.contains("Current registry configuration"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Default registry: docker.io/mycompany"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Available registry types"))
    );
    assert!(output.iter().any(|s| s.contains("ghcr.io")));
    assert!(output.iter().any(|s| s.contains("docker.io")));
    assert!(output.iter().any(|s| s.contains("Custom URL")));
    assert!(output.iter().any(|s| s.contains("Authentication")));
    assert!(output.iter().any(|s| s.contains("docker login")));
}

#[tokio::test]
async fn test_set_various_registry_formats() {
    let test_cases = vec![
        "ghcr.io/myorg",
        "docker.io/mycompany",
        "123456789.dkr.ecr.us-west-2.amazonaws.com",
        "custom.registry.com/namespace",
    ];

    for registry_url in test_cases {
        let mut fixture = TestFixture::new();
        let ui = fixture.ui.clone();

        fixture.mock_ftl_toml_exists(true);
        fixture.mock_read_ftl_toml(
            r#"
[project]
name = "my-project"
"#,
        );
        fixture.mock_write_ftl_toml();

        let deps = fixture.to_deps();
        let result = set_default_registry(&deps, registry_url);

        assert!(result.is_ok(), "Failed to set registry: {registry_url}");

        let output = ui.get_output();
        assert!(
            output
                .iter()
                .any(|s| s.contains(&format!("✓ Default registry set to: {registry_url}")))
        );
    }
}
