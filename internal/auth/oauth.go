package auth

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"
)

// OAuthClient handles OAuth device flow authentication
type OAuthClient struct {
	httpClient    HTTPClient
	authKitDomain string
	clientID      string
}

// Ensure OAuthClient implements OAuthProvider
var _ OAuthProvider = (*OAuthClient)(nil)

// NewOAuthClient creates a new OAuth client
func NewOAuthClient(authKitDomain, clientID string) *OAuthClient {
	if authKitDomain == "" {
		authKitDomain = DefaultAuthKitDomain
	}
	if clientID == "" {
		clientID = DefaultClientID
	}

	return &OAuthClient{
		httpClient:    &http.Client{Timeout: 30 * time.Second},
		authKitDomain: authKitDomain,
		clientID:      clientID,
	}
}

// StartDeviceFlow initiates the OAuth device flow
func (c *OAuthClient) StartDeviceFlow(ctx context.Context) (*DeviceAuthResponse, error) {
	endpoint := fmt.Sprintf("https://%s/oauth2/device_authorization", c.authKitDomain)

	data := url.Values{
		"client_id": {c.clientID},
		"scope":     {"openid profile email offline_access"},
	}

	req, err := http.NewRequestWithContext(ctx, "POST", endpoint, bytes.NewBufferString(data.Encode()))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to request device authorization: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("device authorization failed (status %d): %s", resp.StatusCode, string(body))
	}

	var authResp DeviceAuthResponse
	if err := json.Unmarshal(body, &authResp); err != nil {
		return nil, fmt.Errorf("failed to parse response: %w", err)
	}

	// Set default interval if not provided
	if authResp.Interval == 0 {
		authResp.Interval = 5
	}

	return &authResp, nil
}

// PollForToken polls the token endpoint until authentication completes
func (c *OAuthClient) PollForToken(ctx context.Context, deviceCode string, interval time.Duration) (*TokenResponse, error) {
	endpoint := fmt.Sprintf("https://%s/oauth2/token", c.authKitDomain)

	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	deadline := time.Now().Add(LoginTimeout)

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-ticker.C:
			if time.Now().After(deadline) {
				return nil, fmt.Errorf("login timeout exceeded")
			}

			token, err := c.requestToken(ctx, endpoint, deviceCode)
			if err != nil {
				// Check if it's a token error
				if tokenErr, ok := err.(*TokenError); ok {
					if tokenErr.IsAuthorizationPending() {
						// Continue polling
						continue
					}
					if tokenErr.IsSlowDown() {
						// Increase interval
						ticker.Reset(interval * 2)
						continue
					}
					if tokenErr.IsExpired() {
						return nil, fmt.Errorf("device code expired, please try again")
					}
				}
				return nil, err
			}

			return token, nil
		}
	}
}

// requestToken makes a single token request
func (c *OAuthClient) requestToken(ctx context.Context, endpoint, deviceCode string) (*TokenResponse, error) {
	data := url.Values{
		"grant_type":  {"urn:ietf:params:oauth:grant-type:device_code"},
		"device_code": {deviceCode},
		"client_id":   {c.clientID},
	}

	req, err := http.NewRequestWithContext(ctx, "POST", endpoint, bytes.NewBufferString(data.Encode()))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to request token: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	// Check for error response
	if resp.StatusCode == http.StatusBadRequest {
		var tokenErr TokenError
		if err := json.Unmarshal(body, &tokenErr); err != nil {
			return nil, fmt.Errorf("failed to parse error response: %w", err)
		}
		return nil, &tokenErr
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("token request failed (status %d): %s", resp.StatusCode, string(body))
	}

	var tokenResp TokenResponse
	if err := json.Unmarshal(body, &tokenResp); err != nil {
		return nil, fmt.Errorf("failed to parse token response: %w", err)
	}

	return &tokenResp, nil
}

// RefreshToken refreshes an expired access token
func (c *OAuthClient) RefreshToken(ctx context.Context, refreshToken string) (*TokenResponse, error) {
	endpoint := fmt.Sprintf("https://%s/oauth2/token", c.authKitDomain)

	data := url.Values{
		"grant_type":    {"refresh_token"},
		"refresh_token": {refreshToken},
		"client_id":     {c.clientID},
	}

	req, err := http.NewRequestWithContext(ctx, "POST", endpoint, bytes.NewBufferString(data.Encode()))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to refresh token: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("token refresh failed (status %d): %s", resp.StatusCode, string(body))
	}

	var tokenResp TokenResponse
	if err := json.Unmarshal(body, &tokenResp); err != nil {
		return nil, fmt.Errorf("failed to parse token response: %w", err)
	}

	return &tokenResp, nil
}

// Error implements the error interface for TokenError
func (e *TokenError) Error() string {
	if e.ErrorDescription != "" {
		return fmt.Sprintf("%s: %s", e.ErrorCode, e.ErrorDescription)
	}
	return e.ErrorCode
}
