package auth

import (
	"time"
)

// Credentials represents stored authentication credentials
type Credentials struct {
	// AuthKit domain used for authentication
	AuthKitDomain string `json:"authkit_domain"`
	// OAuth access token
	AccessToken string `json:"access_token"`
	// OAuth refresh token for renewing access
	RefreshToken string `json:"refresh_token,omitempty"`
	// Token expiration time
	ExpiresAt *time.Time `json:"expires_at,omitempty"`
	// Client ID used for authentication
	ClientID string `json:"client_id,omitempty"`
}

// IsExpired checks if the access token has expired
func (c *Credentials) IsExpired() bool {
	if c.ExpiresAt == nil {
		return false // No expiry means token doesn't expire
	}
	return time.Now().After(*c.ExpiresAt)
}

// TimeUntilExpiry returns the duration until token expiry
func (c *Credentials) TimeUntilExpiry() time.Duration {
	if c.ExpiresAt == nil {
		return time.Duration(0) // No expiry
	}
	return time.Until(*c.ExpiresAt)
}

// AuthStatus represents the current authentication status
type AuthStatus struct {
	LoggedIn      bool
	Credentials   *Credentials
	Error         error
	NeedsRefresh  bool
}

// DeviceAuthResponse represents the response from device authorization endpoint
type DeviceAuthResponse struct {
	DeviceCode              string `json:"device_code"`
	UserCode                string `json:"user_code"`
	VerificationURI         string `json:"verification_uri"`
	VerificationURIComplete string `json:"verification_uri_complete"`
	ExpiresIn               int    `json:"expires_in"`
	Interval                int    `json:"interval,omitempty"`
}

// TokenResponse represents the response from token endpoint
type TokenResponse struct {
	AccessToken  string `json:"access_token"`
	TokenType    string `json:"token_type"`
	ExpiresIn    int    `json:"expires_in,omitempty"`
	RefreshToken string `json:"refresh_token,omitempty"`
	IDToken      string `json:"id_token,omitempty"`
}

// TokenError represents an error response from token endpoint
type TokenError struct {
	ErrorCode        string `json:"error"`
	ErrorDescription string `json:"error_description,omitempty"`
}

// IsAuthorizationPending checks if the error indicates pending authorization
func (e *TokenError) IsAuthorizationPending() bool {
	return e.ErrorCode == "authorization_pending"
}

// IsSlowDown checks if we should slow down polling
func (e *TokenError) IsSlowDown() bool {
	return e.ErrorCode == "slow_down"
}

// IsExpired checks if the device code has expired
func (e *TokenError) IsExpired() bool {
	return e.ErrorCode == "expired_token"
}

// LoginConfig contains configuration for the login process
type LoginConfig struct {
	// Don't open browser automatically
	NoBrowser bool
	// Override AuthKit domain (for testing)
	AuthKitDomain string
	// Override OAuth client ID (for testing)
	ClientID string
	// Force re-authentication even if already logged in
	Force bool
}

// Constants for OAuth configuration
const (
	// Default OAuth client ID for FTL authentication
	DefaultClientID = "client_01K2ADMPRAFT9X83PFVJBQ6T49"
	// Default AuthKit domain for authentication
	DefaultAuthKitDomain = "divine-lion-50-staging.authkit.app"
	// Maximum time to wait for login completion
	LoginTimeout = 30 * time.Minute
	// Keyring service name
	KeyringService = "ftl-cli"
	// Keyring username
	KeyringUsername = "default"
)