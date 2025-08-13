package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
)

var initCmd = &cobra.Command{
	Use:   "init [project-name]",
	Short: "Initialize a new spin-compose project",
	Long: `Initialize a new spin-compose project with a template.

The init command creates a new project directory with a spinc.yaml configuration
file and optional supporting files based on the selected template.`,
	Args: cobra.MaximumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		projectName := "my-app"
		if len(args) > 0 {
			projectName = args[0]
		}
		
		template, _ := cmd.Flags().GetString("template")
		return runInit(projectName, template)
	},
}

func init() {
	initCmd.Flags().StringP("template", "t", "mcp", "Template to use (mcp)")
}

func runInit(projectName, template string) error {
	printInfo("Initializing %s project '%s'", template, projectName)
	
	// Create project directory if it doesn't exist
	if _, err := os.Stat(projectName); os.IsNotExist(err) {
		if err := os.MkdirAll(projectName, 0755); err != nil {
			return fmt.Errorf("failed to create project directory: %w", err)
		}
	}
	
	// Generate configuration based on template
	var config string
	switch template {
	case "mcp":
		config = generateMCPConfig(projectName)
	default:
		printError("Unknown template '%s'", template)
		printInfo("Available templates: mcp")
		return fmt.Errorf("unknown template: %s", template)
	}
	
	// Write configuration file
	configPath := filepath.Join(projectName, "spinc.yaml")
	if err := os.WriteFile(configPath, []byte(config), 0644); err != nil {
		return fmt.Errorf("failed to write configuration file: %w", err)
	}
	printSuccess("Created %s", configPath)
	
	// Create .gitignore if it doesn't exist
	gitignorePath := filepath.Join(projectName, ".gitignore")
	if _, err := os.Stat(gitignorePath); os.IsNotExist(err) {
		gitignoreContent := `# Generated files
spin.toml
.spin/

# Build artifacts
*.wasm
dist/
target/
node_modules/

# IDE files
.vscode/
.idea/
*.swp
*.swo
`
		if err := os.WriteFile(gitignorePath, []byte(gitignoreContent), 0644); err != nil {
			return fmt.Errorf("failed to write .gitignore: %w", err)
		}
		printSuccess("Created %s", gitignorePath)
	}
	
	// Print next steps
	fmt.Println()
	printInfo("Next steps:")
	fmt.Printf("  1. %s\n", infoColor.Sprint("cd "+projectName))
	fmt.Printf("  2. %s\n", infoColor.Sprint("Edit spinc.yaml to configure your application"))
	fmt.Printf("  3. %s\n", infoColor.Sprint("spin-compose synth"))
	fmt.Printf("  4. %s\n", infoColor.Sprint("spin up"))
	
	return nil
}

func generateMCPConfig(projectName string) string {
	return fmt.Sprintf(`# spin-compose configuration
# This file defines your MCP (Model Context Protocol) application

name: %s
version: 0.1.0
description: MCP application with authentication and tool gateway

# Authentication configuration
# Set enabled: false to disable authentication entirely
auth:
  enabled: true
  issuer: https://auth.example.com
  audience:
    - api.example.com
  # Optional: specify additional auth parameters
  # jwks_uri: https://auth.example.com/.well-known/jwks.json
  # algorithm: RS256
  # required_scopes: "read:tools write:tools"

# MCP system configuration
mcp:
  gateway: ghcr.io/fastertools/mcp-gateway:latest
  authorizer: ghcr.io/fastertools/mcp-authorizer:latest
  validate_arguments: false

# Application variables
# These can be overridden at runtime or in different environments
variables:
  log_level: debug
  # custom_var: 
  #   default: "default-value"
  # required_var:
  #   required: true

# Your application components
# Add your tools and services here
components:
  # Example tool component (uncomment and modify as needed)
  # example-tool:
  #   source: ./build/example.wasm
  #   route: /example
  #   environment:
  #     LOG_LEVEL: debug
  #   build:
  #     command: cargo build --target wasm32-wasip1 --release
  #     watch:
  #       - src/**/*.rs
  #       - Cargo.toml
`, projectName)
}