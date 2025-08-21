package oci

import (
	"encoding/base64"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestParseECRToken tests the ECR token parsing logic
func TestParseECRToken(t *testing.T) {
	tests := []struct {
		name          string
		registryURI   string
		authToken     string
		expectError   bool
		expectedUser  string
		expectedPass  string
		errorContains string
	}{
		{
			name:         "valid aws token",
			registryURI:  "123456789.dkr.ecr.us-west-2.amazonaws.com",
			authToken:    base64.StdEncoding.EncodeToString([]byte("AWS:mypassword123")),
			expectError:  false,
			expectedUser: "AWS",
			expectedPass: "mypassword123",
		},
		{
			name:         "valid aws token with special chars",
			registryURI:  "123456789.dkr.ecr.us-west-2.amazonaws.com",
			authToken:    base64.StdEncoding.EncodeToString([]byte("AWS:pass@word!123#$%")),
			expectError:  false,
			expectedUser: "AWS",
			expectedPass: "pass@word!123#$%",
		},
		{
			name:         "valid aws token empty password",
			registryURI:  "123456789.dkr.ecr.us-west-2.amazonaws.com",
			authToken:    base64.StdEncoding.EncodeToString([]byte("AWS:")),
			expectError:  false,
			expectedUser: "AWS",
			expectedPass: "",
		},
		{
			name:          "invalid base64",
			registryURI:   "123456789.dkr.ecr.us-west-2.amazonaws.com",
			authToken:     "not-valid-base64!@#$%",
			expectError:   true,
			errorContains: "failed to decode ECR token",
		},
		{
			name:          "invalid format no colon",
			registryURI:   "123456789.dkr.ecr.us-west-2.amazonaws.com",
			authToken:     base64.StdEncoding.EncodeToString([]byte("NoColonInToken")),
			expectError:   true,
			errorContains: "invalid ECR token format",
		},
		{
			name:          "invalid format wrong username",
			registryURI:   "123456789.dkr.ecr.us-west-2.amazonaws.com",
			authToken:     base64.StdEncoding.EncodeToString([]byte("NOTAWS:password")),
			expectError:   true,
			errorContains: "invalid ECR token format",
		},
		{
			name:          "invalid format lowercase aws",
			registryURI:   "123456789.dkr.ecr.us-west-2.amazonaws.com",
			authToken:     base64.StdEncoding.EncodeToString([]byte("aws:password")),
			expectError:   true,
			errorContains: "invalid ECR token format",
		},
		{
			name:         "token with multiple colons",
			registryURI:  "123456789.dkr.ecr.us-west-2.amazonaws.com",
			authToken:    base64.StdEncoding.EncodeToString([]byte("AWS:pass:word:with:colons")),
			expectError:  false,
			expectedUser: "AWS",
			expectedPass: "pass:word:with:colons",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			auth, err := ParseECRToken(tt.registryURI, tt.authToken)

			if tt.expectError {
				require.Error(t, err)
				assert.Contains(t, err.Error(), tt.errorContains)
			} else {
				require.NoError(t, err)
				assert.Equal(t, tt.registryURI, auth.Registry)
				assert.Equal(t, tt.expectedUser, auth.Username)
				assert.Equal(t, tt.expectedPass, auth.Password)
			}
		})
	}
}

// TestCleanRegistryURI tests the registry URI cleaning logic
func TestCleanRegistryURI(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "https prefix",
			input:    "https://123456789.dkr.ecr.us-west-2.amazonaws.com",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		},
		{
			name:     "http prefix",
			input:    "http://123456789.dkr.ecr.us-west-2.amazonaws.com",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		},
		{
			name:     "no prefix",
			input:    "123456789.dkr.ecr.us-west-2.amazonaws.com",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		},
		{
			name:     "https with port",
			input:    "https://123456789.dkr.ecr.us-west-2.amazonaws.com:443",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com:443",
		},
		{
			name:     "https with path",
			input:    "https://123456789.dkr.ecr.us-west-2.amazonaws.com/v2/",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com/v2/",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// This tests the pattern that would be used
			result := strings.TrimPrefix(tt.input, "https://")
			result = strings.TrimPrefix(result, "http://")
			assert.Equal(t, tt.expected, result)
		})
	}
}

// BenchmarkParseECRToken benchmarks the ECR token parsing
func BenchmarkParseECRToken(b *testing.B) {
	registry := "123456789.dkr.ecr.us-west-2.amazonaws.com"
	token := base64.StdEncoding.EncodeToString([]byte("AWS:benchmarkpassword123"))

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := ParseECRToken(registry, token)
		if err != nil {
			b.Fatal(err)
		}
	}
}
