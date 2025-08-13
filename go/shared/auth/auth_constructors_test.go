package auth

import (
	"context"
	"net/http"
	"testing"
	"time"
)

func TestNewManagerWithProvider(t *testing.T) {
	tests := []struct {
		name   string
		config *LoginConfig
	}{
		{
			name: "with config",
			config: &LoginConfig{
				AuthKitDomain: "custom.auth",
				ClientID:      "custom-client",
				NoBrowser:     true,
			},
		},
		{
			name:   "with nil config uses defaults",
			config: nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			store := NewMockStore(nil, nil)
			provider := &MockOAuthProvider{}
			
			manager := NewManagerWithProvider(store, provider, tt.config)
			
			if manager == nil {
				t.Fatal("NewManagerWithProvider returned nil")
			}
			
			if manager.store != store {
				t.Error("Store not set correctly")
			}
			
			if manager.oauthProvider != provider {
				t.Error("OAuth provider not set correctly")
			}
			
			if manager.browserOpener == nil {
				t.Error("Browser opener not set")
			}
			
			if tt.config == nil {
				if manager.config.AuthKitDomain != DefaultAuthKitDomain {
					t.Errorf("Default AuthKitDomain = %v, want %v", 
						manager.config.AuthKitDomain, DefaultAuthKitDomain)
				}
				if manager.config.ClientID != DefaultClientID {
					t.Errorf("Default ClientID = %v, want %v",
						manager.config.ClientID, DefaultClientID)
				}
			}
		})
	}
}

func TestNewManagerWithMocks(t *testing.T) {
	tests := []struct {
		name   string
		config *LoginConfig
	}{
		{
			name: "with config",
			config: &LoginConfig{
				AuthKitDomain: "test.auth",
				ClientID:      "test-client",
				NoBrowser:     false, // Will be overridden to true
			},
		},
		{
			name:   "with nil config",
			config: nil,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			store := NewMockStore(nil, nil)
			provider := &MockOAuthProvider{}
			browser := &MockBrowserOpener{}
			
			manager := NewManagerWithMocks(store, provider, browser, tt.config)
			
			if manager == nil {
				t.Fatal("NewManagerWithMocks returned nil")
			}
			
			// Should always have NoBrowser set to true for tests
			if !manager.config.NoBrowser {
				t.Error("NoBrowser should be true for test manager")
			}
			
			if manager.browserOpener != browser {
				t.Error("Browser opener not set correctly")
			}
		})
	}
}

func TestDefaultBrowserOpener_OpenURL(t *testing.T) {
	// DO NOT actually call OpenURL on the real browser opener in tests!
	// Just verify the type exists and implements the interface
	var _ BrowserOpener = &defaultBrowserOpener{}
	
	// That's all we need - no actual browser opening in tests!
}

func TestMockReset_Functions(t *testing.T) {
	t.Run("MockOAuthProvider.Reset", func(t *testing.T) {
		mock := &MockOAuthProvider{}
		
		// Add some calls
		mock.StartDeviceFlowCalls = append(mock.StartDeviceFlowCalls, struct {
			Ctx context.Context
		}{})
		mock.PollForTokenCalls = append(mock.PollForTokenCalls, struct {
			Ctx        context.Context
			DeviceCode string
			Interval   time.Duration
		}{})
		
		// Reset should clear them
		mock.Reset()
		
		if len(mock.StartDeviceFlowCalls) != 0 {
			t.Error("StartDeviceFlowCalls not cleared")
		}
		if len(mock.PollForTokenCalls) != 0 {
			t.Error("PollForTokenCalls not cleared")
		}
	})
	
	t.Run("MockHTTPClient.Reset", func(t *testing.T) {
		mock := &MockHTTPClient{}
		
		// Add some calls
		mock.DoCalls = append(mock.DoCalls, struct {
			Req *http.Request
		}{})
		
		// Reset should clear them
		mock.Reset()
		
		if len(mock.DoCalls) != 0 {
			t.Error("DoCalls not cleared")
		}
	})
	
	t.Run("MockBrowserOpener.Reset", func(t *testing.T) {
		mock := &MockBrowserOpener{}
		
		// Add some calls
		mock.OpenURLCalls = append(mock.OpenURLCalls, struct {
			URL string
		}{URL: "test"})
		
		// Reset should clear them
		mock.Reset()
		
		if len(mock.OpenURLCalls) != 0 {
			t.Error("OpenURLCalls not cleared")
		}
	})
}