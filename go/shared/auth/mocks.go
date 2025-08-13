package auth

import (
	"context"
	"fmt"
	"net/http"
	"sync"
	"time"
)

// MockOAuthProvider is a mock implementation of OAuthProvider for testing
type MockOAuthProvider struct {
	mu sync.Mutex

	// StartDeviceFlow behavior
	StartDeviceFlowFunc func(ctx context.Context) (*DeviceAuthResponse, error)
	StartDeviceFlowCalls []struct {
		Ctx context.Context
	}

	// PollForToken behavior
	PollForTokenFunc func(ctx context.Context, deviceCode string, interval time.Duration) (*TokenResponse, error)
	PollForTokenCalls []struct {
		Ctx        context.Context
		DeviceCode string
		Interval   time.Duration
	}

	// RefreshToken behavior
	RefreshTokenFunc func(ctx context.Context, refreshToken string) (*TokenResponse, error)
	RefreshTokenCalls []struct {
		Ctx          context.Context
		RefreshToken string
	}
}

// StartDeviceFlow implements OAuthProvider
func (m *MockOAuthProvider) StartDeviceFlow(ctx context.Context) (*DeviceAuthResponse, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.StartDeviceFlowCalls = append(m.StartDeviceFlowCalls, struct {
		Ctx context.Context
	}{
		Ctx: ctx,
	})

	if m.StartDeviceFlowFunc != nil {
		return m.StartDeviceFlowFunc(ctx)
	}

	// Default response
	return &DeviceAuthResponse{
		DeviceCode:      "mock-device-code",
		UserCode:        "MOCK-CODE",
		VerificationURI: "https://mock.auth/device",
		ExpiresIn:       600,
		Interval:        5,
	}, nil
}

// PollForToken implements OAuthProvider
func (m *MockOAuthProvider) PollForToken(ctx context.Context, deviceCode string, interval time.Duration) (*TokenResponse, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.PollForTokenCalls = append(m.PollForTokenCalls, struct {
		Ctx        context.Context
		DeviceCode string
		Interval   time.Duration
	}{
		Ctx:        ctx,
		DeviceCode: deviceCode,
		Interval:   interval,
	})

	if m.PollForTokenFunc != nil {
		return m.PollForTokenFunc(ctx, deviceCode, interval)
	}

	// Default response
	return &TokenResponse{
		AccessToken:  "mock-access-token",
		RefreshToken: "mock-refresh-token",
		ExpiresIn:    3600,
		TokenType:    "Bearer",
	}, nil
}

// RefreshToken implements OAuthProvider
func (m *MockOAuthProvider) RefreshToken(ctx context.Context, refreshToken string) (*TokenResponse, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.RefreshTokenCalls = append(m.RefreshTokenCalls, struct {
		Ctx          context.Context
		RefreshToken string
	}{
		Ctx:          ctx,
		RefreshToken: refreshToken,
	})

	if m.RefreshTokenFunc != nil {
		return m.RefreshTokenFunc(ctx, refreshToken)
	}

	// Default response
	return &TokenResponse{
		AccessToken:  "mock-refreshed-token",
		RefreshToken: "mock-new-refresh-token",
		ExpiresIn:    3600,
		TokenType:    "Bearer",
	}, nil
}

// Reset clears all recorded calls
func (m *MockOAuthProvider) Reset() {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.StartDeviceFlowCalls = nil
	m.PollForTokenCalls = nil
	m.RefreshTokenCalls = nil
}

// Ensure MockOAuthProvider implements OAuthProvider
var _ OAuthProvider = (*MockOAuthProvider)(nil)

// MockHTTPClient is a mock implementation of HTTPClient for testing
type MockHTTPClient struct {
	mu sync.Mutex

	// Do behavior
	DoFunc func(req *http.Request) (*http.Response, error)
	DoCalls []struct {
		Req *http.Request
	}
}

// Do implements HTTPClient
func (m *MockHTTPClient) Do(req *http.Request) (*http.Response, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.DoCalls = append(m.DoCalls, struct {
		Req *http.Request
	}{
		Req: req,
	})

	if m.DoFunc != nil {
		return m.DoFunc(req)
	}

	// Default error response
	return nil, fmt.Errorf("mock HTTP client: no DoFunc configured")
}

// Reset clears all recorded calls
func (m *MockHTTPClient) Reset() {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.DoCalls = nil
}

// Ensure MockHTTPClient implements HTTPClient
var _ HTTPClient = (*MockHTTPClient)(nil)

// MockBrowserOpener is a mock implementation of BrowserOpener for testing
type MockBrowserOpener struct {
	mu sync.Mutex

	// OpenURL behavior
	OpenURLFunc func(url string) error
	OpenURLCalls []struct {
		URL string
	}
}

// OpenURL implements BrowserOpener
func (m *MockBrowserOpener) OpenURL(url string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.OpenURLCalls = append(m.OpenURLCalls, struct {
		URL string
	}{
		URL: url,
	})

	if m.OpenURLFunc != nil {
		return m.OpenURLFunc(url)
	}

	// Default: do nothing (no browser opened in tests)
	return nil
}

// Reset clears all recorded calls
func (m *MockBrowserOpener) Reset() {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.OpenURLCalls = nil
}

// Ensure MockBrowserOpener implements BrowserOpener
var _ BrowserOpener = (*MockBrowserOpener)(nil)

// MockBuilder provides a fluent interface for building test scenarios
type MockBuilder struct {
	provider *MockOAuthProvider
	store    *MockStore
	browser  *MockBrowserOpener
}

// NewMockBuilder creates a new mock builder
func NewMockBuilder() *MockBuilder {
	return &MockBuilder{
		provider: &MockOAuthProvider{},
		store:    NewMockStore(nil, nil),
		browser:  &MockBrowserOpener{},
	}
}

// WithDeviceFlow configures the device flow response
func (b *MockBuilder) WithDeviceFlow(resp *DeviceAuthResponse, err error) *MockBuilder {
	b.provider.StartDeviceFlowFunc = func(ctx context.Context) (*DeviceAuthResponse, error) {
		return resp, err
	}
	return b
}

// WithTokenPolling configures the token polling behavior
// This simulates the entire polling loop, handling authorization_pending internally
func (b *MockBuilder) WithTokenPolling(responses []interface{}) *MockBuilder {
	b.provider.PollForTokenFunc = func(ctx context.Context, deviceCode string, interval time.Duration) (*TokenResponse, error) {
		// Simulate polling with the provided responses
		for _, resp := range responses {
			// Check context cancellation
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			default:
			}

			switch v := resp.(type) {
			case *TokenResponse:
				// Success - return the token
				return v, nil
			case *TokenError:
				// Handle different error types
				if v.IsAuthorizationPending() {
					// Continue to next response (simulating continued polling)
					time.Sleep(10 * time.Millisecond) // Small delay to simulate polling
					continue
				}
				if v.IsSlowDown() {
					// Continue but with longer delay
					time.Sleep(20 * time.Millisecond)
					continue
				}
				// Other errors terminate polling
				return nil, v
			case error:
				return nil, v
			default:
				return nil, fmt.Errorf("unexpected response type: %T", v)
			}
		}
		
		// If we get here, we ran out of responses while still pending
		return nil, fmt.Errorf("polling incomplete: still pending after all responses")
	}
	return b
}

// WithRefreshToken configures the refresh token response
func (b *MockBuilder) WithRefreshToken(resp *TokenResponse, err error) *MockBuilder {
	b.provider.RefreshTokenFunc = func(ctx context.Context, refreshToken string) (*TokenResponse, error) {
		return resp, err
	}
	return b
}

// WithStoredCredentials configures the stored credentials
func (b *MockBuilder) WithStoredCredentials(creds *Credentials) *MockBuilder {
	b.store = NewMockStore(creds, nil)
	return b
}

// WithStoreError configures a store error
func (b *MockBuilder) WithStoreError(err error) *MockBuilder {
	b.store = NewMockStore(nil, err)
	return b
}

// Build creates a Manager with the configured mocks
func (b *MockBuilder) Build() (*Manager, *MockOAuthProvider, *MockStore) {
	// Use NewManagerWithMocks to ensure browser is mocked
	config := &LoginConfig{
		NoBrowser: true, // Always disable browser in tests
	}
	manager := NewManagerWithMocks(b.store, b.provider, b.browser, config)
	return manager, b.provider, b.store
}

// TestHelpers provides utility functions for tests
type TestHelpers struct{}

// NewTestHelpers creates test helpers
func NewTestHelpers() *TestHelpers {
	return &TestHelpers{}
}

// timePtr returns a pointer to a time value
func timePtr(t time.Time) *time.Time {
	return &t
}

// ExpiredCredentials creates expired test credentials
func (h *TestHelpers) ExpiredCredentials() *Credentials {
	return &Credentials{
		AuthKitDomain: "test.auth",
		AccessToken:   "expired-token",
		RefreshToken:  "refresh-token",
		ExpiresAt:     timePtr(time.Now().Add(-time.Hour)),
	}
}

// ValidCredentials creates valid test credentials
func (h *TestHelpers) ValidCredentials() *Credentials {
	return &Credentials{
		AuthKitDomain: "test.auth",
		AccessToken:   "valid-token",
		RefreshToken:  "refresh-token",
		ExpiresAt:     timePtr(time.Now().Add(time.Hour)),
	}
}

// DeviceAuthResponse creates a test device auth response
func (h *TestHelpers) DeviceAuthResponse() *DeviceAuthResponse {
	return &DeviceAuthResponse{
		DeviceCode:              "device-123",
		UserCode:                "USER-123",
		VerificationURI:         "https://auth.example.com/device",
		VerificationURIComplete: "https://auth.example.com/device?code=USER-123",
		ExpiresIn:               600,
		Interval:                5,
	}
}

// TokenResponse creates a test token response
func (h *TestHelpers) TokenResponse() *TokenResponse {
	return &TokenResponse{
		AccessToken:  "access-token",
		RefreshToken: "refresh-token",
		ExpiresIn:    3600,
		TokenType:    "Bearer",
	}
}

// AuthorizationPendingError creates an authorization pending error
func (h *TestHelpers) AuthorizationPendingError() *TokenError {
	return &TokenError{
		ErrorCode: "authorization_pending",
	}
}

// SlowDownError creates a slow down error
func (h *TestHelpers) SlowDownError() *TokenError {
	return &TokenError{
		ErrorCode: "slow_down",
	}
}

// ExpiredTokenError creates an expired token error
func (h *TestHelpers) ExpiredTokenError() *TokenError {
	return &TokenError{
		ErrorCode:        "expired_token",
		ErrorDescription: "The device code has expired",
	}
}

// AccessDeniedError creates an access denied error
func (h *TestHelpers) AccessDeniedError() *TokenError {
	return &TokenError{
		ErrorCode:        "access_denied",
		ErrorDescription: "The user denied the request",
	}
}