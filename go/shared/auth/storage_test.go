package auth

import (
	"testing"
	"time"
)

func TestMockStore_Operations(t *testing.T) {
	// Test successful operations
	t.Run("successful operations", func(t *testing.T) {
		initialCreds := &Credentials{
			AccessToken: "initial-token",
			ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
		}
		
		store := NewMockStore(initialCreds, nil)
		
		// Test Load
		loaded, err := store.Load()
		if err != nil {
			t.Fatalf("Load() error = %v", err)
		}
		if loaded.AccessToken != initialCreds.AccessToken {
			t.Errorf("Load() token = %v, want %v", loaded.AccessToken, initialCreds.AccessToken)
		}
		
		// Test Exists
		if !store.Exists() {
			t.Error("Exists() = false, want true")
		}
		
		// Test Save
		newCreds := &Credentials{
			AccessToken: "new-token",
		}
		err = store.Save(newCreds)
		if err != nil {
			t.Fatalf("Save() error = %v", err)
		}
		
		loaded, _ = store.Load()
		if loaded.AccessToken != newCreds.AccessToken {
			t.Errorf("After Save(), token = %v, want %v", loaded.AccessToken, newCreds.AccessToken)
		}
		
		// Test Delete
		err = store.Delete()
		if err != nil {
			t.Fatalf("Delete() error = %v", err)
		}
		
		if store.Exists() {
			t.Error("After Delete(), Exists() = true, want false")
		}
	})
	
	// Test error handling
	t.Run("error handling", func(t *testing.T) {
		expectedErr := errTest
		store := NewMockStore(nil, expectedErr)
		
		// Load should return error
		_, err := store.Load()
		if err != expectedErr {
			t.Errorf("Load() error = %v, want %v", err, expectedErr)
		}
		
		// Save should return error
		err = store.Save(&Credentials{})
		if err != expectedErr {
			t.Errorf("Save() error = %v, want %v", err, expectedErr)
		}
		
		// Delete should return error
		err = store.Delete()
		if err != expectedErr {
			t.Errorf("Delete() error = %v, want %v", err, expectedErr)
		}
	})
}

// Test error for mock tests
var errTest = &testError{"test error"}

type testError struct {
	msg string
}

func (e *testError) Error() string {
	return e.msg
}