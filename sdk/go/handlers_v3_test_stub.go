//go:build test

package ftl

// createToolsIfAvailable is a no-op stub when HTTP functionality is not available (test builds)
func createToolsIfAvailable(tools map[string]ToolDefinition) {
	// No-op for test builds - CreateTools is not available
	// Tools are still tracked in registeredV3Tools for testing
}