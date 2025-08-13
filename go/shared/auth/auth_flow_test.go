package auth

import (
	"context"
	"errors"
	"testing"
	"time"
)

func TestManager_StartDeviceFlow_WithMocks(t *testing.T) {
	h := NewTestHelpers()

	tests := []struct {
		name          string
		setupMocks    func() (*Manager, *MockOAuthProvider, *MockStore)
		wantErr       bool
		checkResult   func(t *testing.T, resp *DeviceAuthResponse, provider *MockOAuthProvider)
	}{
		{
			name: "successful device flow start",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithDeviceFlow(h.DeviceAuthResponse(), nil).
					Build()
			},
			wantErr: false,
			checkResult: func(t *testing.T, resp *DeviceAuthResponse, provider *MockOAuthProvider) {
				if resp.DeviceCode != "device-123" {
					t.Errorf("DeviceCode = %v, want device-123", resp.DeviceCode)
				}
				if len(provider.StartDeviceFlowCalls) != 1 {
					t.Errorf("StartDeviceFlow called %d times, want 1", len(provider.StartDeviceFlowCalls))
				}
			},
		},
		{
			name: "device flow blocked when already logged in with valid credentials",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithStoredCredentials(h.ValidCredentials()).
					WithDeviceFlow(h.DeviceAuthResponse(), nil).
					Build()
			},
			wantErr: true, // Should error because already logged in
			checkResult: func(t *testing.T, resp *DeviceAuthResponse, provider *MockOAuthProvider) {
				// Should NOT start device flow when already logged in
				if len(provider.StartDeviceFlowCalls) != 0 {
					t.Errorf("StartDeviceFlow called %d times, want 0", len(provider.StartDeviceFlowCalls))
				}
			},
		},
		{
			name: "device flow error",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithDeviceFlow(nil, errors.New("network error")).
					Build()
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			manager, provider, _ := tt.setupMocks()

			resp, err := manager.StartDeviceFlow(context.Background())

			if (err != nil) != tt.wantErr {
				t.Errorf("StartDeviceFlow() error = %v, wantErr %v", err, tt.wantErr)
			}

			if tt.checkResult != nil && !tt.wantErr {
				tt.checkResult(t, resp, provider)
			}
		})
	}
}

func TestManager_CompleteDeviceFlow_WithMocks(t *testing.T) {
	h := NewTestHelpers()

	tests := []struct {
		name        string
		deviceAuth  *DeviceAuthResponse
		setupMocks  func() (*Manager, *MockOAuthProvider, *MockStore)
		wantErr     bool
		checkResult func(t *testing.T, creds *Credentials, provider *MockOAuthProvider, store *MockStore)
	}{
		{
			name:       "immediate success",
			deviceAuth: h.DeviceAuthResponse(),
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithTokenPolling([]interface{}{h.TokenResponse()}).
					Build()
			},
			wantErr: false,
			checkResult: func(t *testing.T, creds *Credentials, provider *MockOAuthProvider, store *MockStore) {
				if creds.AccessToken != "access-token" {
					t.Errorf("AccessToken = %v, want access-token", creds.AccessToken)
				}
				if len(provider.PollForTokenCalls) != 1 {
					t.Errorf("PollForToken called %d times, want 1", len(provider.PollForTokenCalls))
				}
				// Check credentials were saved
				saved, _ := store.Load()
				if saved == nil || saved.AccessToken != "access-token" {
					t.Error("Credentials were not saved correctly")
				}
			},
		},
		{
			name:       "polling with retries",
			deviceAuth: h.DeviceAuthResponse(),
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithTokenPolling([]interface{}{
						h.AuthorizationPendingError(),
						h.AuthorizationPendingError(),
						h.TokenResponse(),
					}).
					Build()
			},
			wantErr: false,
			checkResult: func(t *testing.T, creds *Credentials, provider *MockOAuthProvider, store *MockStore) {
				if creds.AccessToken != "access-token" {
					t.Errorf("AccessToken = %v, want access-token", creds.AccessToken)
				}
				// Our mock simulates the entire polling loop internally,
				// so PollForToken is only called once
				if len(provider.PollForTokenCalls) != 1 {
					t.Errorf("PollForToken called %d times, want 1", len(provider.PollForTokenCalls))
				}
			},
		},
		{
			name:       "expired device code",
			deviceAuth: h.DeviceAuthResponse(),
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithTokenPolling([]interface{}{h.ExpiredTokenError()}).
					Build()
			},
			wantErr: true,
		},
		{
			name:       "access denied",
			deviceAuth: h.DeviceAuthResponse(),
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithTokenPolling([]interface{}{h.AccessDeniedError()}).
					Build()
			},
			wantErr: true,
		},
		{
			name:       "save error",
			deviceAuth: h.DeviceAuthResponse(),
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				builder := NewMockBuilder().
					WithTokenPolling([]interface{}{h.TokenResponse()}).
					WithStoreError(errors.New("keyring error"))
				manager, provider, store := builder.Build()
				// Override with a store that fails on save
				store.err = errors.New("keyring error")
				return manager, provider, store
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			manager, provider, store := tt.setupMocks()

			creds, err := manager.CompleteDeviceFlow(context.Background(), tt.deviceAuth)

			if (err != nil) != tt.wantErr {
				t.Errorf("CompleteDeviceFlow() error = %v, wantErr %v", err, tt.wantErr)
			}

			if tt.checkResult != nil && !tt.wantErr {
				tt.checkResult(t, creds, provider, store)
			}
		})
	}
}

func TestManager_Login_FullFlow(t *testing.T) {
	h := NewTestHelpers()

	tests := []struct {
		name        string
		setupMocks  func() (*Manager, *MockOAuthProvider, *MockStore)
		wantErr     bool
		checkResult func(t *testing.T, creds *Credentials, provider *MockOAuthProvider)
	}{
		{
			name: "successful login flow",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithDeviceFlow(h.DeviceAuthResponse(), nil).
					WithTokenPolling([]interface{}{
						h.AuthorizationPendingError(),
						h.TokenResponse(),
					}).
					Build()
			},
			wantErr: false,
			checkResult: func(t *testing.T, creds *Credentials, provider *MockOAuthProvider) {
				if creds.AccessToken != "access-token" {
					t.Errorf("AccessToken = %v, want access-token", creds.AccessToken)
				}
				if len(provider.StartDeviceFlowCalls) != 1 {
					t.Errorf("StartDeviceFlow called %d times, want 1", len(provider.StartDeviceFlowCalls))
				}
				// PollForToken is called once, internally handling retries
				if len(provider.PollForTokenCalls) != 1 {
					t.Errorf("PollForToken called %d times, want 1", len(provider.PollForTokenCalls))
				}
			},
		},
		{
			name: "device flow fails",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithDeviceFlow(nil, errors.New("network error")).
					Build()
			},
			wantErr: true,
		},
		{
			name: "polling fails",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithDeviceFlow(h.DeviceAuthResponse(), nil).
					WithTokenPolling([]interface{}{
						errors.New("polling error"),
					}).
					Build()
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			manager, provider, _ := tt.setupMocks()

			creds, err := manager.Login(context.Background())

			if (err != nil) != tt.wantErr {
				t.Errorf("Login() error = %v, wantErr %v", err, tt.wantErr)
			}

			if tt.checkResult != nil && !tt.wantErr {
				tt.checkResult(t, creds, provider)
			}
		})
	}
}

func TestManager_Refresh_WithMocks(t *testing.T) {
	h := NewTestHelpers()

	tests := []struct {
		name        string
		creds       *Credentials
		setupMocks  func() (*Manager, *MockOAuthProvider, *MockStore)
		wantErr     bool
		checkResult func(t *testing.T, updated *Credentials, provider *MockOAuthProvider)
	}{
		{
			name:  "successful refresh",
			creds: h.ExpiredCredentials(),
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithRefreshToken(h.TokenResponse(), nil).
					Build()
			},
			wantErr: false,
			checkResult: func(t *testing.T, updated *Credentials, provider *MockOAuthProvider) {
				if updated.AccessToken != "access-token" {
					t.Errorf("AccessToken = %v, want access-token", updated.AccessToken)
				}
				if len(provider.RefreshTokenCalls) != 1 {
					t.Errorf("RefreshToken called %d times, want 1", len(provider.RefreshTokenCalls))
				}
				if provider.RefreshTokenCalls[0].RefreshToken != "refresh-token" {
					t.Errorf("RefreshToken called with %v, want refresh-token",
						provider.RefreshTokenCalls[0].RefreshToken)
				}
			},
		},
		{
			name: "no refresh token",
			creds: &Credentials{
				AccessToken: "expired-token",
				ExpiresAt:   timePtr(time.Now().Add(-time.Hour)),
			},
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().Build()
			},
			wantErr: true,
		},
		{
			name:  "refresh fails",
			creds: h.ExpiredCredentials(),
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithRefreshToken(nil, errors.New("invalid refresh token")).
					Build()
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			manager, provider, _ := tt.setupMocks()

			updated, err := manager.Refresh(context.Background(), tt.creds)

			if (err != nil) != tt.wantErr {
				t.Errorf("Refresh() error = %v, wantErr %v", err, tt.wantErr)
			}

			if tt.checkResult != nil && !tt.wantErr {
				tt.checkResult(t, updated, provider)
			}
		})
	}
}

func TestManager_GetToken_WithRefresh(t *testing.T) {
	h := NewTestHelpers()

	tests := []struct {
		name        string
		setupMocks  func() (*Manager, *MockOAuthProvider, *MockStore)
		wantErr     bool
		wantToken   string
		checkResult func(t *testing.T, provider *MockOAuthProvider)
	}{
		{
			name: "valid token",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithStoredCredentials(h.ValidCredentials()).
					Build()
			},
			wantErr:   false,
			wantToken: "valid-token",
			checkResult: func(t *testing.T, provider *MockOAuthProvider) {
				// Should not refresh
				if len(provider.RefreshTokenCalls) != 0 {
					t.Errorf("RefreshToken called %d times, want 0", len(provider.RefreshTokenCalls))
				}
			},
		},
		{
			name: "expired token with refresh",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithStoredCredentials(h.ExpiredCredentials()).
					WithRefreshToken(&TokenResponse{
						AccessToken:  "new-access-token",
						RefreshToken: "new-refresh-token",
						ExpiresIn:    3600,
					}, nil).
					Build()
			},
			wantErr:   false,
			wantToken: "new-access-token",
			checkResult: func(t *testing.T, provider *MockOAuthProvider) {
				// Should refresh once
				if len(provider.RefreshTokenCalls) != 1 {
					t.Errorf("RefreshToken called %d times, want 1", len(provider.RefreshTokenCalls))
				}
			},
		},
		{
			name: "expired token no refresh token",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				creds := h.ExpiredCredentials()
				creds.RefreshToken = ""
				return NewMockBuilder().
					WithStoredCredentials(creds).
					Build()
			},
			wantErr: true,
		},
		{
			name: "not logged in",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().Build()
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			manager, provider, _ := tt.setupMocks()

			token, err := manager.GetToken(context.Background())

			if (err != nil) != tt.wantErr {
				t.Errorf("GetToken() error = %v, wantErr %v", err, tt.wantErr)
			}

			if !tt.wantErr && token != tt.wantToken {
				t.Errorf("GetToken() = %v, want %v", token, tt.wantToken)
			}

			if tt.checkResult != nil {
				tt.checkResult(t, provider)
			}
		})
	}
}

func TestManager_GetOrRefreshToken(t *testing.T) {
	h := NewTestHelpers()

	tests := []struct {
		name      string
		setupMocks func() (*Manager, *MockOAuthProvider, *MockStore)
		wantErr   bool
		wantToken string
	}{
		{
			name: "valid token",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithStoredCredentials(h.ValidCredentials()).
					Build()
			},
			wantErr:   false,
			wantToken: "valid-token",
		},
		{
			name: "expired token with refresh",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().
					WithStoredCredentials(h.ExpiredCredentials()).
					WithRefreshToken(&TokenResponse{
						AccessToken: "refreshed-token",
						ExpiresIn:   3600,
					}, nil).
					Build()
			},
			wantErr:   false,
			wantToken: "refreshed-token",
		},
		{
			name: "not logged in",
			setupMocks: func() (*Manager, *MockOAuthProvider, *MockStore) {
				return NewMockBuilder().Build()
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			manager, _, _ := tt.setupMocks()

			token, err := manager.GetOrRefreshToken(context.Background())

			if (err != nil) != tt.wantErr {
				t.Errorf("GetOrRefreshToken() error = %v, wantErr %v", err, tt.wantErr)
			}

			if !tt.wantErr && token != tt.wantToken {
				t.Errorf("GetOrRefreshToken() = %v, want %v", token, tt.wantToken)
			}
		})
	}
}