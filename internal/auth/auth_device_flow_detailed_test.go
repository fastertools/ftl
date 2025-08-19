package auth

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"
)

func TestManager_StartDeviceFlow_DetailedCoverage(t *testing.T) {
	h := NewTestHelpers()

	t.Run("with existing expired credentials and refresh token", func(t *testing.T) {
		// This tests the code path where we have expired credentials with refresh token
		expiredCreds := h.ExpiredCredentials()
		builder := NewMockBuilder().
			WithStoredCredentials(expiredCreds).
			WithRefreshToken(h.TokenResponse(), nil). // Successful refresh
			WithDeviceFlow(h.DeviceAuthResponse(), nil)

		manager, provider, _ := builder.Build()

		// Should error because token was refreshed successfully
		_, err := manager.StartDeviceFlow(context.Background())
		if err == nil {
			t.Error("StartDeviceFlow() should error when token is refreshed")
		}

		// Verify device flow was NOT started (refresh succeeded)
		if len(provider.StartDeviceFlowCalls) != 0 {
			t.Errorf("StartDeviceFlow called %d times, want 0", len(provider.StartDeviceFlowCalls))
		}
	})

	t.Run("with expired credentials and no refresh token", func(t *testing.T) {
		// Test expired credentials without refresh token
		expiredCreds := h.ExpiredCredentials()
		expiredCreds.RefreshToken = "" // Remove refresh token

		builder := NewMockBuilder().
			WithStoredCredentials(expiredCreds).
			WithDeviceFlow(h.DeviceAuthResponse(), nil)

		manager, provider, _ := builder.Build()

		// Should proceed with device flow
		resp, err := manager.StartDeviceFlow(context.Background())
		if err != nil {
			t.Errorf("StartDeviceFlow() error = %v", err)
		}

		if resp == nil {
			t.Error("StartDeviceFlow() returned nil response")
		}

		// Verify device flow was started
		if len(provider.StartDeviceFlowCalls) != 1 {
			t.Errorf("StartDeviceFlow called %d times, want 1", len(provider.StartDeviceFlowCalls))
		}
	})

	t.Run("browser opening when not disabled", func(t *testing.T) {
		store := NewMockStore(nil, nil)
		provider := &MockOAuthProvider{}
		browser := &MockBrowserOpener{}

		// Create manager with browser enabled
		config := &LoginConfig{
			NoBrowser: false,
		}
		manager := NewManagerWithMocks(store, provider, browser, config)

		// Override config to enable browser
		manager.config.NoBrowser = false

		provider.StartDeviceFlowFunc = func(ctx context.Context) (*DeviceAuthResponse, error) {
			return h.DeviceAuthResponse(), nil
		}

		resp, err := manager.StartDeviceFlow(context.Background())
		if err != nil {
			t.Errorf("StartDeviceFlow() error = %v", err)
		}

		if resp == nil {
			t.Fatal("StartDeviceFlow() returned nil")
		}

		// Browser should have been called
		if len(browser.OpenURLCalls) != 1 {
			t.Errorf("Browser OpenURL called %d times, want 1", len(browser.OpenURLCalls))
		}

		if len(browser.OpenURLCalls) > 0 && browser.OpenURLCalls[0].URL != resp.VerificationURIComplete {
			t.Errorf("Browser opened %v, want %v",
				browser.OpenURLCalls[0].URL, resp.VerificationURIComplete)
		}
	})

	t.Run("store load error is logged but not fatal", func(t *testing.T) {
		store := NewMockStore(nil, errors.New("keyring error"))
		provider := &MockOAuthProvider{
			StartDeviceFlowFunc: func(ctx context.Context) (*DeviceAuthResponse, error) {
				return h.DeviceAuthResponse(), nil
			},
		}
		browser := &MockBrowserOpener{}

		manager := NewManagerWithMocks(store, provider, browser, nil)

		// Should still work despite store error
		resp, err := manager.StartDeviceFlow(context.Background())
		if err != nil {
			t.Errorf("StartDeviceFlow() error = %v", err)
		}

		if resp == nil {
			t.Error("StartDeviceFlow() returned nil")
		}
	})
}

func TestManager_CompleteDeviceFlow_EdgeCases(t *testing.T) {
	h := NewTestHelpers()

	t.Run("handles slow_down response", func(t *testing.T) {
		builder := NewMockBuilder().
			WithTokenPolling([]interface{}{
				h.SlowDownError(),
				h.TokenResponse(),
			})

		manager, provider, _ := builder.Build()

		creds, err := manager.CompleteDeviceFlow(context.Background(), h.DeviceAuthResponse())
		if err != nil {
			t.Errorf("CompleteDeviceFlow() error = %v", err)
		}

		if creds == nil {
			t.Error("CompleteDeviceFlow() returned nil credentials")
		}

		// Should have called poll once (mock handles retries internally)
		if len(provider.PollForTokenCalls) != 1 {
			t.Errorf("PollForToken called %d times, want 1", len(provider.PollForTokenCalls))
		}
	})

	t.Run("handles context cancellation in mock", func(t *testing.T) {
		builder := NewMockBuilder()

		// Configure polling to check context
		builder.provider.PollForTokenFunc = func(ctx context.Context, deviceCode string, interval time.Duration) (*TokenResponse, error) {
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(100 * time.Millisecond):
				return h.TokenResponse(), nil
			}
		}

		manager, _, _ := builder.Build()

		ctx, cancel := context.WithCancel(context.Background())
		cancel() // Cancel immediately

		_, err := manager.CompleteDeviceFlow(ctx, h.DeviceAuthResponse())
		if err == nil {
			t.Error("CompleteDeviceFlow() should error on cancelled context")
		}
	})
}

func TestWithTokenPolling_EdgeCases(t *testing.T) {
	h := NewTestHelpers()

	t.Run("handles mixed response types", func(t *testing.T) {
		builder := NewMockBuilder().
			WithTokenPolling([]interface{}{
				h.AuthorizationPendingError(),
				h.SlowDownError(),
				errors.New("network error"),
			})

		manager, _, _ := builder.Build()

		_, err := manager.CompleteDeviceFlow(context.Background(), h.DeviceAuthResponse())
		if err == nil {
			t.Error("Expected error from network error in polling")
		}
	})

	t.Run("handles unexpected type in responses", func(t *testing.T) {
		builder := NewMockBuilder()

		builder.provider.PollForTokenFunc = func(ctx context.Context, deviceCode string, interval time.Duration) (*TokenResponse, error) {
			// Return through the response handler with unexpected type
			responses := []interface{}{
				"unexpected string type",
			}

			for _, resp := range responses {
				switch v := resp.(type) {
				case *TokenResponse:
					return v, nil
				case *TokenError:
					return nil, v
				case error:
					return nil, v
				default:
					return nil, fmt.Errorf("unexpected response type: %T", v)
				}
			}
			return nil, errors.New("no responses")
		}

		manager, _, _ := builder.Build()

		_, err := manager.CompleteDeviceFlow(context.Background(), h.DeviceAuthResponse())
		if err == nil {
			t.Error("Expected error from unexpected type")
		}
		if !strings.Contains(err.Error(), "unexpected response type") {
			t.Errorf("Error should mention unexpected type, got: %v", err)
		}
	})
}
