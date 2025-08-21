package cli

import (
	"fmt"
	"os"

	"github.com/AlecAivazis/survey/v2"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl/internal/scaffold"
)

// InitOptions holds options for the init command
type InitOptions struct {
	Name          string
	Description   string
	Template      string
	Language      string // Configuration language: yaml, go, cue, json
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
	cmd.Flags().StringVarP(&opts.Language, "language", "l", "", "configuration language (yaml, go, cue, json)")
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

	// Prompt for config language if not specified
	if opts.Language == "" {
		if opts.NoInteractive {
			// Default to YAML in non-interactive mode
			opts.Language = "yaml"
		} else {
			if err := promptForLanguage(opts); err != nil {
				return err
			}
		}
	}

	// Create project directory
	projectDir := opts.Name
	if !opts.Force {
		if _, err := os.Stat(projectDir); err == nil {
			return fmt.Errorf("directory %s already exists (use --force to overwrite)", projectDir)
		}
	}

	if err := os.MkdirAll(projectDir, 0750); err != nil {
		return fmt.Errorf("failed to create project directory: %w", err)
	}

	Info("Initializing FTL project '%s' with %s configuration", opts.Name, opts.Language)

	// Get description or use default
	description := opts.Description
	if description == "" {
		description = fmt.Sprintf("%s - An FTL application", opts.Name)
	}

	// Use scaffolder to generate project files from templates
	scaffolder, err := scaffold.NewScaffolder()
	if err != nil {
		return fmt.Errorf("failed to initialize scaffolder: %w", err)
	}

	if err := scaffolder.GenerateProject(projectDir, opts.Name, description, opts.Language); err != nil {
		return fmt.Errorf("failed to generate project: %w", err)
	}

	// Success messages for created files
	switch opts.Language {
	case "yaml":
		Success("Created ftl.yaml")
	case "go":
		Success("Created main.go")
		Success("Created go.mod")
	case "cue":
		Success("Created app.cue")
	case "json":
		Success("Created ftl.json")
	}
	Success("Created .gitignore")

	// Print next steps based on format
	fmt.Println()
	Info("Next steps:")
	fmt.Println("  1. cd", opts.Name)
	switch opts.Language {
	case "go":
		fmt.Println("  2. Edit main.go to add your components")
		fmt.Println("  3. go run main.go > spin.toml")
		fmt.Println("  4. spin up")
	case "cue":
		fmt.Println("  2. Edit app.cue to add your components")
		fmt.Println("  3. ftl synth app.cue")
		fmt.Println("  4. spin up")
	default:
		fmt.Printf("  2. Edit %s to add your components\n",
			map[string]string{"yaml": "ftl.yaml", "json": "ftl.json"}[opts.Language])
		fmt.Println("  3. ftl build")
		fmt.Println("  4. ftl up")
	}

	return nil
}

func promptForName(opts *InitOptions) error {
	prompt := &survey.Input{
		Message: "Project name:",
		Help:    "The name of your FTL project (lowercase, alphanumeric, hyphens)",
	}
	return survey.AskOne(prompt, &opts.Name, survey.WithValidator(survey.Required))
}

func promptForLanguage(opts *InitOptions) error {
	prompt := &survey.Select{
		Message: "Choose configuration language:",
		Options: []string{
			"yaml - Simple declarative YAML configuration (recommended)",
			"json - JSON configuration",
			"cue - Advanced CUE configuration language",
			"go - Programmatic Go code with FTL SDK",
		},
		Default: "yaml - Simple declarative YAML configuration (recommended)",
		Help:    "YAML is recommended for most users. CUE offers advanced validation. Go provides full programmatic control.",
	}

	var choice string
	if err := survey.AskOne(prompt, &choice); err != nil {
		return err
	}

	// Extract config language from choice
	switch choice {
	case "yaml - Simple declarative YAML configuration (recommended)":
		opts.Language = "yaml"
	case "json - JSON configuration":
		opts.Language = "json"
	case "cue - Advanced CUE configuration language":
		opts.Language = "cue"
	case "go - Programmatic Go code with FTL SDK":
		opts.Language = "go"
	}

	return nil
}

// All template generation functions have been moved to scaffold/templates.cue
// and are now handled by scaffolder.GenerateProject()
