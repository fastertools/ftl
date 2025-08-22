package cli

import (
	"fmt"
	"strings"

	"github.com/AlecAivazis/survey/v2"
	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/internal/scaffold"
)

// AddOptions holds options for the add command
type AddOptions struct {
	Name     string
	Language string
}

// newAddCmd creates the add command
func newAddCmd() *cobra.Command {
	opts := &AddOptions{}

	cmd := &cobra.Command{
		Use:   "add [name]",
		Short: "Add a new component to your FTL project",
		Long: `Add a new component to your FTL project.

This command scaffolds a new MCP tool component in your chosen language,
including all necessary files, build configuration, and example code.

Supported languages:
  - rust       Rust with ftl-sdk
  - typescript TypeScript with ftl-sdk and Zod
  - python     Python with ftl-sdk and Pydantic
  - go         Go with ftl-sdk-go

Examples:
  # Interactive mode
  ftl add

  # With component name
  ftl add my-tool

  # With name and language
  ftl add my-tool --language rust`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				opts.Name = args[0]
			}
			return runAdd(opts)
		},
	}

	cmd.Flags().StringVarP(&opts.Language, "language", "l", "", "programming language (rust, typescript, python, go)")

	return cmd
}

func runAdd(opts *AddOptions) error {
	// Create scaffolder
	scaffolder, err := scaffold.NewScaffolder()
	if err != nil {
		return fmt.Errorf("failed to initialize scaffolder: %w", err)
	}

	// Get component name if not provided
	if opts.Name == "" {
		prompt := &survey.Input{
			Message: "Component name:",
			Help:    "The name of your component (lowercase, hyphens allowed)",
		}
		if err := survey.AskOne(prompt, &opts.Name, survey.WithValidator(survey.Required)); err != nil {
			return err
		}
	}

	// Validate component name
	if err := scaffold.ValidateComponentName(opts.Name); err != nil {
		return err
	}

	// Get language if not provided
	if opts.Language == "" {
		languageOptions := []string{
			"rust       - High-performance systems language",
			"typescript - Type-safe JavaScript",
			"python     - Versatile scripting language",
			"go         - Simple and efficient",
		}

		prompt := &survey.Select{
			Message: "Select programming language:",
			Options: languageOptions,
			Default: languageOptions[0],
		}

		var choice string
		if err := survey.AskOne(prompt, &choice); err != nil {
			return err
		}

		// Extract language from choice
		opts.Language = strings.Fields(choice)[0]
	}

	// Validate language
	validLanguages := scaffolder.ListLanguages()
	found := false
	for _, lang := range validLanguages {
		if lang == opts.Language {
			found = true
			break
		}
	}
	if !found {
		return fmt.Errorf("invalid language '%s': must be one of %v", opts.Language, validLanguages)
	}

	Info("Creating %s component '%s'", opts.Language, opts.Name)

	// Generate the component
	if err := scaffolder.GenerateComponent(opts.Name, opts.Language); err != nil {
		return fmt.Errorf("failed to generate component: %w", err)
	}

	// Print success message with project type awareness
	printSuccessMessage(opts.Name, opts.Language)

	return nil
}


// generateGoSnippet creates a Go code snippet for adding the tool to main.go
func generateGoSnippet(toolName string) string {
	return fmt.Sprintf(`// Add this to your main.go file in the appropriate location:

app.AddComponent("%s").
    FromLocal("./%s/%s.wasm").
    WithBuild("cd %s && make build").
    Build()`, toolName, toolName, toolName, toolName)
}

func printSuccessMessage(name, language string) {
	// Determine main file based on language
	var mainFile string
	switch language {
	case "rust":
		mainFile = fmt.Sprintf("%s/src/lib.rs", name)
	case "typescript":
		mainFile = fmt.Sprintf("%s/src/index.ts", name)
	case "python":
		mainFile = fmt.Sprintf("%s/src/main.py", name)
	case "go":
		mainFile = fmt.Sprintf("%s/main.go", name)
	}

	Success("Component '%s' created successfully!", name)
	fmt.Println()
	fmt.Println("📁 Component structure:")
	fmt.Printf("  %s/\n", name)
	fmt.Printf("  ├── %s\n", getMainFileName(language))
	fmt.Printf("  ├── Makefile\n")
	fmt.Printf("  └── %s\n", getConfigFileName(language))
	fmt.Println()
	fmt.Printf("💡 Edit %s to implement your tool logic\n", mainFile)
	fmt.Println()

	// Check if this is a Go CDK project and provide special instructions
	projectConfig := scaffold.DetectProject()
	if projectConfig.Type == scaffold.ProjectTypeGo {
		fmt.Println("🔧 Go CDK Project Detected!")
		fmt.Println()
		Warn("Manual component registration required for Go-based configurations.")
		fmt.Println()
		fmt.Println("📝 Add the following to your main.go file:")
		fmt.Println()
		fmt.Println(generateGoSnippet(name))
		fmt.Println()
		fmt.Println("🔨 Next steps:")
		fmt.Println("  1. cd", name)
		fmt.Println("  2. Edit the source files to implement your tools")
		fmt.Println("  3. Run 'make build' to compile")
		fmt.Println("  4. Add the component registration code to your main.go")
		fmt.Println("  5. Run 'ftl build' to generate the updated configuration")
		fmt.Println("  6. Run 'ftl up' to start the MCP server")
	} else {
		fmt.Println("🔨 Next steps:")
		fmt.Println("  1. cd", name)
		fmt.Println("  2. Edit the source files to implement your tools")
		fmt.Println("  3. Run 'make build' to compile")
		fmt.Println("  4. Return to project root and run 'ftl build'")
		fmt.Println("  5. Run 'ftl up' to start the MCP server")
	}

	fmt.Println()
	fmt.Println("📚 Learn more about the FTL SDK for", language+":")
	fmt.Printf("  https://github.com/fastertools/ftl-sdk-%s\n", getSdkSuffix(language))
}

func getMainFileName(language string) string {
	switch language {
	case "rust":
		return "src/lib.rs"
	case "typescript":
		return "src/index.ts"
	case "python":
		return "src/main.py"
	case "go":
		return "main.go"
	default:
		return "main"
	}
}

func getConfigFileName(language string) string {
	switch language {
	case "rust":
		return "Cargo.toml"
	case "typescript":
		return "package.json"
	case "python":
		return "pyproject.toml"
	case "go":
		return "go.mod"
	default:
		return "config"
	}
}

func getSdkSuffix(language string) string {
	switch language {
	case "rust":
		return "rust"
	case "typescript":
		return "js"
	case "python":
		return "python"
	case "go":
		return "go"
	default:
		return language
	}
}
