package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/AlecAivazis/survey/v2"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/shared/config"
)

// InitOptions holds options for the init command
type InitOptions struct {
	Name        string
	Description string
	Template    string
	NoInteractive bool
	Force       bool
}

// newInitCmd creates the init command
func newInitCmd() *cobra.Command {
	opts := &InitOptions{}

	cmd := &cobra.Command{
		Use:   "init [name]",
		Short: "Initialize a new FTL project",
		Long: `Initialize a new FTL project with the specified name.

This command creates a new FTL project directory with:
- ftl.yaml configuration file
- spinc.yaml for application composition
- Basic project structure
- Example components (optional)`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				opts.Name = args[0]
			}
			return runInit(opts)
		},
	}

	cmd.Flags().StringVarP(&opts.Description, "description", "d", "", "project description")
	cmd.Flags().StringVarP(&opts.Template, "template", "t", "mcp", "project template (mcp, basic, empty)")
	cmd.Flags().BoolVar(&opts.NoInteractive, "no-interactive", false, "disable interactive prompts")
	cmd.Flags().BoolVarP(&opts.Force, "force", "f", false, "overwrite existing files")

	return cmd
}

func runInit(opts *InitOptions) error {
	// Validate or prompt for name
	if opts.Name == "" {
		if opts.NoInteractive {
			return fmt.Errorf("project name is required")
		}
		if err := promptForName(opts); err != nil {
			return err
		}
	}

	// Create project directory
	projectDir := opts.Name
	if !opts.Force {
		if _, err := os.Stat(projectDir); err == nil {
			return fmt.Errorf("directory %s already exists (use --force to overwrite)", projectDir)
		}
	}

	if err := os.MkdirAll(projectDir, 0755); err != nil {
		return fmt.Errorf("failed to create project directory: %w", err)
	}

	Info("Initializing FTL project '%s'", opts.Name)

	// Create ftl.yaml
	if err := createFTLConfig(projectDir, opts); err != nil {
		return fmt.Errorf("failed to create ftl.yaml: %w", err)
	}
	Success("Created ftl.yaml")

	// Create spinc.yaml based on template
	if err := createSpinComposeConfig(projectDir, opts); err != nil {
		return fmt.Errorf("failed to create spinc.yaml: %w", err)
	}
	Success("Created spinc.yaml")

	// Create .gitignore
	if err := createGitignore(projectDir); err != nil {
		return fmt.Errorf("failed to create .gitignore: %w", err)
	}
	Success("Created .gitignore")

	// Create example component if using template
	if opts.Template != "empty" {
		if err := createExampleComponent(projectDir, opts.Template); err != nil {
			Warn("Failed to create example component: %v", err)
		} else {
			Success("Created example component")
		}
	}

	// Print next steps
	fmt.Println()
	Info("Next steps:")
	fmt.Println("  1. cd", opts.Name)
	fmt.Println("  2. ftl component add <name> --language <lang>")
	fmt.Println("  3. ftl build")
	fmt.Println("  4. ftl up")

	return nil
}

func promptForName(opts *InitOptions) error {
	prompt := &survey.Input{
		Message: "Project name:",
		Help:    "The name of your FTL project (lowercase, alphanumeric, hyphens)",
	}
	return survey.AskOne(prompt, &opts.Name, survey.WithValidator(survey.Required))
}

func createFTLConfig(dir string, opts *InitOptions) error {
	cfg := &config.FTLConfig{
		Name:        opts.Name,
		Version:     "0.1.0",
		Description: opts.Description,
		Compose:     "./spinc.yaml",
	}

	if opts.Description == "" {
		cfg.Description = fmt.Sprintf("%s - An FTL application", opts.Name)
	}

	configPath := filepath.Join(dir, "ftl.yaml")
	return cfg.Save(configPath)
}

func createSpinComposeConfig(dir string, opts *InitOptions) error {
	var content string

	switch opts.Template {
	case "", "mcp": // Default to MCP template if not specified
		content = generateMCPTemplate(opts.Name)
	case "basic":
		content = generateBasicTemplate(opts.Name)
	case "empty":
		content = generateEmptyTemplate(opts.Name)
	default:
		return fmt.Errorf("unknown template: %s", opts.Template)
	}

	configPath := filepath.Join(dir, "spinc.yaml")
	return os.WriteFile(configPath, []byte(content), 0644)
}

func createGitignore(dir string) error {
	content := `.spin/
spin.toml
*.wasm
.ftl/
.env
.env.local
target/
dist/
node_modules/
__pycache__/
*.pyc
.DS_Store
`
	gitignorePath := filepath.Join(dir, ".gitignore")
	return os.WriteFile(gitignorePath, []byte(content), 0644)
}

func createExampleComponent(dir string, template string) error {
	// This would create an example component based on the template
	// For now, we'll just create a placeholder directory
	componentDir := filepath.Join(dir, "components", "example")
	return os.MkdirAll(componentDir, 0755)
}

func generateMCPTemplate(name string) string {
	return fmt.Sprintf(`# MCP Application Configuration
name: %s
version: 0.1.0
description: MCP application with authentication and tool gateway

# Authentication configuration
auth:
  enabled: false  # Set to true to enable authentication
  # issuer: https://auth.example.com
  # audience:
  #   - api.example.com

# MCP components
mcp:
  gateway: ghcr.io/fastertools/mcp-gateway:latest
  authorizer: ghcr.io/fastertools/mcp-authorizer:latest
  validate_arguments: false

# Application variables
variables:
  log_level: info

# Components - add your tools here
components:
  # example-tool:
  #   source: ./components/example/build/example.wasm
  #   route: /example
`, name)
}

func generateBasicTemplate(name string) string {
	return fmt.Sprintf(`# Basic Application Configuration
name: %s
version: 0.1.0
description: Basic Spin application

# Application variables
variables:
  log_level: info

# Components
components:
  # Add your components here
`, name)
}

func generateEmptyTemplate(name string) string {
	return fmt.Sprintf(`# Application Configuration
name: %s
version: 0.1.0

components: {}
`, name)
}