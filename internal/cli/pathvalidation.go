package cli

import (
	"fmt"
	"path/filepath"
	"strings"
)

// validateUserPath validates user-provided file paths to prevent path traversal
// and shell injection attacks. Based on the validation patterns from synth.go.
func validateUserPath(path string) error {
	if path == "" {
		return fmt.Errorf("path cannot be empty")
	}

	// Clean the path first to resolve any . and .. elements
	cleaned := filepath.Clean(path)
	
	// Check for path traversal patterns
	if strings.Contains(cleaned, "..") {
		return fmt.Errorf("path traversal not allowed in path: %s", path)
	}
	
	// Check for shell metacharacters (defense in depth)
	// This matches the validation from synth.go:190-192
	if strings.ContainsAny(cleaned, ";|&$`\\\"'<>(){}[]!*?~") {
		return fmt.Errorf("invalid characters in file path: %s", path)
	}
	
	return nil
}

// validateAndCleanPath validates a user path and returns the cleaned version
func validateAndCleanPath(path string) (string, error) {
	if err := validateUserPath(path); err != nil {
		return "", err
	}
	return filepath.Clean(path), nil
}