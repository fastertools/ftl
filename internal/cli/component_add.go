package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/AlecAivazis/survey/v2"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
)

// AddComponentOptions holds options for adding a component
type AddComponentOptions struct {
	Name        string
	Source      string
	Registry    string
	Description string
	Template    string
	Build       string
}

func newComponentAddCmd() *cobra.Command {
	opts := &AddComponentOptions{}

	cmd := &cobra.Command{
		Use:   "add [name]",
		Short: "Add a component to the application",
		Long:  `Add a new component to your FTL application`,
		Args:  cobra.MaximumNArgs(1),
		Example: `  # Add a component interactively
  ftl component add

  # Add a component from a local path
  ftl component add my-component --source ./my-component

  # Add a component from a registry
  ftl component add my-component --registry ghcr.io/user:package:version

  # Add a component from a template
  ftl component add my-component --template go-http`,
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				opts.Name = args[0]
			}
			return runComponentAdd(opts)
		},
	}

	cmd.Flags().StringVarP(&opts.Source, "source", "s", "", "Path to component source")
	cmd.Flags().StringVarP(&opts.Registry, "registry", "r", "", "Registry source (format: registry/package:version)")
	cmd.Flags().StringVarP(&opts.Description, "description", "d", "", "Component description")
	cmd.Flags().StringVarP(&opts.Template, "template", "t", "", "Use a template (go-http, rust-wasm, js-http, python-http)")
	cmd.Flags().StringVarP(&opts.Build, "build", "b", "", "Build command")

	return cmd
}

func runComponentAdd(opts *AddComponentOptions) error {
	// Load existing manifest
	manifest, err := loadManifest("ftl.yaml")
	if err != nil {
		return fmt.Errorf("failed to load ftl.yaml: %w", err)
	}

	// Get component name if not provided
	if opts.Name == "" {
		prompt := &survey.Input{
			Message: "Component name:",
		}
		if err := survey.AskOne(prompt, &opts.Name, survey.WithValidator(survey.Required)); err != nil {
			return err
		}
	}

	// Validate name doesn't already exist
	components, _ := manifest["components"].([]interface{})
	for _, c := range components {
		comp := c.(map[interface{}]interface{})
		if comp["id"] == opts.Name {
			return fmt.Errorf("component '%s' already exists", opts.Name)
		}
	}

	// Determine source type
	var component map[string]interface{}
	if opts.Template != "" {
		component = createFromTemplate(opts)
	} else if opts.Registry != "" {
		component = createFromRegistry(opts)
	} else if opts.Source != "" {
		component = createFromLocal(opts)
	} else {
		// Interactive mode
		component, err = createInteractive(opts)
		if err != nil {
			return err
		}
	}

	// Add to manifest
	if components == nil {
		components = []interface{}{}
	}
	manifest["components"] = append(components, component)

	// Save manifest
	if err := saveManifest("ftl.yaml", manifest); err != nil {
		return fmt.Errorf("failed to save manifest: %w", err)
	}

	color.Green("✓ Component '%s' added successfully", opts.Name)
	fmt.Println()
	color.Blue("→ Next steps:")
	fmt.Println("  1. Run 'ftl synth' to generate spin.toml")
	fmt.Println("  2. Run 'ftl up' to start development")

	return nil
}

func createFromTemplate(opts *AddComponentOptions) map[string]interface{} {
	comp := make(map[string]interface{})
	comp["id"] = opts.Name

	// Set source based on template
	templateDir := fmt.Sprintf("components/%s", opts.Name)
	comp["source"] = templateDir

	// Set build command based on template type
	build := make(map[string]interface{})
	switch opts.Template {
	case "go-http":
		build["command"] = "tinygo build -target=wasip2 -o " + opts.Name + ".wasm main.go"
		build["watch"] = []string{"**/*.go", "go.mod"}
	case "rust-wasm":
		build["command"] = "cargo build --target wasm32-wasip2 --release"
		build["workdir"] = templateDir
		build["watch"] = []string{"src/**/*.rs", "Cargo.toml"}
	case "js-http":
		build["command"] = "npm run build"
		build["workdir"] = templateDir
		build["watch"] = []string{"src/**/*.js", "package.json"}
	case "python-http":
		build["command"] = "componentize-py -w spin-http componentize -o " + opts.Name + ".wasm app"
		build["workdir"] = templateDir
		build["watch"] = []string{"**/*.py"}
	}
	if len(build) > 0 {
		comp["build"] = build
	}

	// Create the template directory and files
	if err := createTemplateFiles(templateDir, opts.Template, opts.Name); err != nil {
		// Log error but continue - the component can still be added to manifest
		fmt.Fprintf(os.Stderr, "Warning: %v\n", err)
	}

	return comp
}

func createFromRegistry(opts *AddComponentOptions) map[string]interface{} {
	comp := make(map[string]interface{})
	comp["id"] = opts.Name

	// Parse registry string (format: registry/package:version)
	parts := strings.Split(opts.Registry, "/")
	if len(parts) != 2 {
		color.Yellow("⚠ Invalid registry format. Using as-is.")
		comp["source"] = opts.Registry
		return comp
	}

	registry := parts[0]
	remainder := parts[1]

	// Split package:version
	packageParts := strings.Split(remainder, ":")
	if len(packageParts) != 2 {
		color.Yellow("⚠ Invalid package format. Using as-is.")
		comp["source"] = opts.Registry
		return comp
	}

	source := make(map[string]interface{})
	source["registry"] = registry
	source["package"] = packageParts[0]
	source["version"] = packageParts[1]
	comp["source"] = source

	return comp
}

func createFromLocal(opts *AddComponentOptions) map[string]interface{} {
	comp := make(map[string]interface{})
	comp["id"] = opts.Name
	comp["source"] = opts.Source

	// Check if it needs build config
	info, err := os.Stat(opts.Source)
	if err == nil && (info.IsDir() || !strings.HasSuffix(opts.Source, ".wasm")) {
		// It's source code, needs build
		if opts.Build != "" {
			build := make(map[string]interface{})
			build["command"] = opts.Build
			comp["build"] = build
		}
	}

	return comp
}

func createInteractive(opts *AddComponentOptions) (map[string]interface{}, error) {
	// Ask for source type
	sourceType := ""
	sourcePrompt := &survey.Select{
		Message: "Component source:",
		Options: []string{"Local path", "Registry", "Create from template"},
	}
	if err := survey.AskOne(sourcePrompt, &sourceType); err != nil {
		return nil, err
	}

	switch sourceType {
	case "Local path":
		pathPrompt := &survey.Input{
			Message: "Path to component:",
			Default: fmt.Sprintf("./components/%s", opts.Name),
		}
		if err := survey.AskOne(pathPrompt, &opts.Source); err != nil {
			return nil, err
		}
		return createFromLocal(opts), nil

	case "Registry":
		regPrompt := &survey.Input{
			Message: "Registry source (registry/package:version):",
		}
		if err := survey.AskOne(regPrompt, &opts.Registry); err != nil {
			return nil, err
		}
		return createFromRegistry(opts), nil

	case "Create from template":
		templatePrompt := &survey.Select{
			Message: "Select template:",
			Options: []string{"go-http", "rust-wasm", "js-http", "python-http"},
		}
		if err := survey.AskOne(templatePrompt, &opts.Template); err != nil {
			return nil, err
		}
		return createFromTemplate(opts), nil
	}

	return nil, fmt.Errorf("invalid source type")
}

func createTemplateFiles(dir, template, name string) error {
	// Create directory
	if err := os.MkdirAll(dir, 0750); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	switch template {
	case "go-http":
		// Create main.go
		mainGo := fmt.Sprintf(`package main

import (
	"net/http"
	spinhttp "github.com/fermyon/spin-go-sdk/http"
)

func init() {
	spinhttp.Handle(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("Hello from %s!"))
	})
}

func main() {}
`, name)
		if err := os.WriteFile(filepath.Join(dir, "main.go"), []byte(mainGo), 0600); err != nil {
			return fmt.Errorf("failed to write main.go: %w", err)
		}

		// Create go.mod
		goMod := fmt.Sprintf(`module github.com/example/%s

go 1.24

require github.com/fermyon/spin-go-sdk v0.2.0
`, name)
		if err := os.WriteFile(filepath.Join(dir, "go.mod"), []byte(goMod), 0600); err != nil {
			return fmt.Errorf("failed to write go.mod: %w", err)
		}

	case "rust-wasm":
		// Create Cargo.toml
		cargoToml := fmt.Sprintf(`[package]
name = "%s"
version = "0.1.0"
edition = "2021"

[dependencies]
spin-sdk = "3.0"

[lib]
crate-type = ["cdylib"]
`, name)
		if err := os.WriteFile(filepath.Join(dir, "Cargo.toml"), []byte(cargoToml), 0600); err != nil {
			return fmt.Errorf("failed to write Cargo.toml: %w", err)
		}

		// Create src/lib.rs
		if err := os.MkdirAll(filepath.Join(dir, "src"), 0750); err != nil {
			return fmt.Errorf("failed to create src directory: %w", err)
		}
		libRs := `use spin_sdk::http::{IntoResponse, Request, Response};

#[spin_sdk::http_component]
fn handle_request(_req: Request) -> anyhow::Result<impl IntoResponse> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello from Rust!")
        .build())
}
`
		if err := os.WriteFile(filepath.Join(dir, "src", "lib.rs"), []byte(libRs), 0600); err != nil {
			return fmt.Errorf("failed to write lib.rs: %w", err)
		}

		// Add other templates as needed
	}
	return nil
}

func loadManifest(path string) (map[interface{}]interface{}, error) {
	// Clean the path to prevent directory traversal
	path = filepath.Clean(path)
	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			// Create a new manifest if it doesn't exist
			return map[interface{}]interface{}{
				"name":       "app",
				"version":    "0.1.0",
				"components": []interface{}{},
				"access":     "public",
			}, nil
		}
		return nil, err
	}

	var manifest map[interface{}]interface{}
	if err := yaml.Unmarshal(data, &manifest); err != nil {
		return nil, fmt.Errorf("failed to parse manifest: %w", err)
	}

	return manifest, nil
}

func saveManifest(path string, manifest map[interface{}]interface{}) error {
	data, err := yaml.Marshal(manifest)
	if err != nil {
		return fmt.Errorf("failed to marshal manifest: %w", err)
	}
	return os.WriteFile(path, data, 0600)
}
