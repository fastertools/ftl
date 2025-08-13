package construct

import (
	"testing"
)

func TestGetAvailableConstructs(t *testing.T) {
	constructs := GetAvailableConstructs()

	if len(constructs) == 0 {
		t.Errorf("Expected at least one construct, got none")
	}

	// Verify MCP construct is present and stable
	var mcpConstruct *Construct
	for _, c := range constructs {
		if c.Name == "mcp" {
			mcpConstruct = &c
			break
		}
	}

	if mcpConstruct == nil {
		t.Errorf("MCP construct should be available")
	} else {
		if mcpConstruct.Status != "stable" {
			t.Errorf("MCP construct should be stable, got %s", mcpConstruct.Status)
		}
		if mcpConstruct.Description == "" {
			t.Errorf("MCP construct should have a description")
		}
		if len(mcpConstruct.Features) == 0 {
			t.Errorf("MCP construct should have features listed")
		}
	}

	// Verify all constructs have required fields
	for _, c := range constructs {
		if c.Name == "" {
			t.Errorf("Construct should have a name")
		}
		if c.Description == "" {
			t.Errorf("Construct %s should have a description", c.Name)
		}
		if c.Status == "" {
			t.Errorf("Construct %s should have a status", c.Name)
		}
		if c.Status != "stable" && c.Status != "preview" && c.Status != "planned" {
			t.Errorf("Construct %s has invalid status: %s", c.Name, c.Status)
		}
	}
}

func TestGetConstruct(t *testing.T) {
	tests := []struct {
		name     string
		expected bool
	}{
		{"mcp", true},
		{"wordpress", true},
		{"microservices", true},
		{"ai-pipeline", true},
		{"nonexistent", false},
		{"", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := GetConstruct(tt.name)
			
			if tt.expected && result == nil {
				t.Errorf("Expected to find construct %q, but got nil", tt.name)
			}
			
			if !tt.expected && result != nil {
				t.Errorf("Expected not to find construct %q, but got %+v", tt.name, result)
			}
			
			if result != nil && result.Name != tt.name {
				t.Errorf("Expected construct name %q, got %q", tt.name, result.Name)
			}
		})
	}
}

func TestConstructFields(t *testing.T) {
	mcp := GetConstruct("mcp")
	if mcp == nil {
		t.Fatal("MCP construct should exist")
	}

	// Test MCP-specific fields
	expectedFeatures := []string{
		"JWT-based authentication",
		"MCP gateway",
		"component discovery",
		"security",
		"multi-tool",
	}

	for _, expected := range expectedFeatures {
		found := false
		for _, feature := range mcp.Features {
			if containsIgnoreCase(feature, expected) {
				found = true
				break
			}
		}
		if !found {
			t.Errorf("MCP construct should mention %q in features, got: %v", expected, mcp.Features)
		}
	}

	// Test examples
	if len(mcp.Examples) == 0 {
		t.Errorf("MCP construct should have usage examples")
	}

	for _, example := range mcp.Examples {
		if !containsIgnoreCase(example, "spin-compose init") {
			t.Errorf("MCP examples should show how to use init command, got: %q", example)
		}
	}
}

// Helper function to check if a string contains a substring (case insensitive)
func containsIgnoreCase(s, substr string) bool {
	s = toLower(s)
	substr = toLower(substr)
	return contains(s, substr)
}

// Simple case conversion (for test purposes)
func toLower(s string) string {
	result := make([]byte, len(s))
	for i, b := range []byte(s) {
		if b >= 'A' && b <= 'Z' {
			result[i] = b + 32
		} else {
			result[i] = b
		}
	}
	return string(result)
}

// Simple substring check
func contains(s, substr string) bool {
	if len(substr) > len(s) {
		return false
	}
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}