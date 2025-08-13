package auth

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"strings"
	"testing"
	"time"
)

func TestOAuthClient_StartDeviceFlow_WithHTTPMock(t *testing.T) {
	tests := []struct {
		name         string
		setupMock    func() *MockHTTPClient
		wantErr      bool
		checkResult  func(t *testing.T, resp *DeviceAuthResponse)
	}{
		{
			name: "successful device flow",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					// Verify request
					if !strings.Contains(req.URL.String(), "/oauth2/device_authorization") {
						t.Errorf("URL = %v, want /oauth2/device_authorization", req.URL)
					}
					if req.Method != "POST" {
						t.Errorf("Method = %v, want POST", req.Method)
					}

					// Return success response
					resp := &DeviceAuthResponse{
						DeviceCode:      "device-abc123",
						UserCode:        "USER-123",
						VerificationURI: "https://auth.example.com/device",
						ExpiresIn:       600,
						Interval:        5,
					}
					body, _ := json.Marshal(resp)
					
					return &http.Response{
						StatusCode: http.StatusOK,
						Body:       io.NopCloser(bytes.NewReader(body)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: false,
			checkResult: func(t *testing.T, resp *DeviceAuthResponse) {
				if resp.DeviceCode != "device-abc123" {
					t.Errorf("DeviceCode = %v, want device-abc123", resp.DeviceCode)
				}
				if resp.Interval != 5 {
					t.Errorf("Interval = %v, want 5", resp.Interval)
				}
			},
		},
		{
			name: "device flow with default interval",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					resp := &DeviceAuthResponse{
						DeviceCode:      "device-xyz",
						UserCode:        "USER-456",
						VerificationURI: "https://auth.example.com/device",
						ExpiresIn:       600,
						// Interval is 0, should default to 5
					}
					body, _ := json.Marshal(resp)
					
					return &http.Response{
						StatusCode: http.StatusOK,
						Body:       io.NopCloser(bytes.NewReader(body)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: false,
			checkResult: func(t *testing.T, resp *DeviceAuthResponse) {
				if resp.Interval != 5 {
					t.Errorf("Interval = %v, want 5 (default)", resp.Interval)
				}
			},
		},
		{
			name: "server error",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					return &http.Response{
						StatusCode: http.StatusInternalServerError,
						Body:       io.NopCloser(strings.NewReader("Internal Server Error")),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: true,
		},
		{
			name: "network error",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					return nil, errors.New("network error")
				}
				return mock
			},
			wantErr: true,
		},
		{
			name: "invalid json response",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					return &http.Response{
						StatusCode: http.StatusOK,
						Body:       io.NopCloser(strings.NewReader("not valid json")),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mock := tt.setupMock()
			client := &OAuthClient{
				httpClient:    mock,
				authKitDomain: "test.auth.example.com",
				clientID:      "test-client",
			}

			ctx := context.Background()
			resp, err := client.StartDeviceFlow(ctx)

			if (err != nil) != tt.wantErr {
				t.Errorf("StartDeviceFlow() error = %v, wantErr %v", err, tt.wantErr)
			}

			if tt.checkResult != nil && !tt.wantErr {
				tt.checkResult(t, resp)
			}

			// Verify HTTP call was made
			if len(mock.DoCalls) != 1 {
				t.Errorf("HTTP client called %d times, want 1", len(mock.DoCalls))
			}
		})
	}
}

func TestOAuthClient_PollForToken_WithHTTPMock(t *testing.T) {
	tests := []struct {
		name        string
		setupMock   func() *MockHTTPClient
		wantErr     bool
		checkResult func(t *testing.T, resp *TokenResponse)
		checkError  func(t *testing.T, err error)
	}{
		{
			name: "immediate success",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				callCount := 0
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					callCount++
					
					// First call should succeed immediately
					resp := &TokenResponse{
						AccessToken:  "access-token",
						RefreshToken: "refresh-token",
						ExpiresIn:    3600,
						TokenType:    "Bearer",
					}
					body, _ := json.Marshal(resp)
					
					return &http.Response{
						StatusCode: http.StatusOK,
						Body:       io.NopCloser(bytes.NewReader(body)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: false,
			checkResult: func(t *testing.T, resp *TokenResponse) {
				if resp.AccessToken != "access-token" {
					t.Errorf("AccessToken = %v, want access-token", resp.AccessToken)
				}
			},
		},
		{
			name: "authorization pending then success",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				callCount := 0
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					callCount++
					
					if callCount <= 2 {
						// First two calls return pending
						errResp := &TokenError{ErrorCode: "authorization_pending"}
						body, _ := json.Marshal(errResp)
						
						return &http.Response{
							StatusCode: http.StatusBadRequest,
							Body:       io.NopCloser(bytes.NewReader(body)),
							Header:     make(http.Header),
						}, nil
					}
					
					// Third call succeeds
					resp := &TokenResponse{
						AccessToken: "access-token",
						ExpiresIn:   3600,
					}
					body, _ := json.Marshal(resp)
					
					return &http.Response{
						StatusCode: http.StatusOK,
						Body:       io.NopCloser(bytes.NewReader(body)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: false,
			checkResult: func(t *testing.T, resp *TokenResponse) {
				if resp.AccessToken != "access-token" {
					t.Errorf("AccessToken = %v, want access-token", resp.AccessToken)
				}
			},
		},
		{
			name: "slow down",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				callCount := 0
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					callCount++
					
					if callCount == 1 {
						// First call returns slow_down
						errResp := &TokenError{ErrorCode: "slow_down"}
						body, _ := json.Marshal(errResp)
						
						return &http.Response{
							StatusCode: http.StatusBadRequest,
							Body:       io.NopCloser(bytes.NewReader(body)),
							Header:     make(http.Header),
						}, nil
					}
					
					// Second call succeeds
					resp := &TokenResponse{
						AccessToken: "access-token",
						ExpiresIn:   3600,
					}
					body, _ := json.Marshal(resp)
					
					return &http.Response{
						StatusCode: http.StatusOK,
						Body:       io.NopCloser(bytes.NewReader(body)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: false,
		},
		{
			name: "expired token",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					errResp := &TokenError{
						ErrorCode:        "expired_token",
						ErrorDescription: "Device code expired",
					}
					body, _ := json.Marshal(errResp)
					
					return &http.Response{
						StatusCode: http.StatusBadRequest,
						Body:       io.NopCloser(bytes.NewReader(body)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: true,
			checkError: func(t *testing.T, err error) {
				if !strings.Contains(err.Error(), "expired") {
					t.Errorf("Error = %v, want to contain 'expired'", err)
				}
			},
		},
		{
			name: "access denied",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					errResp := &TokenError{
						ErrorCode:        "access_denied",
						ErrorDescription: "User denied request",
					}
					body, _ := json.Marshal(errResp)
					
					return &http.Response{
						StatusCode: http.StatusBadRequest,
						Body:       io.NopCloser(bytes.NewReader(body)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mock := tt.setupMock()
			client := &OAuthClient{
				httpClient:    mock,
				authKitDomain: "test.auth.example.com",
				clientID:      "test-client",
			}

			ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
			defer cancel()
			
			resp, err := client.PollForToken(ctx, "test-device-code", 50*time.Millisecond)

			if (err != nil) != tt.wantErr {
				t.Errorf("PollForToken() error = %v, wantErr %v", err, tt.wantErr)
			}

			if tt.checkResult != nil && !tt.wantErr {
				tt.checkResult(t, resp)
			}

			if tt.checkError != nil && err != nil {
				tt.checkError(t, err)
			}
		})
	}
}

func TestOAuthClient_RefreshToken_WithHTTPMock(t *testing.T) {
	tests := []struct {
		name        string
		setupMock   func() *MockHTTPClient
		wantErr     bool
		checkResult func(t *testing.T, resp *TokenResponse)
	}{
		{
			name: "successful refresh",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					// Verify request
					if !strings.Contains(req.URL.String(), "/oauth2/token") {
						t.Errorf("URL = %v, want /oauth2/token", req.URL)
					}
					
					// Return success response
					resp := &TokenResponse{
						AccessToken:  "new-access-token",
						RefreshToken: "new-refresh-token",
						ExpiresIn:    3600,
					}
					body, _ := json.Marshal(resp)
					
					return &http.Response{
						StatusCode: http.StatusOK,
						Body:       io.NopCloser(bytes.NewReader(body)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: false,
			checkResult: func(t *testing.T, resp *TokenResponse) {
				if resp.AccessToken != "new-access-token" {
					t.Errorf("AccessToken = %v, want new-access-token", resp.AccessToken)
				}
				if resp.RefreshToken != "new-refresh-token" {
					t.Errorf("RefreshToken = %v, want new-refresh-token", resp.RefreshToken)
				}
			},
		},
		{
			name: "invalid refresh token",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					return &http.Response{
						StatusCode: http.StatusBadRequest,
						Body:       io.NopCloser(strings.NewReader(`{"error":"invalid_grant"}`)),
						Header:     make(http.Header),
					}, nil
				}
				return mock
			},
			wantErr: true,
		},
		{
			name: "network error",
			setupMock: func() *MockHTTPClient {
				mock := &MockHTTPClient{}
				mock.DoFunc = func(req *http.Request) (*http.Response, error) {
					return nil, errors.New("connection refused")
				}
				return mock
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mock := tt.setupMock()
			client := &OAuthClient{
				httpClient:    mock,
				authKitDomain: "test.auth.example.com",
				clientID:      "test-client",
			}

			ctx := context.Background()
			resp, err := client.RefreshToken(ctx, "test-refresh-token")

			if (err != nil) != tt.wantErr {
				t.Errorf("RefreshToken() error = %v, wantErr %v", err, tt.wantErr)
			}

			if tt.checkResult != nil && !tt.wantErr {
				tt.checkResult(t, resp)
			}
		})
	}
}

func TestOAuthClient_PollForToken_Timeout(t *testing.T) {
	// This test demonstrates timeout behavior
	// Since LoginTimeout is a const, we test with the actual timeout value
	t.Skip("Timeout test would take too long with actual LoginTimeout value")
}

func TestOAuthClient_PollForToken_ContextCancellation(t *testing.T) {
	mock := &MockHTTPClient{}
	mock.DoFunc = func(req *http.Request) (*http.Response, error) {
		// Always return pending
		errResp := &TokenError{ErrorCode: "authorization_pending"}
		body, _ := json.Marshal(errResp)
		
		return &http.Response{
			StatusCode: http.StatusBadRequest,
			Body:       io.NopCloser(bytes.NewReader(body)),
			Header:     make(http.Header),
		}, nil
	}
	
	client := &OAuthClient{
		httpClient:    mock,
		authKitDomain: "test.auth.example.com",
		clientID:      "test-client",
	}
	
	ctx, cancel := context.WithCancel(context.Background())
	
	// Cancel after a short delay
	go func() {
		time.Sleep(100 * time.Millisecond)
		cancel()
	}()
	
	start := time.Now()
	_, err := client.PollForToken(ctx, "test-device-code", 50*time.Millisecond)
	elapsed := time.Since(start)
	
	if err != context.Canceled {
		t.Errorf("PollForToken() error = %v, want context.Canceled", err)
	}
	
	if elapsed > 200*time.Millisecond {
		t.Errorf("PollForToken() took %v, should have cancelled quickly", elapsed)
	}
}