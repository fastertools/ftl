package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/AlecAivazis/survey/v2"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/config"
)

// InitOptions holds options for the init command
type InitOptions struct {
	Name          string
	Description   string
	Template      string
	NoInteractive bool
	Force         bool
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

	// Create ftl.yaml configuration
	if err := createFTLConfig(projectDir, opts); err != nil {
		return fmt.Errorf("failed to create ftl.yaml: %w", err)
	}
	Success("Created ftl.yaml")

	// Create .gitignore
	if err := createGitignore(projectDir); err != nil {
		return fmt.Errorf("failed to create .gitignore: %w", err)
	}
	Success("Created .gitignore")


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
	description := opts.Description
	if description == "" {
		description = fmt.Sprintf("%s - An FTL application", opts.Name)
	}

	cfg := &config.FTLConfig{
		Application: config.ApplicationConfig{
			Name:        opts.Name,
			Version:     "0.1.0",
			Description: description,
		},
	}

	configPath := filepath.Join(dir, "ftl.yaml")
	data, err := yaml.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}
	return os.WriteFile(configPath, data, 0644)
}

// createSpinComposeConfig is no longer needed as we create ftl.yaml in createFTLConfig

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

// Template generation functions will be reimplemented with the new schema
// func createExampleComponent(dir string, template string) error { ... }
// func generateMCPTemplate(name string) string { ... }
// func generateBasicTemplate(name string) string { ... }
// func generateEmptyTemplate(name string) string { ... }
