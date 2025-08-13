package auth

import (
	"context"
	"net/http"
	"time"
)

// OAuthProvider defines the interface for OAuth operations
type OAuthProvider interface {
	// StartDeviceFlow initiates the OAuth device flow
	StartDeviceFlow(ctx context.Context) (*DeviceAuthResponse, error)
	// PollForToken polls the token endpoint until authentication completes
	PollForToken(ctx context.Context, deviceCode string, interval time.Duration) (*TokenResponse, error)
	// RefreshToken refreshes an expired access token
	RefreshToken(ctx context.Context, refreshToken string) (*TokenResponse, error)
}

// HTTPClient defines the interface for HTTP operations
type HTTPClient interface {
	Do(req *http.Request) (*http.Response, error)
}

// BrowserOpener defines the interface for opening URLs in a browser
type BrowserOpener interface {
	OpenURL(url string) error
}