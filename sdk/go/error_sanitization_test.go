//go:build test

package ftl

import (
	"errors"
	"strings"
	"testing"
)

// TestSanitizeErrorMessage tests error message sanitization
func TestSanitizeErrorMessage(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "SafeMessage",
			input:    "validation failed: field is required",
			expected: "validation failed: field is required",
		},
		{
			name:     "FilePathReference",
			input:    "error in /usr/local/go/src/runtime/panic.go:123",
			expected: "An error occurred during processing",
		},
		{
			name:     "PanicStackTrace",
			input:    "panic: runtime error: nil pointer dereference",
			expected: "An error occurred during processing",
		},
		{
			name:     "MemoryAddress",
			input:    "invalid memory address 0x7fff5fbff7e8",
			expected: "An error occurred during processing",
		},
		{
			name:     "RuntimeInternals",
			input:    "runtime.gopanic at /usr/local/go/src/runtime/panic.go:838",
			expected: "An error occurred during processing",
		},
		{
			name:     "ReflectionInternals",
			input:    "reflect.Value.Interface: cannot return value obtained from unexported field",
			expected: "An error occurred during processing",
		},
		{
			name:     "TooLongMessage",
			input:    strings.Repeat("a", 250),
			expected: strings.Repeat("a", 200) + "...",
		},
		{
			name:     "EmptyMessage",
			input:    "",
			expected: "An error occurred during processing",
		},
		{
			name:     "WhitespaceOnlyMessage",
			input:    "   \t\n  ",
			expected: "An error occurred during processing",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := sanitizeErrorMessage(tt.input)
			if result != tt.expected {
				t.Errorf("sanitizeErrorMessage(%q) = %q, want %q", tt.input, result, tt.expected)
			}
		})
	}
}

// TestConvertError_Sanitization tests that convertError properly sanitizes error messages
func TestConvertError_Sanitization(t *testing.T) {
	tests := []struct {
		name           string
		input          error
		expectGeneric  bool
		expectedPrefix string
	}{
		{
			name:           "ValidationError_Safe",
			input:          ValidationError{Field: "name", Message: "field is required"},
			expectGeneric:  false,
			expectedPrefix: "Invalid input for field 'name': field is required",
		},
		{
			name:           "ValidationError_Unsafe",
			input:          ValidationError{Field: "name", Message: "panic: runtime error"},
			expectGeneric:  false,
			expectedPrefix: "Invalid input for field 'name': An error occurred during processing",
		},
		{
			name:          "ToolError_WithCause",
			input:         ToolError{Code: "test", Message: "operation failed", Cause: errors.New("internal panic")},
			expectGeneric: false,
			expectedPrefix: "operation failed: internal error occurred",
		},
		{
			name:          "ToolError_UnsafeMessage",
			input:         ToolError{Code: "test", Message: "error in /usr/local/go/src/runtime/panic.go:123"},
			expectGeneric: true,
		},
		{
			name:          "GenericError",
			input:         errors.New("some internal error"),
			expectGeneric: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			response := convertError(tt.input)
			
			if response.IsError != true {
				t.Errorf("convertError should return error response")
			}
			
			if len(response.Content) == 0 {
				t.Errorf("convertError should return content")
				return
			}
			
			errorMsg := response.Content[0].Text
			
			if tt.expectGeneric {
				if errorMsg != "An error occurred during processing" {
					t.Errorf("convertError(%v) should return generic message, got %q", tt.input, errorMsg)
				}
			} else if tt.expectedPrefix != "" {
				if errorMsg != tt.expectedPrefix {
					t.Errorf("convertError(%v) = %q, want %q", tt.input, errorMsg, tt.expectedPrefix)
				}
			}
		})
	}
}

// TestToolError_SanitizedString tests that ToolError.Error() method sanitizes output
func TestToolError_SanitizedString(t *testing.T) {
	tests := []struct {
		name     string
		err      ToolError
		expected string
	}{
		{
			name: "SafeMessage",
			err: ToolError{
				Code:    "validation_error",
				Message: "invalid input provided",
			},
			expected: "invalid input provided",
		},
		{
			name: "UnsafeMessage",
			err: ToolError{
				Code:    "internal_error",
				Message: "panic: runtime error at /usr/local/go/src/panic.go:123",
			},
			expected: "An error occurred during processing",
		},
		{
			name: "SafeMessageWithCause",
			err: ToolError{
				Code:    "execution_failed",
				Message: "database operation failed",
				Cause:   errors.New("connection refused"),
			},
			expected: "database operation failed: internal error occurred",
		},
		{
			name: "UnsafeMessageWithCause",
			err: ToolError{
				Code:    "execution_failed",
				Message: "runtime.gopanic at /usr/local/go/src/runtime/panic.go:838",
				Cause:   errors.New("some internal error"),
			},
			expected: "An error occurred during processing",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.err.Error()
			if result != tt.expected {
				t.Errorf("ToolError.Error() = %q, want %q", result, tt.expected)
			}
		})
	}
}