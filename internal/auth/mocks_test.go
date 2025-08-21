package auth

import (
	"context"
	"net/http"
	"testing"
	"time"
)

func TestMockBrowserOpener_OpenURL(t *testing.T) {
	mock := &MockBrowserOpener{}

	// Test without custom function
	err := mock.OpenURL("https://example.com")
	if err != nil {
		t.Errorf("OpenURL() error = %v, want nil", err)
	}

	if len(mock.OpenURLCalls) != 1 {
		t.Errorf("OpenURLCalls length = %d, want 1", len(mock.OpenURLCalls))
	}

	if mock.OpenURLCalls[0].URL != "https://example.com" {
		t.Errorf("URL = %v, want https://example.com", mock.OpenURLCalls[0].URL)
	}

	// Test with custom function
	mock.OpenURLFunc = func(url string) error {
		return nil
	}

	err = mock.OpenURL("https://test.com")
	if err != nil {
		t.Errorf("OpenURL() with custom func error = %v", err)
	}

	if len(mock.OpenURLCalls) != 2 {
		t.Errorf("OpenURLCalls length = %d, want 2", len(mock.OpenURLCalls))
	}
}

func TestTestHelpers_SlowDownError(t *testing.T) {
	h := NewTestHelpers()
	err := h.SlowDownError()

	if err.ErrorCode != "slow_down" {
		t.Errorf("ErrorCode = %v, want slow_down", err.ErrorCode)
	}

	if !err.IsSlowDown() {
		t.Error("IsSlowDown() = false, want true")
	}
}

func TestMockHTTPClient_Do(t *testing.T) {
	mock := &MockHTTPClient{}

	req, _ := http.NewRequest("GET", "https://example.com", nil)

	// Test without custom function (should error)
	_, err := mock.Do(req)
	if err == nil {
		t.Error("Do() without DoFunc should error")
	}

	if len(mock.DoCalls) != 1 {
		t.Errorf("DoCalls length = %d, want 1", len(mock.DoCalls))
	}

	// Test with custom function
	mock.DoFunc = func(r *http.Request) (*http.Response, error) {
		return &http.Response{
			StatusCode: 200,
		}, nil
	}

	resp, err := mock.Do(req)
	if err != nil {
		t.Errorf("Do() with DoFunc error = %v", err)
	}

	if resp.StatusCode != 200 {
		t.Errorf("StatusCode = %d, want 200", resp.StatusCode)
	}
}

func TestFileKeyringPrompt(t *testing.T) {
	// Test the file keyring prompt function
	password1, err := fileKeyringPrompt("test prompt 1")
	if err != nil {
		t.Errorf("fileKeyringPrompt() error = %v", err)
	}

	if password1 == "" {
		t.Error("fileKeyringPrompt() returned empty password")
	}

	// Should return consistent password
	password2, err := fileKeyringPrompt("test prompt 2")
	if err != nil {
		t.Errorf("fileKeyringPrompt() error = %v", err)
	}

	if password1 != password2 {
		t.Error("fileKeyringPrompt() should return consistent password")
	}
}

func TestMockOAuthProvider_DefaultBehavior(t *testing.T) {
	mock := &MockOAuthProvider{}

	// Test StartDeviceFlow default behavior
	resp, err := mock.StartDeviceFlow(context.Background())
	if err != nil {
		t.Errorf("StartDeviceFlow() error = %v", err)
	}

	if resp.DeviceCode != "mock-device-code" {
		t.Errorf("DeviceCode = %v, want mock-device-code", resp.DeviceCode)
	}

	// Test PollForToken default behavior
	token, err := mock.PollForToken(context.Background(), "device", 5*time.Second)
	if err != nil {
		t.Errorf("PollForToken() error = %v", err)
	}

	if token.AccessToken != "mock-access-token" {
		t.Errorf("AccessToken = %v, want mock-access-token", token.AccessToken)
	}

	// Test RefreshToken default behavior
	refreshed, err := mock.RefreshToken(context.Background(), "refresh")
	if err != nil {
		t.Errorf("RefreshToken() error = %v", err)
	}

	if refreshed.AccessToken != "mock-refreshed-token" {
		t.Errorf("AccessToken = %v, want mock-refreshed-token", refreshed.AccessToken)
	}
}
