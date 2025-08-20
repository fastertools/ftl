package auth

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"strings"
	"time"
)

// M2MConfig holds configuration for machine-to-machine authentication
type M2MConfig struct {
	ClientID     string `json:"client_id"`
	ClientSecret string `json:"client_secret"`
	Issuer       string `json:"issuer,omitempty"`
	OrgID        string `json:"org_id,omitempty"` // Set after token exchange
}

// M2MTokenResponse represents the response from the token endpoint
type M2MTokenResponse struct {
	AccessToken string `json:"access_token"`
	TokenType   string `json:"token_type"`
	ExpiresIn   int    `json:"expires_in"`
}

// M2MManager handles machine-to-machine authentication
type M2MManager struct {
	store  CredentialStore
	client *http.Client
}

// NewM2MManager creates a new M2M authentication manager
func NewM2MManager(store CredentialStore) *M2MManager {
	return &M2MManager{
		store: store,
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// LoadM2MConfig loads M2M configuration from environment or stored config
func (m *M2MManager) LoadM2MConfig() (*M2MConfig, error) {
	// First check environment variables (highest priority for CI/CD)
	clientID := os.Getenv("FTL_CLIENT_ID")
	clientSecret := os.Getenv("FTL_CLIENT_SECRET")
	issuer := os.Getenv("FTL_ISSUER")

	if clientID != "" && clientSecret != "" {
		config := &M2MConfig{
			ClientID:     clientID,
			ClientSecret: clientSecret,
			Issuer:       issuer,
		}

		// Use default issuer if not provided
		if config.Issuer == "" {
			config.Issuer = DefaultAuthKitDomain
		}

		return config, nil
	}

	// Check for stored M2M config
	storedConfig, err := m.store.GetM2MConfig()
	if err == nil && storedConfig != nil {
		return storedConfig, nil
	}

	return nil, fmt.Errorf("no M2M credentials found. Set FTL_CLIENT_ID and FTL_CLIENT_SECRET environment variables")
}

// ExchangeCredentials exchanges client credentials for an access token
func (m *M2MManager) ExchangeCredentials(ctx context.Context, config *M2MConfig) (*TokenResponse, error) {
	if config == nil {
		return nil, fmt.Errorf("M2M config is required")
	}

	// Construct token endpoint URL
	tokenURL := fmt.Sprintf("%s/oauth2/token", strings.TrimSuffix(config.Issuer, "/"))

	// Prepare request body
	data := url.Values{}
	data.Set("grant_type", "client_credentials")
	data.Set("client_id", config.ClientID)
	data.Set("client_secret", config.ClientSecret)

	// Create request
	req, err := http.NewRequestWithContext(ctx, "POST", tokenURL, strings.NewReader(data.Encode()))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	req.Header.Set("Accept", "application/json")

	// Make request
	resp, err := m.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to exchange credentials: %w", err)
	}
	defer resp.Body.Close()

	// Read response
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	// Check for errors
	if resp.StatusCode != http.StatusOK {
		var errorResp struct {
			Error            string `json:"error"`
			ErrorDescription string `json:"error_description"`
		}

		if err := json.Unmarshal(body, &errorResp); err == nil && errorResp.Error != "" {
			return nil, fmt.Errorf("authentication failed: %s - %s", errorResp.Error, errorResp.ErrorDescription)
		}

		return nil, fmt.Errorf("authentication failed with status %d: %s", resp.StatusCode, string(body))
	}

	// Parse successful response
	var tokenResp M2MTokenResponse
	if err := json.Unmarshal(body, &tokenResp); err != nil {
		return nil, fmt.Errorf("failed to parse token response: %w", err)
	}

	// Extract org_id from the JWT token (we'd need to decode it)
	// For now, we'll trust the token contains the correct org_id claim

	// Convert to our standard TokenResponse format
	return &TokenResponse{
		AccessToken: tokenResp.AccessToken,
		TokenType:   tokenResp.TokenType,
		ExpiresIn:   tokenResp.ExpiresIn,
	}, nil
}

// LoginMachine performs machine login using client credentials
func (m *Manager) LoginMachine(ctx context.Context) error {
	// Create M2M manager
	m2mManager := NewM2MManager(m.store)

	// Load M2M config
	config, err := m2mManager.LoadM2MConfig()
	if err != nil {
		return fmt.Errorf("failed to load M2M configuration: %w", err)
	}

	// Exchange credentials for token
	tokenResp, err := m2mManager.ExchangeCredentials(ctx, config)
	if err != nil {
		return fmt.Errorf("failed to exchange credentials: %w", err)
	}

	// Store the token
	if err := m.store.StoreToken(tokenResp.AccessToken, tokenResp.ExpiresIn); err != nil {
		return fmt.Errorf("failed to store token: %w", err)
	}

	// Store that this is a machine token
	if err := m.store.SetActorType("machine"); err != nil {
		// Non-fatal, just log it
		fmt.Printf("Warning: failed to store actor type: %v\n", err)
	}

	return nil
}

// LoginMachineWithToken logs in using a pre-existing M2M token
func (m *Manager) LoginMachineWithToken(ctx context.Context, token string) error {
	// Validate token format (basic check)
	token = strings.TrimSpace(token)
	if token == "" {
		return fmt.Errorf("token cannot be empty")
	}

	// Check if it looks like a JWT (has 3 parts separated by dots)
	parts := strings.Split(token, ".")
	if len(parts) != 3 {
		return fmt.Errorf("invalid token format: expected JWT with 3 parts")
	}

	// Store the token with a default expiry (we can't know the actual expiry without decoding)
	// Using 1 hour as a reasonable default for M2M tokens
	if err := m.store.StoreToken(token, 3600); err != nil {
		return fmt.Errorf("failed to store token: %w", err)
	}

	// Mark as machine actor
	if err := m.store.SetActorType("machine"); err != nil {
		// Non-fatal, just log it
		fmt.Printf("Warning: failed to store actor type: %v\n", err)
	}

	return nil
}

// ConfigureM2M stores M2M credentials for later use
func (m *Manager) ConfigureM2M(config *M2MConfig) error {
	if config.ClientID == "" || config.ClientSecret == "" {
		return fmt.Errorf("client_id and client_secret are required")
	}

	// Set default issuer if not provided
	if config.Issuer == "" {
		config.Issuer = DefaultAuthKitDomain
	}

	// Store the configuration
	if err := m.store.StoreM2MConfig(config); err != nil {
		return fmt.Errorf("failed to store M2M configuration: %w", err)
	}

	return nil
}

// GetActorType returns whether the current actor is a user or machine
func (m *Manager) GetActorType(ctx context.Context) (string, error) {
	// First check if we have a stored actor type
	actorType, err := m.store.GetActorType()
	if err == nil && actorType != "" {
		return actorType, nil
	}

	// Check if we have M2M credentials configured
	m2mManager := NewM2MManager(m.store)
	if config, err := m2mManager.LoadM2MConfig(); err == nil && config != nil {
		return "machine", nil
	}

	// Default to user
	return "user", nil
}

// IsM2MConfigured checks if M2M credentials are available
func IsM2MConfigured() bool {
	// Check environment variables
	clientID := os.Getenv("FTL_CLIENT_ID")
	clientSecret := os.Getenv("FTL_CLIENT_SECRET")

	return clientID != "" && clientSecret != ""
}

// GetM2MTokenFromEnv gets a pre-generated M2M token from environment
func GetM2MTokenFromEnv() string {
	return os.Getenv("FTL_M2M_TOKEN")
}
