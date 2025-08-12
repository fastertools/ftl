//go:build !test

package ftl

// createToolsIfAvailable calls CreateTools when the HTTP functionality is available
func createToolsIfAvailable(tools map[string]ToolDefinition) {
	CreateTools(tools)
}