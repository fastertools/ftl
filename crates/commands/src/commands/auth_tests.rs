//! Unit tests for the auth command

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use crate::commands::auth::{AuthDependencies, Clock, CredentialsProvider, status_with_deps};
use ftl_common::ui::TestUserInterface;
use ftl_runtime::deps::{StoredCredentials, UserInterface};

// Mock implementation of CredentialsProvider
struct MockCredentialsProvider {
    should_fail: bool,
    error_message: Option<String>,
    credentials: Option<StoredCredentials>,
}

impl MockCredentialsProvider {
    fn new() -> Self {
        Self {
            should_fail: false,
            error_message: None,
            credentials: Some(StoredCredentials {
                access_token: "test_token".to_string(),
                refresh_token: Some("refresh_token".to_string()),
                id_token: Some("id_token".to_string()),
                expires_at: Some(Utc::now() + Duration::hours(2)),
                authkit_domain: "auth.example.com".to_string(),
            }),
        }
    }

    fn with_failure(mut self, message: &str) -> Self {
        self.should_fail = true;
        self.error_message = Some(message.to_string());
        self
    }

    fn with_credentials(mut self, credentials: StoredCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }
}

impl CredentialsProvider for MockCredentialsProvider {
    fn get_stored_credentials(&self) -> Result<StoredCredentials, anyhow::Error> {
        if self.should_fail {
            Err(anyhow::anyhow!(
                self.error_message
                    .clone()
                    .unwrap_or_else(|| "Failed to get credentials".to_string())
            ))
        } else {
            Ok(self.credentials.clone().unwrap())
        }
    }
}

// Mock implementation of Clock
struct MockClock {
    now: DateTime<Utc>,
}

impl MockClock {
    fn new() -> Self {
        Self { now: Utc::now() }
    }

    #[allow(dead_code)]
    fn with_time(now: DateTime<Utc>) -> Self {
        Self { now }
    }
}

impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
    }
}

struct TestFixture {
    ui: Arc<TestUserInterface>,
    credentials_provider: Arc<MockCredentialsProvider>,
    clock: Arc<MockClock>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            credentials_provider: Arc::new(MockCredentialsProvider::new()),
            clock: Arc::new(MockClock::new()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<AuthDependencies> {
        Arc::new(AuthDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            credentials_provider: self.credentials_provider as Arc<dyn CredentialsProvider>,
            clock: self.clock as Arc<dyn Clock>,
        })
    }
}

#[tokio::test]
async fn test_auth_status_logged_in_with_valid_token() {
    let fixture = TestFixture::new();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    status_with_deps(&deps);

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Authentication Status")));
    assert!(output.iter().any(|s| s.contains("✅ Logged in")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("AuthKit Domain: auth.example.com"))
    );
    assert!(output.iter().any(|s| s.contains("Access Token: Valid for")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Refresh Token: Available"))
    );
}

#[tokio::test]
async fn test_auth_status_logged_in_with_expired_token() {
    let mut fixture = TestFixture::new();

    let past_time = Utc::now() - Duration::hours(1);
    fixture.credentials_provider = Arc::new(MockCredentialsProvider::new().with_credentials(
        StoredCredentials {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            id_token: None,
            expires_at: Some(past_time),
            authkit_domain: "auth.example.com".to_string(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    status_with_deps(&deps);

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("✅ Logged in")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Access Token: ⚠️  Expired"))
    );
}

#[tokio::test]
async fn test_auth_status_logged_in_without_expiry() {
    let mut fixture = TestFixture::new();

    fixture.credentials_provider = Arc::new(MockCredentialsProvider::new().with_credentials(
        StoredCredentials {
            access_token: "test_token".to_string(),
            refresh_token: None,
            id_token: None,
            expires_at: None,
            authkit_domain: "auth.example.com".to_string(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    status_with_deps(&deps);

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("✅ Logged in")));
    assert!(output.iter().any(|s| s.contains("Access Token: Valid")));
    assert!(!output.iter().any(|s| s.contains("Refresh Token:")));
}

#[tokio::test]
async fn test_auth_status_not_logged_in() {
    let mut fixture = TestFixture::new();
    fixture.credentials_provider =
        Arc::new(MockCredentialsProvider::new().with_failure("No matching entry found in keyring"));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    status_with_deps(&deps);

    // Verify output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("🔐 Not logged in")));
    assert!(
        output
            .iter()
            .any(|s| s.contains("Run ftl login to authenticate"))
    );
}

#[tokio::test]
async fn test_auth_status_error_checking_credentials() {
    let mut fixture = TestFixture::new();
    fixture.credentials_provider =
        Arc::new(MockCredentialsProvider::new().with_failure("Database connection failed"));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    status_with_deps(&deps);

    // Verify output
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("⚠️  Error checking authentication status"))
    );
    assert!(
        output
            .iter()
            .any(|s| s.contains("Run ftl login to re-authenticate"))
    );
}

#[tokio::test]
async fn test_auth_status_token_expiring_soon() {
    let mut fixture = TestFixture::new();

    let expires_at = Utc::now() + Duration::minutes(45);
    fixture.credentials_provider = Arc::new(MockCredentialsProvider::new().with_credentials(
        StoredCredentials {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            id_token: None,
            expires_at: Some(expires_at),
            authkit_domain: "auth.example.com".to_string(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    status_with_deps(&deps);

    // Verify output shows 0h 45m
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Access Token: Valid for 0h 45m"))
    );
}

#[tokio::test]
async fn test_auth_status_token_expiring_days_away() {
    let mut fixture = TestFixture::new();

    let expires_at = Utc::now() + Duration::days(7) + Duration::hours(3) + Duration::minutes(30);
    fixture.credentials_provider = Arc::new(MockCredentialsProvider::new().with_credentials(
        StoredCredentials {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            id_token: None,
            expires_at: Some(expires_at),
            authkit_domain: "auth.example.com".to_string(),
        },
    ));

    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    status_with_deps(&deps);

    // Verify output shows total hours (7*24 + 3 = 171h)
    let output = ui.get_output();
    assert!(
        output
            .iter()
            .any(|s| s.contains("Access Token: Valid for 171h 30m"))
    );
}

#[tokio::test]
async fn test_auth_status_various_error_messages() {
    // Test various error messages that should trigger "Not logged in"
    let not_found_errors = vec![
        "No matching entry found",
        "No matching entry found in keyring",
        "Entry not found: No matching entry found",
    ];

    for error_msg in not_found_errors {
        let mut fixture = TestFixture::new();
        fixture.credentials_provider =
            Arc::new(MockCredentialsProvider::new().with_failure(error_msg));

        let ui = fixture.ui.clone();
        let deps = fixture.to_deps();

        status_with_deps(&deps);

        let output = ui.get_output();
        assert!(output.iter().any(|s| s.contains("🔐 Not logged in")));
    }
}
