package auth

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/zalando/go-keyring"
)

// CredentialStore provides secure storage for authentication credentials
type CredentialStore interface {
	// Load retrieves stored credentials
	Load() (*Credentials, error)
	// Save stores credentials securely
	Save(creds *Credentials) error
	// Delete removes stored credentials
	Delete() error
	// Exists checks if credentials are stored
	Exists() bool

	// M2M-specific methods
	StoreToken(token string, expiresIn int) error
	GetM2MConfig() (*M2MConfig, error)
	StoreM2MConfig(config *M2MConfig) error
	SetActorType(actorType string) error
	GetActorType() (string, error)
}

// KeyringStore implements CredentialStore using OS keyring
type KeyringStore struct{}

// NewKeyringStore creates a new keyring-based credential store
func NewKeyringStore() (*KeyringStore, error) {
	// The zalando keyring library handles backend selection automatically
	return &KeyringStore{}, nil
}

// Load retrieves stored credentials from the keyring
func (s *KeyringStore) Load() (*Credentials, error) {
	data, err := keyring.Get(KeyringService, KeyringUsername)
	if err != nil {
		if err == keyring.ErrNotFound {
			return nil, fmt.Errorf("not logged in")
		}
		return nil, fmt.Errorf("failed to load credentials: %w", err)
	}

	var creds Credentials
	if err := json.Unmarshal([]byte(data), &creds); err != nil {
		return nil, fmt.Errorf("failed to parse credentials: %w", err)
	}

	return &creds, nil
}

// Save stores credentials in the keyring
func (s *KeyringStore) Save(creds *Credentials) error {
	if creds == nil {
		return fmt.Errorf("cannot save nil credentials")
	}

	data, err := json.Marshal(creds)
	if err != nil {
		return fmt.Errorf("failed to marshal credentials: %w", err)
	}

	if err := keyring.Set(KeyringService, KeyringUsername, string(data)); err != nil {
		return fmt.Errorf("failed to save credentials: %w", err)
	}

	return nil
}

// Delete removes stored credentials from the keyring
func (s *KeyringStore) Delete() error {
	err := keyring.Delete(KeyringService, KeyringUsername)
	if err != nil && err != keyring.ErrNotFound {
		return fmt.Errorf("failed to delete credentials: %w", err)
	}
	return nil
}

// Exists checks if credentials are stored
func (s *KeyringStore) Exists() bool {
	_, err := keyring.Get(KeyringService, KeyringUsername)
	return err == nil
}

// StoreToken stores just an access token (for M2M flows)
func (s *KeyringStore) StoreToken(token string, expiresIn int) error {
	creds := &Credentials{
		AccessToken: token,
	}
	// Calculate expiration time if provided
	if expiresIn > 0 {
		expiresAt := time.Now().Add(time.Duration(expiresIn) * time.Second)
		creds.ExpiresAt = &expiresAt
	}
	return s.Save(creds)
}

// GetM2MConfig retrieves stored M2M configuration
func (s *KeyringStore) GetM2MConfig() (*M2MConfig, error) {
	data, err := keyring.Get(KeyringService, "m2m-config")
	if err != nil {
		if err == keyring.ErrNotFound {
			return nil, fmt.Errorf("no M2M configuration found")
		}
		return nil, fmt.Errorf("failed to load M2M config: %w", err)
	}

	var config M2MConfig
	if err := json.Unmarshal([]byte(data), &config); err != nil {
		return nil, fmt.Errorf("failed to parse M2M config: %w", err)
	}

	return &config, nil
}

// StoreM2MConfig stores M2M configuration
func (s *KeyringStore) StoreM2MConfig(config *M2MConfig) error {
	data, err := json.Marshal(config)
	if err != nil {
		return fmt.Errorf("failed to marshal M2M config: %w", err)
	}

	if err := keyring.Set(KeyringService, "m2m-config", string(data)); err != nil {
		return fmt.Errorf("failed to store M2M config: %w", err)
	}

	return nil
}

// SetActorType stores whether the current actor is a user or machine
func (s *KeyringStore) SetActorType(actorType string) error {
	return keyring.Set(KeyringService, "actor-type", actorType)
}

// GetActorType retrieves the stored actor type
func (s *KeyringStore) GetActorType() (string, error) {
	actorType, err := keyring.Get(KeyringService, "actor-type")
	if err != nil {
		if err == keyring.ErrNotFound {
			return "", fmt.Errorf("actor type not set")
		}
		return "", fmt.Errorf("failed to get actor type: %w", err)
	}
	return actorType, nil
}

// fileKeyringPrompt provides a password for the file-based keyring fallback
func fileKeyringPrompt(prompt string) (string, error) {
	// In production, this would prompt the user
	// For now, use a static key derived from the service name
	// This is only used as a fallback when OS keyring is unavailable
	return "ftl-keyring-encryption-key", nil
}

// MockStore implements CredentialStore for testing
type MockStore struct {
	creds     *Credentials
	m2mConfig *M2MConfig
	actorType string
	err       error
}

// NewMockStore creates a mock credential store for testing
func NewMockStore(creds *Credentials, err error) *MockStore {
	return &MockStore{creds: creds, err: err}
}

// Load returns the mock credentials
func (m *MockStore) Load() (*Credentials, error) {
	if m.err != nil {
		return nil, m.err
	}
	return m.creds, nil
}

// Save stores the mock credentials
func (m *MockStore) Save(creds *Credentials) error {
	if m.err != nil {
		return m.err
	}
	m.creds = creds
	return nil
}

// Delete clears the mock credentials
func (m *MockStore) Delete() error {
	if m.err != nil {
		return m.err
	}
	m.creds = nil
	return nil
}

// Exists checks if mock credentials exist
func (m *MockStore) Exists() bool {
	return m.creds != nil
}

// StoreToken stores just an access token (for M2M flows)
func (m *MockStore) StoreToken(token string, expiresIn int) error {
	if m.err != nil {
		return m.err
	}
	m.creds = &Credentials{
		AccessToken: token,
	}
	// Calculate expiration time if provided
	if expiresIn > 0 {
		expiresAt := time.Now().Add(time.Duration(expiresIn) * time.Second)
		m.creds.ExpiresAt = &expiresAt
	}
	return nil
}

// GetM2MConfig retrieves stored M2M configuration
func (m *MockStore) GetM2MConfig() (*M2MConfig, error) {
	if m.err != nil {
		return nil, m.err
	}
	return m.m2mConfig, nil
}

// StoreM2MConfig stores M2M configuration
func (m *MockStore) StoreM2MConfig(config *M2MConfig) error {
	if m.err != nil {
		return m.err
	}
	m.m2mConfig = config
	return nil
}

// SetActorType stores whether the current actor is a user or machine
func (m *MockStore) SetActorType(actorType string) error {
	if m.err != nil {
		return m.err
	}
	m.actorType = actorType
	return nil
}

// GetActorType retrieves the stored actor type
func (m *MockStore) GetActorType() (string, error) {
	if m.err != nil {
		return "", m.err
	}
	if m.actorType == "" {
		return "", fmt.Errorf("actor type not set")
	}
	return m.actorType, nil
}
