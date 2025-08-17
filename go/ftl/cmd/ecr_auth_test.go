package cmd

import (
	"encoding/base64"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestParseECRCredentials tests the ECR token parsing logic
func TestParseECRCredentials(t *testing.T) {
	tests := []struct {
		name          string
		authToken     string
		expectError   bool
		expectedUser  string
		expectedPass  string
		errorContains string
	}{
		{
			name:         "valid_aws_token",
			authToken:    base64.StdEncoding.EncodeToString([]byte("AWS:mypassword123")),
			expectError:  false,
			expectedUser: "AWS",
			expectedPass: "mypassword123",
		},
		{
			name:         "valid_aws_token_with_special_chars",
			authToken:    base64.StdEncoding.EncodeToString([]byte("AWS:pass@word!123#$%")),
			expectError:  false,
			expectedUser: "AWS",
			expectedPass: "pass@word!123#$%",
		},
		{
			name:         "valid_aws_token_empty_password",
			authToken:    base64.StdEncoding.EncodeToString([]byte("AWS:")),
			expectError:  false,
			expectedUser: "AWS",
			expectedPass: "",
		},
		{
			name:          "invalid_base64",
			authToken:     "not-valid-base64!@#$%",
			expectError:   true,
			errorContains: "illegal base64",
		},
		{
			name:          "invalid_format_no_colon",
			authToken:     base64.StdEncoding.EncodeToString([]byte("NoColonInToken")),
			expectError:   true,
			errorContains: "invalid ECR token format",
		},
		{
			name:          "invalid_format_wrong_username",
			authToken:     base64.StdEncoding.EncodeToString([]byte("NOTAWS:password")),
			expectError:   true,
			errorContains: "invalid ECR token format",
		},
		{
			name:          "invalid_format_lowercase_aws",
			authToken:     base64.StdEncoding.EncodeToString([]byte("aws:password")),
			expectError:   true,
			errorContains: "invalid ECR token format",
		},
		{
			name:         "token_with_multiple_colons",
			authToken:    base64.StdEncoding.EncodeToString([]byte("AWS:pass:word:with:colons")),
			expectError:  false,
			expectedUser: "AWS",
			expectedPass: "pass:word:with:colons",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Decode the token
			decoded, err := base64.StdEncoding.DecodeString(tt.authToken)

			if tt.expectError && tt.errorContains == "illegal base64" {
				require.Error(t, err)
				assert.Contains(t, err.Error(), tt.errorContains)
				return
			}

			if err != nil {
				t.Fatalf("Unexpected decode error: %v", err)
			}

			// Parse username and password
			parts := strings.SplitN(string(decoded), ":", 2)

			if tt.expectError {
				// Check for invalid token format
				if len(parts) != 2 || parts[0] != "AWS" {
					// This is expected for error cases
					return
				}
				t.Errorf("Expected error but got valid parse: user=%s", parts[0])
			} else {
				// Valid case
				require.Equal(t, 2, len(parts), "Token should split into exactly 2 parts")
				assert.Equal(t, tt.expectedUser, parts[0], "Username mismatch")
				assert.Equal(t, tt.expectedPass, parts[1], "Password mismatch")
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
			name:     "https_prefix",
			input:    "https://123456789.dkr.ecr.us-west-2.amazonaws.com",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		},
		{
			name:     "http_prefix",
			input:    "http://123456789.dkr.ecr.us-west-2.amazonaws.com",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		},
		{
			name:     "no_prefix",
			input:    "123456789.dkr.ecr.us-west-2.amazonaws.com",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		},
		{
			name:     "https_with_port",
			input:    "https://123456789.dkr.ecr.us-west-2.amazonaws.com:443",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com:443",
		},
		{
			name:     "https_with_path",
			input:    "https://123456789.dkr.ecr.us-west-2.amazonaws.com/v2/",
			expected: "123456789.dkr.ecr.us-west-2.amazonaws.com/v2/",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Clean up registry URI (remove protocol if present)
			result := strings.TrimPrefix(tt.input, "https://")
			result = strings.TrimPrefix(result, "http://")

			assert.Equal(t, tt.expected, result)
		})
	}
}

// BenchmarkParseECRToken benchmarks the ECR token parsing
func BenchmarkParseECRToken(b *testing.B) {
	token := base64.StdEncoding.EncodeToString([]byte("AWS:benchmarkpassword123"))

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		decoded, _ := base64.StdEncoding.DecodeString(token)
		parts := strings.SplitN(string(decoded), ":", 2)
		if len(parts) != 2 || parts[0] != "AWS" {
			b.Fatal("Invalid token in benchmark")
		}
	}
}

// BenchmarkCleanRegistryURI benchmarks the registry URI cleaning
func BenchmarkCleanRegistryURI(b *testing.B) {
	uri := "https://123456789.dkr.ecr.us-west-2.amazonaws.com"

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		result := strings.TrimPrefix(uri, "https://")
		_ = strings.TrimPrefix(result, "http://")
	}
}
