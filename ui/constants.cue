package ui

// ===========================================================================
// FTL Constants (shared across all commands and UI)
// ===========================================================================

#Constants: {
	// Supported languages for FTL projects
	languages: ["rust", "typescript", "python", "go"]
	
	// Supported config file formats
	formats: ["yaml", "json", "cue", "go"]
	
	// CLI output field ordering for consistent table display
	output_fields: ["Name", "ID", "Status", "URL", "Error", "Access", "OrgID", "Created", "Updated"]
	
	// MCP core tools registry
	mcp_tools: ["ftl-init", "ftl-build", "ftl-up", "ftl-status", "ftl-logs"]
	
	// Language-specific watch patterns for development
	watch_patterns: {
		go:         ["**/*.go", "go.mod"]
		rust:       ["src/**/*.rs", "Cargo.toml"]
		typescript: ["src/**/*.js", "package.json"]
		python:     ["**/*.py"]
	}
	
	// Config file detection priority order
	config_priorities: {
		default: ["ftl.yaml", "ftl.yml", "ftl.json", "app.cue"]
		build:   ["ftl.yaml", "ftl.json", "app.cue", "main.go"]
		synth:   ["ftl.yaml", "ftl.yml", "ftl.json", "main.go", "platform.go", "ftl.cue", "app.cue"]
	}
}