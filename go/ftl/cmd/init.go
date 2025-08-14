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
	Format        string // Configuration format: yaml, go, cue, json
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
	cmd.Flags().StringVar(&opts.Format, "format", "yaml", "configuration format (yaml, go, cue, json)")
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

	// Prompt for format if not specified and in interactive mode
	if !opts.NoInteractive && opts.Format == "" {
		if err := promptForFormat(opts); err != nil {
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

	Info("Initializing FTL project '%s' with %s format", opts.Name, opts.Format)

	// Create configuration based on format
	switch opts.Format {
	case "yaml":
		if err := createYAMLConfig(projectDir, opts); err != nil {
			return fmt.Errorf("failed to create ftl.yaml: %w", err)
		}
		Success("Created ftl.yaml")
	case "go":
		if err := createGoConfig(projectDir, opts); err != nil {
			return fmt.Errorf("failed to create main.go: %w", err)
		}
		Success("Created main.go")
	case "cue":
		if err := createCUEConfig(projectDir, opts); err != nil {
			return fmt.Errorf("failed to create app.cue: %w", err)
		}
		Success("Created app.cue")
	case "json":
		if err := createJSONConfig(projectDir, opts); err != nil {
			return fmt.Errorf("failed to create ftl.json: %w", err)
		}
		Success("Created ftl.json")
	default:
		return fmt.Errorf("unsupported format: %s", opts.Format)
	}

	// Create .gitignore
	if err := createGitignore(projectDir); err != nil {
		return fmt.Errorf("failed to create .gitignore: %w", err)
	}
	Success("Created .gitignore")

	// Print next steps based on format
	fmt.Println()
	Info("Next steps:")
	fmt.Println("  1. cd", opts.Name)
	switch opts.Format {
	case "go":
		fmt.Println("  2. Edit main.go to add your components")
		fmt.Println("  3. go run main.go > spin.toml")
		fmt.Println("  4. spin up")
	case "cue":
		fmt.Println("  2. Edit app.cue to add your components")
		fmt.Println("  3. ftl synth app.cue")
		fmt.Println("  4. spin up")
	default:
		fmt.Println("  2. ftl component add <name>")
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

func promptForFormat(opts *InitOptions) error {
	prompt := &survey.Select{
		Message: "Configuration format:",
		Options: []string{
			"yaml - Simple declarative YAML configuration",
			"go - Programmatic Go code with FTL SDK",
			"cue - Advanced CUE configuration language",
			"json - JSON configuration",
		},
		Default: "yaml - Simple declarative YAML configuration",
	}

	var choice string
	if err := survey.AskOne(prompt, &choice); err != nil {
		return err
	}

	// Extract format from choice
	switch choice {
	case "yaml - Simple declarative YAML configuration":
		opts.Format = "yaml"
	case "go - Programmatic Go code with FTL SDK":
		opts.Format = "go"
	case "cue - Advanced CUE configuration language":
		opts.Format = "cue"
	case "json - JSON configuration":
		opts.Format = "json"
	}

	return nil
}

func createYAMLConfig(dir string, opts *InitOptions) error {
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

// createGoConfig creates a Go-based configuration
func createGoConfig(dir string, opts *InitOptions) error {
	description := opts.Description
	if description == "" {
		description = fmt.Sprintf("%s - An FTL application", opts.Name)
	}

	// Create a main.go file with FTL CDK
	content := fmt.Sprintf(`package main

import (
	"fmt"
	"log"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
)

func main() {
	// Create your FTL application using the CDK
	cdk := synthesis.NewCDK()
	app := cdk.NewApp("%s").
		SetDescription("%s").
		SetVersion("0.1.0")

	// Add your components here
	// Example:
	// app.AddComponent("my-component").
	//     FromLocal("./build/component.wasm").
	//     WithBuild("cargo build --release").
	//     WithEnv("LOG_LEVEL", "info").
	//     Build()

	// Enable authentication (optional)
	// app.EnableWorkOSAuth("org_123456")

	// Build and synthesize to spin.toml
	builtCDK := app.Build()
	manifest, err := builtCDK.Synthesize()
	if err != nil {
		log.Fatalf("Failed to synthesize: %%v", err)
	}

	// Output the manifest
	fmt.Print(manifest)
}
`, opts.Name, description)

	// Write main.go
	mainPath := filepath.Join(dir, "main.go")
	if err := os.WriteFile(mainPath, []byte(content), 0644); err != nil {
		return err
	}

	// Create go.mod
	goMod := fmt.Sprintf(`module %s

go 1.21

require github.com/fastertools/ftl-cli/go/ftl v0.1.0

// For local development, uncomment and adjust the path:
// replace github.com/fastertools/ftl-cli/go/ftl => ../path/to/ftl
`, opts.Name)

	goModPath := filepath.Join(dir, "go.mod")
	return os.WriteFile(goModPath, []byte(goMod), 0644)
}

// createCUEConfig creates a CUE-based configuration
func createCUEConfig(dir string, opts *InitOptions) error {
	description := opts.Description
	if description == "" {
		description = fmt.Sprintf("%s - An FTL application", opts.Name)
	}

	// Create app.cue with FTL patterns
	content := fmt.Sprintf(`package app

import "github.com/fastertools/ftl-cli/patterns"

// Define your FTL application
app: #FTLApplication & {
	name:        "%s"
	version:     "0.1.0"
	description: "%s"
	
	// Add your components here
	components: [
		// {
		//     id: "my-component"
		//     source: "./build/component.wasm"
		//     build: {
		//         command: "cargo build --release"
		//         watch: ["src/**/*.rs", "Cargo.toml"]
		//     }
		//     variables: {
		//         LOG_LEVEL: "info"
		//     }
		// },
	]
	
	// Configure access (public or private)
	access: "public"
	
	// Configure authentication (optional)
	// auth: {
	//     provider: "workos"
	//     org_id: "org_123456"
	// }
}
`, opts.Name, description)

	cuePath := filepath.Join(dir, "app.cue")
	return os.WriteFile(cuePath, []byte(content), 0644)
}

// createJSONConfig creates a JSON-based configuration
func createJSONConfig(dir string, opts *InitOptions) error {
	description := opts.Description
	if description == "" {
		description = fmt.Sprintf("%s - An FTL application", opts.Name)
	}

	// Create ftl.json
	content := fmt.Sprintf(`{
  "application": {
    "name": "%s",
    "version": "0.1.0",
    "description": "%s"
  },
  "components": [
    
  ],
  "triggers": [
    
  ]
}
`, opts.Name, description)

	jsonPath := filepath.Join(dir, "ftl.json")
	return os.WriteFile(jsonPath, []byte(content), 0644)
}
