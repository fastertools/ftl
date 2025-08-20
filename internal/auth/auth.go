package auth

import (
	"context"
	"fmt"
	"time"

	"github.com/fastertools/ftl-cli/internal/config"
	"github.com/pkg/browser"
)

// Manager handles authentication operations
type Manager struct {
	store         CredentialStore
	oauthProvider OAuthProvider
	browserOpener BrowserOpener
	config        *LoginConfig
}

// defaultBrowserOpener implements BrowserOpener using the browser package
type defaultBrowserOpener struct{}

func (d *defaultBrowserOpener) OpenURL(url string) error {
	return browser.OpenURL(url)
}

// NewManager creates a new authentication manager
func NewManager(store CredentialStore, config *LoginConfig) *Manager {
	if config == nil {
		config = &LoginConfig{
			AuthKitDomain: DefaultAuthKitDomain,
			ClientID:      DefaultClientID,
		}
	}

	return &Manager{
		store:         store,
		oauthProvider: NewOAuthClient(config.AuthKitDomain, config.ClientID),
		browserOpener: &defaultBrowserOpener{},
		config:        config,
	}
}

// NewManagerWithProvider creates a new authentication manager with a custom OAuth provider
// This is primarily for testing but can be used for custom OAuth implementations
func NewManagerWithProvider(store CredentialStore, provider OAuthProvider, config *LoginConfig) *Manager {
	if config == nil {
		config = &LoginConfig{
			AuthKitDomain: DefaultAuthKitDomain,
			ClientID:      DefaultClientID,
		}
	}

	return &Manager{
		store:         store,
		oauthProvider: provider,
		browserOpener: &defaultBrowserOpener{},
		config:        config,
	}
}

// NewManagerWithMocks creates a new authentication manager with all dependencies mocked
// This is specifically for testing to prevent any external interactions
func NewManagerWithMocks(store CredentialStore, provider OAuthProvider, browser BrowserOpener, config *LoginConfig) *Manager {
	if config == nil {
		config = &LoginConfig{
			AuthKitDomain: DefaultAuthKitDomain,
			ClientID:      DefaultClientID,
		}
	}

	// Always disable browser in tests, regardless of config
	config.NoBrowser = true

	return &Manager{
		store:         store,
		oauthProvider: provider,
		browserOpener: browser,
		config:        config,
	}
}

// StartDeviceFlow starts the OAuth device flow and returns device auth info
func (m *Manager) StartDeviceFlow(ctx context.Context) (*DeviceAuthResponse, error) {
	// Check if already logged in (unless force is set)
	if !m.config.Force {
		if creds, err := m.store.Load(); err == nil && creds != nil {
			if !creds.IsExpired() {
				return nil, fmt.Errorf("already logged in")
			}
			// Try to refresh if we have a refresh token
			if creds.RefreshToken != "" {
				if _, err := m.Refresh(ctx, creds); err == nil {
					return nil, fmt.Errorf("already logged in (token refreshed)")
				}
			}
		}
	}

	// Start device flow
	deviceAuth, err := m.oauthProvider.StartDeviceFlow(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to start device flow: %w", err)
	}

	// Open browser if not disabled
	if !m.config.NoBrowser && m.browserOpener != nil {
		_ = m.browserOpener.OpenURL(deviceAuth.VerificationURIComplete)
	}

	return deviceAuth, nil
}

// CompleteDeviceFlow completes the device flow authentication
func (m *Manager) CompleteDeviceFlow(ctx context.Context, deviceAuth *DeviceAuthResponse) (*Credentials, error) {
	// Poll for token
	interval := time.Duration(deviceAuth.Interval) * time.Second
	token, err := m.oauthProvider.PollForToken(ctx, deviceAuth.DeviceCode, interval)
	if err != nil {
		return nil, fmt.Errorf("failed to complete login: %w", err)
	}

	// Create credentials
	creds := &Credentials{
		AuthKitDomain: m.config.AuthKitDomain,
		AccessToken:   token.AccessToken,
		RefreshToken:  token.RefreshToken,
		ClientID:      m.config.ClientID,
	}

	// Calculate expiration time
	if token.ExpiresIn > 0 {
		expiresAt := time.Now().Add(time.Duration(token.ExpiresIn) * time.Second)
		creds.ExpiresAt = &expiresAt
	}

	// Save credentials
	if err := m.store.Save(creds); err != nil {
		return nil, fmt.Errorf("failed to save credentials: %w", err)
	}

	// Extract and save user info from JWT
	if err := m.SaveUserInfoFromToken(token); err != nil {
		// Non-fatal, just log it
		fmt.Printf("Warning: failed to save user info: %v\n", err)
	}

	// Mark as user actor (not machine)
	if err := m.store.SetActorType("user"); err != nil {
		// Non-fatal, just log it
		fmt.Printf("Warning: failed to store actor type: %v\n", err)
	}

	return creds, nil
}

// Login performs the complete OAuth device flow login
func (m *Manager) Login(ctx context.Context) (*Credentials, error) {
	deviceAuth, err := m.StartDeviceFlow(ctx)
	if err != nil {
		return nil, err
	}
	return m.CompleteDeviceFlow(ctx, deviceAuth)
}

// Logout removes stored credentials
func (m *Manager) Logout() error {
	return m.store.Delete()
}

// Status returns the current authentication status
func (m *Manager) Status() *AuthStatus {
	creds, err := m.store.Load()
	if err != nil || creds == nil {
		return &AuthStatus{
			LoggedIn: false,
			Error:    err,
		}
	}

	status := &AuthStatus{
		LoggedIn:    true,
		Credentials: creds,
	}

	if creds.IsExpired() {
		status.NeedsRefresh = true
	}

	return status
}

// GetToken returns the current access token, refreshing if necessary
func (m *Manager) GetToken(ctx context.Context) (string, error) {
	creds, err := m.store.Load()
	if err != nil || creds == nil {
		return "", fmt.Errorf("not logged in")
	}

	// Check if token needs refresh
	if creds.IsExpired() {
		if creds.RefreshToken == "" {
			return "", fmt.Errorf("token expired and no refresh token available")
		}
		refreshed, err := m.Refresh(ctx, creds)
		if err != nil {
			return "", fmt.Errorf("failed to refresh token: %w", err)
		}
		creds = refreshed
	}

	return creds.AccessToken, nil
}

// Refresh refreshes an expired access token
func (m *Manager) Refresh(ctx context.Context, creds *Credentials) (*Credentials, error) {
	if creds.RefreshToken == "" {
		return nil, fmt.Errorf("no refresh token available")
	}

	token, err := m.oauthProvider.RefreshToken(ctx, creds.RefreshToken)
	if err != nil {
		return nil, fmt.Errorf("failed to refresh token: %w", err)
	}

	// Update credentials
	newCreds := &Credentials{
		AuthKitDomain: creds.AuthKitDomain,
		AccessToken:   token.AccessToken,
		RefreshToken:  creds.RefreshToken, // Keep existing refresh token if not provided
		ClientID:      creds.ClientID,
	}

	// Update refresh token if provided
	if token.RefreshToken != "" {
		newCreds.RefreshToken = token.RefreshToken
	}

	// Calculate expiration time
	if token.ExpiresIn > 0 {
		expiresAt := time.Now().Add(time.Duration(token.ExpiresIn) * time.Second)
		newCreds.ExpiresAt = &expiresAt
	}

	// Save updated credentials
	if err := m.store.Save(newCreds); err != nil {
		return nil, fmt.Errorf("failed to save refreshed credentials: %w", err)
	}

	return newCreds, nil
}

// GetOrRefreshToken gets a valid token, refreshing if necessary
func (m *Manager) GetOrRefreshToken(ctx context.Context) (string, error) {
	return m.GetToken(ctx)
}

// SaveUserInfoFromToken extracts and saves user info from a token
func (m *Manager) SaveUserInfoFromToken(token *TokenResponse) error {
	// Import config package
	cfg, err := config.Load()
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}
	
	// Extract user info from JWT
	claims, err := ExtractIDToken(token)
	if err != nil {
		// Try extracting from access token
		claims, err = ExtractUserInfo(token.AccessToken)
		if err != nil {
			return fmt.Errorf("failed to extract user info: %w", err)
		}
	}
	
	// Save user info to config
	userInfo := &config.UserInfo{
		Username:  claims.GetDisplayName(),
		Email:     claims.Email,
		UserID:    claims.UserID,
		UpdatedAt: time.Now().Format(time.RFC3339),
	}
	
	return cfg.SetCurrentUser(userInfo)
}
