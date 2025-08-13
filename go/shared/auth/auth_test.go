package auth

import (
	"context"
	"testing"
	"time"
)

func TestManager_Status(t *testing.T) {
	tests := []struct {
		name      string
		creds     *Credentials
		storeErr  error
		wantLogin bool
		wantRefresh bool
	}{
		{
			name:      "not logged in",
			creds:     nil,
			storeErr:  nil,
			wantLogin: false,
		},
		{
			name: "logged in with valid token",
			creds: &Credentials{
				AuthKitDomain: "test.auth",
				AccessToken:   "valid-token",
				ExpiresAt:     timePtr(time.Now().Add(time.Hour)),
			},
			wantLogin: true,
			wantRefresh: false,
		},
		{
			name: "logged in with expired token",
			creds: &Credentials{
				AuthKitDomain: "test.auth",
				AccessToken:   "expired-token",
				RefreshToken:  "refresh-token",
				ExpiresAt:     timePtr(time.Now().Add(-time.Hour)),
			},
			wantLogin: true,
			wantRefresh: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			store := NewMockStore(tt.creds, tt.storeErr)
			manager := NewManager(store, nil)
			
			status := manager.Status()
			
			if status.LoggedIn != tt.wantLogin {
				t.Errorf("Status.LoggedIn = %v, want %v", status.LoggedIn, tt.wantLogin)
			}
			
			if status.NeedsRefresh != tt.wantRefresh {
				t.Errorf("Status.NeedsRefresh = %v, want %v", status.NeedsRefresh, tt.wantRefresh)
			}
		})
	}
}

func TestManager_Logout(t *testing.T) {
	store := NewMockStore(&Credentials{
		AccessToken: "test-token",
	}, nil)
	
	manager := NewManager(store, nil)
	
	// Should have credentials initially
	if !store.Exists() {
		t.Error("Expected credentials to exist before logout")
	}
	
	// Logout
	err := manager.Logout()
	if err != nil {
		t.Errorf("Logout() error = %v", err)
	}
	
	// Should not have credentials after logout
	if store.Exists() {
		t.Error("Expected credentials to be deleted after logout")
	}
}

func TestManager_GetToken(t *testing.T) {
	tests := []struct {
		name    string
		creds   *Credentials
		wantErr bool
	}{
		{
			name:    "no credentials",
			creds:   nil,
			wantErr: true,
		},
		{
			name: "valid token",
			creds: &Credentials{
				AccessToken: "valid-token",
				ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
			},
			wantErr: false,
		},
		{
			name: "expired token without refresh",
			creds: &Credentials{
				AccessToken: "expired-token",
				ExpiresAt:   timePtr(time.Now().Add(-time.Hour)),
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			store := NewMockStore(tt.creds, nil)
			manager := NewManager(store, nil)
			
			token, err := manager.GetToken(context.Background())
			
			if (err != nil) != tt.wantErr {
				t.Errorf("GetToken() error = %v, wantErr %v", err, tt.wantErr)
			}
			
			if !tt.wantErr && token != tt.creds.AccessToken {
				t.Errorf("GetToken() = %v, want %v", token, tt.creds.AccessToken)
			}
		})
	}
}