package auth

import (
	"encoding/json"
	"fmt"

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
}

// KeyringStore implements CredentialStore using OS keyring
type KeyringStore struct {}

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

// fileKeyringPrompt provides a password for the file-based keyring fallback
func fileKeyringPrompt(prompt string) (string, error) {
	// In production, this would prompt the user
	// For now, use a static key derived from the service name
	// This is only used as a fallback when OS keyring is unavailable
	return "ftl-cli-keyring-encryption-key", nil
}

// MockStore implements CredentialStore for testing
type MockStore struct {
	creds *Credentials
	err   error
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