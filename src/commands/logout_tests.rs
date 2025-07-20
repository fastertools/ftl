//! Unit tests for the logout command

use std::sync::Arc;

use crate::commands::logout::*;
use crate::deps::*;
use crate::ui::TestUserInterface;

// Mock implementation of CredentialsClearer
struct MockCredentialsClearer {
    should_fail: bool,
    error_message: Option<String>,
}

impl MockCredentialsClearer {
    fn new() -> Self {
        Self {
            should_fail: false,
            error_message: None,
        }
    }

    fn with_failure(mut self, message: &str) -> Self {
        self.should_fail = true;
        self.error_message = Some(message.to_string());
        self
    }
}

impl CredentialsClearer for MockCredentialsClearer {
    fn clear_stored_credentials(&self) -> Result<(), anyhow::Error> {
        if self.should_fail {
            Err(anyhow::anyhow!(self.error_message.clone().unwrap_or_else(
                || "Failed to clear credentials".to_string()
            )))
        } else {
            Ok(())
        }
    }
}

struct TestFixture {
    ui: Arc<TestUserInterface>,
    credentials_clearer: Arc<MockCredentialsClearer>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            credentials_clearer: Arc::new(MockCredentialsClearer::new()),
        }
    }

    fn to_deps(self) -> Arc<LogoutDependencies> {
        Arc::new(LogoutDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            credentials_clearer: self.credentials_clearer as Arc<dyn CredentialsClearer>,
        })
    }
}

#[tokio::test]
async fn test_logout_success() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(deps).await;
    assert!(result.is_ok());

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Logging out of FTL")));
    assert!(output.iter().any(|s| s.contains("Successfully logged out")));
}

#[tokio::test]
async fn test_logout_not_logged_in() {
    let mut fixture = TestFixture::new();
    fixture.credentials_clearer =
        Arc::new(MockCredentialsClearer::new().with_failure("No matching entry found in keyring"));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(deps).await;
    assert!(result.is_ok()); // Should not fail even if not logged in

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Not currently logged in")));
}

#[tokio::test]
async fn test_logout_keyring_error() {
    let mut fixture = TestFixture::new();
    fixture.credentials_clearer =
        Arc::new(MockCredentialsClearer::new().with_failure("Failed to access keyring"));

    let deps = fixture.to_deps();

    let result = execute_with_deps(deps).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to access keyring")
    );
}

#[tokio::test]
async fn test_logout_with_different_keyring_errors() {
    // Test various keyring error messages that indicate "not logged in"
    let not_found_errors = vec![
        "No matching entry found",
        "No matching entry found in keyring",
        "Entry not found: No matching entry found",
    ];

    for error_msg in not_found_errors {
        let mut fixture = TestFixture::new();
        fixture.credentials_clearer =
            Arc::new(MockCredentialsClearer::new().with_failure(error_msg));

        let ui = fixture.ui.clone();
        let deps = fixture.to_deps();

        let result = execute_with_deps(deps).await;
        assert!(
            result.is_ok(),
            "Should handle '{}' as not logged in",
            error_msg
        );

        let output = ui.get_output();
        assert!(output.iter().any(|s| s.contains("Not currently logged in")));
    }
}

#[tokio::test]
async fn test_logout_unexpected_error() {
    let mut fixture = TestFixture::new();
    fixture.credentials_clearer =
        Arc::new(MockCredentialsClearer::new().with_failure("Unexpected database error"));

    let deps = fixture.to_deps();

    let result = execute_with_deps(deps).await;
    assert!(result.is_err());
    assert!(
        !result
            .unwrap_err()
            .to_string()
            .contains("No matching entry found")
    );
}
