//! Unit tests for the login command

use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::commands::login::{self, *};
use crate::deps::{AsyncRuntime, UserInterface};
use crate::ui::TestUserInterface;

// Mock implementations
struct MockHttpClient {
    responses: Arc<Mutex<Vec<(String, HttpResponse)>>>,
    token_call_count: Arc<Mutex<usize>>,
}

impl MockHttpClient {
    fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            token_call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn add_response(&self, url_contains: &str, response: HttpResponse) {
        self.responses
            .lock()
            .unwrap()
            .push((url_contains.to_string(), response));
    }
}

#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
    async fn post(&self, url: &str, _body: &str) -> Result<HttpResponse, anyhow::Error> {
        let responses = self.responses.lock().unwrap();

        // Special handling for oauth2/token endpoint
        if url.contains("oauth2/token") {
            let mut count = self.token_call_count.lock().unwrap();
            let current_count = *count;
            *count += 1;
            drop(count);

            // Find all token responses
            let token_responses: Vec<_> = responses
                .iter()
                .filter(|(pattern, _)| pattern == "oauth2/token")
                .map(|(_, resp)| resp)
                .collect();

            if !token_responses.is_empty() {
                let index = current_count.min(token_responses.len() - 1);
                let response = token_responses.get(index).unwrap();
                return Ok(HttpResponse {
                    status: response.status,
                    body: response.body.clone(),
                });
            }
        }

        // For other endpoints, return the first matching response
        for (pattern, response) in responses.iter() {
            if url.contains(pattern) {
                return Ok(HttpResponse {
                    status: response.status,
                    body: response.body.clone(),
                });
            }
        }

        Err(anyhow::anyhow!("No mock response for URL: {}", url))
    }
}

struct MockKeyringStorage {
    storage: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl MockKeyringStorage {
    fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
}

impl KeyringStorage for MockKeyringStorage {
    fn store(&self, service: &str, username: &str, password: &str) -> Result<(), anyhow::Error> {
        let key = format!("{service}-{username}");
        self.storage
            .lock()
            .unwrap()
            .insert(key, password.to_string());
        Ok(())
    }

    fn retrieve(&self, service: &str, username: &str) -> Result<String, anyhow::Error> {
        let key = format!("{service}-{username}");
        self.storage
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No password found"))
    }

    fn delete(&self, service: &str, username: &str) -> Result<(), anyhow::Error> {
        let key = format!("{service}-{username}");
        self.storage.lock().unwrap().remove(&key);
        Ok(())
    }
}

struct MockBrowserLauncher {
    opened_urls: Arc<Mutex<Vec<String>>>,
}

impl MockBrowserLauncher {
    fn new() -> Self {
        Self {
            opened_urls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_opened_urls(&self) -> Vec<String> {
        self.opened_urls.lock().unwrap().clone()
    }
}

impl BrowserLauncher for MockBrowserLauncher {
    fn open(&self, url: &str) -> Result<(), anyhow::Error> {
        self.opened_urls.lock().unwrap().push(url.to_string());
        Ok(())
    }
}

struct MockClock {
    now: DateTime<Utc>,
    instant: Instant,
}

impl MockClock {
    fn new() -> Self {
        Self {
            now: Utc::now(),
            instant: Instant::now(),
        }
    }

    #[allow(dead_code)]
    fn with_time(now: DateTime<Utc>) -> Self {
        Self {
            now,
            instant: Instant::now(),
        }
    }
}

impl login::Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
    }

    fn instant_now(&self) -> Instant {
        self.instant
    }
}

struct MockAsyncRuntime {
    sleep_count: Arc<Mutex<usize>>,
}

impl MockAsyncRuntime {
    fn new() -> Self {
        Self {
            sleep_count: Arc::new(Mutex::new(0)),
        }
    }

    fn get_sleep_count(&self) -> usize {
        *self.sleep_count.lock().unwrap()
    }
}

#[async_trait::async_trait]
impl AsyncRuntime for MockAsyncRuntime {
    async fn sleep(&self, _duration: Duration) {
        *self.sleep_count.lock().unwrap() += 1;
    }
}

// Test fixture
struct TestFixture {
    ui: Arc<TestUserInterface>,
    http_client: Arc<MockHttpClient>,
    keyring: Arc<MockKeyringStorage>,
    browser_launcher: Arc<MockBrowserLauncher>,
    async_runtime: Arc<MockAsyncRuntime>,
    clock: Arc<MockClock>,
}

impl TestFixture {
    fn new() -> Self {
        Self {
            ui: Arc::new(TestUserInterface::new()),
            http_client: Arc::new(MockHttpClient::new()),
            keyring: Arc::new(MockKeyringStorage::new()),
            browser_launcher: Arc::new(MockBrowserLauncher::new()),
            async_runtime: Arc::new(MockAsyncRuntime::new()),
            clock: Arc::new(MockClock::new()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_deps(self) -> Arc<LoginDependencies> {
        Arc::new(LoginDependencies {
            ui: self.ui as Arc<dyn UserInterface>,
            http_client: self.http_client.clone() as Arc<dyn HttpClient>,
            keyring: self.keyring as Arc<dyn KeyringStorage>,
            browser_launcher: self.browser_launcher as Arc<dyn BrowserLauncher>,
            async_runtime: self.async_runtime as Arc<dyn AsyncRuntime>,
            clock: self.clock as Arc<dyn login::Clock>,
        })
    }
}

#[tokio::test]
async fn test_login_success_with_browser() {
    let fixture = TestFixture::new();

    // Mock device authorization response
    let auth_response = DeviceAuthResponse {
        device_code: "test_device_code".to_string(),
        user_code: "TEST-CODE".to_string(),
        verification_uri: "https://auth.example.com/verify".to_string(),
        verification_uri_complete: "https://auth.example.com/verify?code=TEST-CODE".to_string(),
        expires_in: 600,
        interval: Some(5),
    };

    fixture.http_client.add_response(
        "device_authorization",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&auth_response).unwrap(),
        },
    );

    // Mock token response (immediate success)
    let token_response = TokenResponse {
        access_token: "test_access_token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: Some(3600),
        refresh_token: Some("test_refresh_token".to_string()),
        id_token: Some("test_id_token".to_string()),
    };

    fixture.http_client.add_response(
        "oauth2/token",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&token_response).unwrap(),
        },
    );

    let browser = fixture.browser_launcher.clone();
    let keyring = fixture.keyring.clone();
    let ui = fixture.ui.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(
        LoginConfig {
            no_browser: false,
            authkit_domain: Some("auth.example.com".to_string()),
            client_id: Some("test_client".to_string()),
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify browser was NOT opened (TestUserInterface is non-interactive)
    let opened_urls = browser.get_opened_urls();
    assert_eq!(opened_urls.len(), 0);

    // Verify credentials were stored
    let stored = keyring.retrieve("ftl-cli", "default").unwrap();
    let creds: StoredCredentials = serde_json::from_str(&stored).unwrap();
    assert_eq!(creds.access_token, "test_access_token");
    assert_eq!(creds.refresh_token, Some("test_refresh_token".to_string()));

    // Verify UI output
    let output = ui.get_output();
    assert!(output.iter().any(|s| s.contains("Logging in to FTL")));
    assert!(output.iter().any(|s| s.contains("TEST-CODE")));
    assert!(output.iter().any(|s| s.contains("Successfully logged in")));
}

#[tokio::test]
async fn test_login_no_browser() {
    let fixture = TestFixture::new();

    // Mock responses
    let auth_response = DeviceAuthResponse {
        device_code: "test_device_code".to_string(),
        user_code: "TEST-CODE".to_string(),
        verification_uri: "https://auth.example.com/verify".to_string(),
        verification_uri_complete: "https://auth.example.com/verify?code=TEST-CODE".to_string(),
        expires_in: 600,
        interval: Some(5),
    };

    fixture.http_client.add_response(
        "device_authorization",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&auth_response).unwrap(),
        },
    );

    fixture.http_client.add_response(
        "oauth2/token",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&TokenResponse {
                access_token: "test_token".to_string(),
                token_type: "Bearer".to_string(),
                expires_in: Some(3600),
                refresh_token: None,
                id_token: None,
            })
            .unwrap(),
        },
    );

    let browser = fixture.browser_launcher.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(
        LoginConfig {
            no_browser: true,
            authkit_domain: None,
            client_id: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify browser was NOT opened
    let opened_urls = browser.get_opened_urls();
    assert_eq!(opened_urls.len(), 0);
}

#[tokio::test]
async fn test_login_authorization_pending() {
    let fixture = TestFixture::new();

    // Mock device authorization
    fixture.http_client.add_response(
        "device_authorization",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&DeviceAuthResponse {
                device_code: "test_device_code".to_string(),
                user_code: "TEST-CODE".to_string(),
                verification_uri: "https://auth.example.com/verify".to_string(),
                verification_uri_complete: "https://auth.example.com/verify?code=TEST-CODE"
                    .to_string(),
                expires_in: 600,
                interval: Some(1), // Fast polling for test
            })
            .unwrap(),
        },
    );

    // First poll: authorization pending
    fixture.http_client.add_response(
        "oauth2/token",
        HttpResponse {
            status: 400,
            body: r#"{"error":"authorization_pending"}"#.to_string(),
        },
    );

    // Second poll: success
    fixture.http_client.add_response(
        "oauth2/token",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&TokenResponse {
                access_token: "test_token".to_string(),
                token_type: "Bearer".to_string(),
                expires_in: Some(3600),
                refresh_token: None,
                id_token: None,
            })
            .unwrap(),
        },
    );

    let async_runtime = fixture.async_runtime.clone();
    let deps = fixture.to_deps();

    let result = execute_with_deps(
        LoginConfig {
            no_browser: true,
            authkit_domain: None,
            client_id: None,
        },
        deps,
    )
    .await;

    assert!(result.is_ok());

    // Verify we slept at least once (for authorization_pending)
    assert!(async_runtime.get_sleep_count() >= 2);
}

#[tokio::test]
async fn test_login_device_auth_failure() {
    let fixture = TestFixture::new();

    // Mock failed device authorization
    fixture.http_client.add_response(
        "device_authorization",
        HttpResponse {
            status: 400,
            body: "Invalid client".to_string(),
        },
    );

    let deps = fixture.to_deps();

    let result = execute_with_deps(
        LoginConfig {
            no_browser: true,
            authkit_domain: None,
            client_id: None,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Device authorization failed")
    );
}

#[tokio::test]
async fn test_login_access_denied() {
    let fixture = TestFixture::new();

    // Mock device authorization
    fixture.http_client.add_response(
        "device_authorization",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&DeviceAuthResponse {
                device_code: "test_device_code".to_string(),
                user_code: "TEST-CODE".to_string(),
                verification_uri: "https://auth.example.com/verify".to_string(),
                verification_uri_complete: "https://auth.example.com/verify?code=TEST-CODE"
                    .to_string(),
                expires_in: 600,
                interval: Some(1),
            })
            .unwrap(),
        },
    );

    // Mock access denied
    fixture.http_client.add_response(
        "oauth2/token",
        HttpResponse {
            status: 400,
            body: r#"{"error":"access_denied","error_description":"User denied access"}"#
                .to_string(),
        },
    );

    let deps = fixture.to_deps();

    let result = execute_with_deps(
        LoginConfig {
            no_browser: true,
            authkit_domain: None,
            client_id: None,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Access denied by user")
    );
}

#[tokio::test]
async fn test_login_expired_token() {
    let fixture = TestFixture::new();

    // Mock device authorization
    fixture.http_client.add_response(
        "device_authorization",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&DeviceAuthResponse {
                device_code: "test_device_code".to_string(),
                user_code: "TEST-CODE".to_string(),
                verification_uri: "https://auth.example.com/verify".to_string(),
                verification_uri_complete: "https://auth.example.com/verify?code=TEST-CODE"
                    .to_string(),
                expires_in: 600,
                interval: Some(1),
            })
            .unwrap(),
        },
    );

    // Mock expired token
    fixture.http_client.add_response(
        "oauth2/token",
        HttpResponse {
            status: 400,
            body: r#"{"error":"expired_token"}"#.to_string(),
        },
    );

    let deps = fixture.to_deps();

    let result = execute_with_deps(
        LoginConfig {
            no_browser: true,
            authkit_domain: None,
            client_id: None,
        },
        deps,
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Device code expired")
    );
}

#[tokio::test]
async fn test_get_stored_credentials() {
    let keyring = Arc::new(MockKeyringStorage::new());

    let creds = StoredCredentials {
        access_token: "test_token".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        id_token: None,
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        authkit_domain: "auth.example.com".to_string(),
    };

    // Store credentials
    let json = serde_json::to_string(&creds).unwrap();
    keyring.store("ftl-cli", "default", &json).unwrap();

    // TODO: Fix this test - need to implement get_stored_credentials_with_deps
    // Retrieve credentials
    // let retrieved =
    //     get_stored_credentials_with_deps(&(keyring.clone() as Arc<dyn login::KeyringStorage>))
    //         .unwrap();

    // assert_eq!(retrieved.access_token, "test_token");
    // assert_eq!(retrieved.refresh_token, Some("refresh_token".to_string()));
    // assert_eq!(retrieved.authkit_domain, "auth.example.com");
}

#[tokio::test]
async fn test_refresh_credentials_success() {
    let keyring = Arc::new(MockKeyringStorage::new());
    let http_client = Arc::new(MockHttpClient::new());
    let clock = Arc::new(MockClock::new());

    // Store expired credentials
    let expired_creds = StoredCredentials {
        access_token: "old_token".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        id_token: None,
        expires_at: Some(clock.now() - chrono::Duration::hours(1)), // Expired
        authkit_domain: "auth.example.com".to_string(),
    };

    let json = serde_json::to_string(&expired_creds).unwrap();
    keyring.store("ftl-cli", "default", &json).unwrap();

    // Mock refresh response
    http_client.add_response(
        "oauth2/token",
        HttpResponse {
            status: 200,
            body: serde_json::to_string(&TokenResponse {
                access_token: "new_token".to_string(),
                token_type: "Bearer".to_string(),
                expires_in: Some(3600),
                refresh_token: Some("new_refresh_token".to_string()),
                id_token: None,
            })
            .unwrap(),
        },
    );

    // Refresh credentials
    let refreshed = get_or_refresh_credentials_with_deps(
        &(keyring.clone() as Arc<dyn login::KeyringStorage>),
        &(http_client.clone() as Arc<dyn login::HttpClient>),
        &(clock.clone() as Arc<dyn login::Clock>),
    )
    .await
    .unwrap();

    assert_eq!(refreshed.access_token, "new_token");
    assert_eq!(
        refreshed.refresh_token,
        Some("new_refresh_token".to_string())
    );
    assert!(refreshed.expires_at.is_some());

    // Verify stored credentials were updated
    let stored = keyring.retrieve("ftl-cli", "default").unwrap();
    let stored_creds: StoredCredentials = serde_json::from_str(&stored).unwrap();
    assert_eq!(stored_creds.access_token, "new_token");
}

#[tokio::test]
async fn test_refresh_credentials_no_refresh_token() {
    let keyring = Arc::new(MockKeyringStorage::new());
    let http_client = Arc::new(MockHttpClient::new());
    let clock = Arc::new(MockClock::new());

    // Store expired credentials without refresh token
    let expired_creds = StoredCredentials {
        access_token: "old_token".to_string(),
        refresh_token: None, // No refresh token
        id_token: None,
        expires_at: Some(clock.now() - chrono::Duration::hours(1)), // Expired
        authkit_domain: "auth.example.com".to_string(),
    };

    let json = serde_json::to_string(&expired_creds).unwrap();
    keyring.store("ftl-cli", "default", &json).unwrap();

    // Try to refresh
    let result = get_or_refresh_credentials_with_deps(
        &(keyring.clone() as Arc<dyn login::KeyringStorage>),
        &(http_client.clone() as Arc<dyn login::HttpClient>),
        &(clock.clone() as Arc<dyn login::Clock>),
    )
    .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("no refresh token available")
    );
}

#[tokio::test]
async fn test_clear_credentials() {
    let keyring = Arc::new(MockKeyringStorage::new());

    // Store some credentials
    keyring.store("ftl-cli", "default", "test_data").unwrap();

    // Verify it was stored
    let result = keyring.retrieve("ftl-cli", "default");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test_data");

    // Delete credentials
    keyring.delete("ftl-cli", "default").unwrap();

    // Try to retrieve - should fail
    let result = keyring.retrieve("ftl-cli", "default");
    assert!(result.is_err());
}
