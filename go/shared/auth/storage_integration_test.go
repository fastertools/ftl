package auth

import (
	"testing"
	"time"

	"github.com/zalando/go-keyring"
)

func TestKeyringStore_Integration(t *testing.T) {
	// Use the mock keyring for testing
	keyring.MockInit()

	store, err := NewKeyringStore()
	if err != nil {
		t.Fatalf("NewKeyringStore() error = %v", err)
	}

	// Test 1: Initially no credentials
	if store.Exists() {
		t.Error("Exists() = true, want false for new store")
	}

	_, err = store.Load()
	if err == nil {
		t.Error("Load() should error when no credentials exist")
	}

	// Test 2: Save credentials
	creds := &Credentials{
		AuthKitDomain: "test.auth.example.com",
		AccessToken:   "test-access-token",
		RefreshToken:  "test-refresh-token",
		ExpiresAt:     timePtr(time.Date(2025, 12, 31, 23, 59, 59, 0, time.UTC)),
	}

	err = store.Save(creds)
	if err != nil {
		t.Fatalf("Save() error = %v", err)
	}

	// Test 3: Credentials should exist now
	if !store.Exists() {
		t.Error("Exists() = false, want true after save")
	}

	// Test 4: Load credentials
	loaded, err := store.Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if loaded.AuthKitDomain != creds.AuthKitDomain {
		t.Errorf("AuthKitDomain = %v, want %v", loaded.AuthKitDomain, creds.AuthKitDomain)
	}
	if loaded.AccessToken != creds.AccessToken {
		t.Errorf("AccessToken = %v, want %v", loaded.AccessToken, creds.AccessToken)
	}
	if loaded.RefreshToken != creds.RefreshToken {
		t.Errorf("RefreshToken = %v, want %v", loaded.RefreshToken, creds.RefreshToken)
	}
	if !loaded.ExpiresAt.Equal(*creds.ExpiresAt) {
		t.Errorf("ExpiresAt = %v, want %v", loaded.ExpiresAt, creds.ExpiresAt)
	}

	// Test 5: Update credentials
	newCreds := &Credentials{
		AuthKitDomain: "new.auth.example.com",
		AccessToken:   "new-access-token",
		RefreshToken:  "new-refresh-token",
		ExpiresAt:     timePtr(time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)),
	}

	err = store.Save(newCreds)
	if err != nil {
		t.Fatalf("Save() update error = %v", err)
	}

	loaded, err = store.Load()
	if err != nil {
		t.Fatalf("Load() after update error = %v", err)
	}

	if loaded.AccessToken != newCreds.AccessToken {
		t.Errorf("After update, AccessToken = %v, want %v", loaded.AccessToken, newCreds.AccessToken)
	}

	// Test 6: Delete credentials
	err = store.Delete()
	if err != nil {
		t.Fatalf("Delete() error = %v", err)
	}

	if store.Exists() {
		t.Error("Exists() = true, want false after delete")
	}

	// Test 7: Delete non-existent (should not error)
	err = store.Delete()
	if err != nil {
		t.Errorf("Delete() non-existent error = %v, want nil", err)
	}

	// Test 8: Save nil credentials
	err = store.Save(nil)
	if err == nil {
		t.Error("Save(nil) should return error")
	}
}

func TestKeyringStore_ErrorHandling(t *testing.T) {
	// Test with error-inducing mock
	keyring.MockInitWithError(keyring.ErrNotFound)

	store, err := NewKeyringStore()
	if err != nil {
		t.Fatalf("NewKeyringStore() error = %v", err)
	}

	// Load should fail
	_, err = store.Load()
	if err == nil {
		t.Error("Load() should error when keyring returns error")
	}

	// Save should fail
	creds := &Credentials{
		AccessToken: "test-token",
	}
	err = store.Save(creds)
	if err == nil {
		t.Error("Save() should error when keyring returns error")
	}

	// Exists should return false
	if store.Exists() {
		t.Error("Exists() = true, want false when keyring has error")
	}
}