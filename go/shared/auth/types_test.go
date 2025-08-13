package auth

import (
	"testing"
	"time"
)

func TestCredentials_IsExpired(t *testing.T) {
	tests := []struct {
		name      string
		expiresAt *time.Time
		want      bool
	}{
		{
			name:      "no expiry",
			expiresAt: nil,
			want:      false,
		},
		{
			name:      "future expiry",
			expiresAt: timePtr(time.Now().Add(time.Hour)),
			want:      false,
		},
		{
			name:      "past expiry",
			expiresAt: timePtr(time.Now().Add(-time.Hour)),
			want:      true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Credentials{
				ExpiresAt: tt.expiresAt,
			}
			if got := c.IsExpired(); got != tt.want {
				t.Errorf("IsExpired() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestCredentials_TimeUntilExpiry(t *testing.T) {
	tests := []struct {
		name      string
		expiresAt *time.Time
		wantRange [2]time.Duration // min, max acceptable range
	}{
		{
			name:      "no expiry",
			expiresAt: nil,
			wantRange: [2]time.Duration{0, 0},
		},
		{
			name:      "future expiry",
			expiresAt: timePtr(time.Now().Add(time.Hour)),
			wantRange: [2]time.Duration{59 * time.Minute, 61 * time.Minute},
		},
		{
			name:      "past expiry",
			expiresAt: timePtr(time.Now().Add(-time.Hour)),
			wantRange: [2]time.Duration{-61 * time.Minute, -59 * time.Minute},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := &Credentials{
				ExpiresAt: tt.expiresAt,
			}
			got := c.TimeUntilExpiry()
			
			if tt.wantRange[0] == 0 && tt.wantRange[1] == 0 {
				if got != 0 {
					t.Errorf("TimeUntilExpiry() = %v, want 0", got)
				}
			} else if got < tt.wantRange[0] || got > tt.wantRange[1] {
				t.Errorf("TimeUntilExpiry() = %v, want between %v and %v", 
					got, tt.wantRange[0], tt.wantRange[1])
			}
		})
	}
}

func TestTokenError_IsAuthorizationPending(t *testing.T) {
	tests := []struct {
		name      string
		errorCode string
		want      bool
	}{
		{
			name:      "authorization pending",
			errorCode: "authorization_pending",
			want:      true,
		},
		{
			name:      "other error",
			errorCode: "invalid_grant",
			want:      false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			e := &TokenError{
				ErrorCode: tt.errorCode,
			}
			if got := e.IsAuthorizationPending(); got != tt.want {
				t.Errorf("IsAuthorizationPending() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestTokenError_IsSlowDown(t *testing.T) {
	tests := []struct {
		name      string
		errorCode string
		want      bool
	}{
		{
			name:      "slow down",
			errorCode: "slow_down",
			want:      true,
		},
		{
			name:      "other error",
			errorCode: "authorization_pending",
			want:      false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			e := &TokenError{
				ErrorCode: tt.errorCode,
			}
			if got := e.IsSlowDown(); got != tt.want {
				t.Errorf("IsSlowDown() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestTokenError_IsExpired(t *testing.T) {
	tests := []struct {
		name      string
		errorCode string
		want      bool
	}{
		{
			name:      "expired token",
			errorCode: "expired_token",
			want:      true,
		},
		{
			name:      "other error",
			errorCode: "invalid_request",
			want:      false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			e := &TokenError{
				ErrorCode: tt.errorCode,
			}
			if got := e.IsExpired(); got != tt.want {
				t.Errorf("IsExpired() = %v, want %v", got, tt.want)
			}
		})
	}
}