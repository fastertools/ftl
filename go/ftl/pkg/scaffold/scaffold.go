package scaffold

import (
	"bytes"
	_ "embed"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/shared/config"
)

//go:embed templates.cue
var templatesCUE string

// Scaffolder handles component generation using CUE templates
type Scaffolder struct {
	ctx       *cue.Context
	templates cue.Value
}

// NewScaffolder creates a new scaffolder with embedded templates
func NewScaffolder() (*Scaffolder, error) {
	ctx := cuecontext.New()

	// Compile the embedded CUE templates
	templates := ctx.CompileString(templatesCUE)
	if templates.Err() != nil {
		return nil, fmt.Errorf("failed to compile templates: %w", templates.Err())
	}

	return &Scaffolder{
		ctx:       ctx,
		templates: templates,
	}, nil
}

// GenerateComponent creates a new component from templates
func (s *Scaffolder) GenerateComponent(name, language string) error {
	// Validate inputs
	if err := s.validateInputs(name, language); err != nil {
		return err
	}

	// Create component instance from template
	component, err := s.createComponentInstance(name, language)
	if err != nil {
		return fmt.Errorf("failed to create component instance: %w", err)
	}

	// Generate files
	if err := s.generateFiles(name, component); err != nil {
		return fmt.Errorf("failed to generate files: %w", err)
	}

	// Update ftl.yaml
	if err := s.updateFTLConfig(name, component); err != nil {
		return fmt.Errorf("failed to update ftl.yaml: %w", err)
	}

	return nil
}

// validateInputs checks that the name and language are valid
func (s *Scaffolder) validateInputs(name, language string) error {
	// Validate component name
	nameValidator := s.ctx.CompileString(fmt.Sprintf(`
		import "regexp"
		name: %q
		valid: regexp.Match("^[a-z][a-z0-9-]*$", name)
	`, name))

	valid := nameValidator.LookupPath(cue.ParsePath("valid"))
	if v, _ := valid.Bool(); !v {
		return fmt.Errorf("invalid component name '%s': must be lowercase with hyphens (e.g., my-tool)", name)
	}

	// Validate language
	validLanguages := []string{"rust", "typescript", "python", "go"}
	found := false
	for _, lang := range validLanguages {
		if lang == language {
			found = true
			break
		}
	}
	if !found {
		return fmt.Errorf("invalid language '%s': must be one of %v", language, validLanguages)
	}

	return nil
}

// createComponentInstance creates a CUE value for the component
func (s *Scaffolder) createComponentInstance(name, language string) (cue.Value, error) {
	// Create the component configuration with the templates
	componentDef := fmt.Sprintf(`
		%s
		
		component: #Templates[%q] & {
			name: %q
		}
	`, templatesCUE, language, name)

	// Compile the full definition
	instance := s.ctx.CompileString(componentDef)
	unified := instance

	if err := unified.Err(); err != nil {
		return cue.Value{}, fmt.Errorf("failed to create component: %w", err)
	}

	// Extract the component value
	component := unified.LookupPath(cue.ParsePath("component"))
	if !component.Exists() {
		return cue.Value{}, fmt.Errorf("component not found in template")
	}

	return component, nil
}

// generateFiles creates all the component files
func (s *Scaffolder) generateFiles(name string, component cue.Value) error {
	// Create component directory
	if err := os.MkdirAll(name, 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	// Get language for special handling
	language, _ := component.LookupPath(cue.ParsePath("language")).String()

	// Prepare name variations for template substitution
	nameUnderscore := strings.ReplaceAll(name, "-", "_")

	// Extract files map
	files := component.LookupPath(cue.ParsePath("files"))
	if !files.Exists() {
		return fmt.Errorf("no files defined in template")
	}

	// Iterate over files and create them
	iter, err := files.Fields()
	if err != nil {
		return fmt.Errorf("failed to iterate files: %w", err)
	}

	for iter.Next() {
		path := iter.Selector().Unquoted()
		content := iter.Value()

		// Get content as string
		contentStr, err := content.String()
		if err != nil {
			return fmt.Errorf("failed to get content for %s: %w", path, err)
		}

		// Apply language-specific substitutions
		if language == "rust" {
			// Rust specific: In Cargo.toml, package names must use underscores
			if path == "Cargo.toml" {
				contentStr = strings.ReplaceAll(contentStr, `name = "`+name+`"`, `name = "`+nameUnderscore+`"`)
			}
			// In Makefile, the built WASM file uses underscores
			if path == "Makefile" {
				contentStr = strings.ReplaceAll(contentStr, "/"+name+".wasm", "/"+nameUnderscore+".wasm")
				contentStr = strings.ReplaceAll(contentStr, " "+name+".wasm", " "+nameUnderscore+".wasm")
			}
		}

		// Create full path
		fullPath := filepath.Join(name, path)

		// Create directory if needed
		dir := filepath.Dir(fullPath)
		if dir != "." && dir != name {
			if err := os.MkdirAll(dir, 0755); err != nil {
				return fmt.Errorf("failed to create directory %s: %w", dir, err)
			}
		}

		// Write file
		if err := os.WriteFile(fullPath, []byte(contentStr), 0644); err != nil {
			return fmt.Errorf("failed to write file %s: %w", fullPath, err)
		}
	}

	return nil
}

// updateFTLConfig adds the new component to ftl.yaml
func (s *Scaffolder) updateFTLConfig(name string, component cue.Value) error {
	// Check if ftl.yaml exists
	configPath := "ftl.yaml"
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		return fmt.Errorf("ftl.yaml not found - not in an FTL project directory")
	}

	// Read existing config
	data, err := os.ReadFile(configPath)
	if err != nil {
		return fmt.Errorf("failed to read ftl.yaml: %w", err)
	}

	var cfg config.FTLConfig
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return fmt.Errorf("failed to parse ftl.yaml: %w", err)
	}

	// Extract build configuration from component
	build := component.LookupPath(cue.ParsePath("build"))

	command, _ := build.LookupPath(cue.ParsePath("command")).String()

	// Extract watch patterns
	watchIter, _ := build.LookupPath(cue.ParsePath("watch")).List()
	var watchPatterns []string
	for watchIter.Next() {
		pattern, _ := watchIter.Value().String()
		watchPatterns = append(watchPatterns, pattern)
	}

	// Extract language to determine WASM output path
	language, _ := component.LookupPath(cue.ParsePath("language")).String()
	wasmPath := s.getWasmPath(name, language)

	// Create new component config
	newComponent := config.ComponentConfig{
		ID:     name,
		Source: wasmPath,
		Build: &config.BuildConfig{
			Command: command,
			Workdir: name,
			Watch:   watchPatterns,
		},
	}

	// Check for duplicate
	for _, comp := range cfg.Components {
		if comp.ID == name {
			return fmt.Errorf("component '%s' already exists", name)
		}
	}

	// Add component
	cfg.Components = append(cfg.Components, newComponent)

	// Write back
	var buf bytes.Buffer
	encoder := yaml.NewEncoder(&buf)
	encoder.SetIndent(2)
	if err := encoder.Encode(&cfg); err != nil {
		return fmt.Errorf("failed to encode config: %w", err)
	}

	if err := os.WriteFile(configPath, buf.Bytes(), 0644); err != nil {
		return fmt.Errorf("failed to write ftl.yaml: %w", err)
	}

	return nil
}

// getWasmPath returns the WASM output path for a component
func (s *Scaffolder) getWasmPath(name, language string) string {
	switch language {
	case "rust":
		// Rust uses underscores in the binary name
		binaryName := strings.ReplaceAll(name, "-", "_")
		return filepath.Join(name, binaryName+".wasm")
	case "typescript":
		return filepath.Join(name, "dist", name+".wasm")
	case "python":
		return filepath.Join(name, "app.wasm")
	case "go":
		return filepath.Join(name, "main.wasm")
	default:
		return filepath.Join(name, name+".wasm")
	}
}

// ListLanguages returns the available languages
func (s *Scaffolder) ListLanguages() []string {
	return []string{"rust", "typescript", "python", "go"}
}

// ValidateComponentName checks if a component name is valid
func ValidateComponentName(name string) error {
	if name == "" {
		return fmt.Errorf("component name cannot be empty")
	}

	// Check for valid characters (lowercase, numbers, hyphens)
	for i, c := range name {
		if !((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '-') {
			return fmt.Errorf("component name must contain only lowercase letters, numbers, and hyphens")
		}

		// First character must be a letter
		if i == 0 && !(c >= 'a' && c <= 'z') {
			return fmt.Errorf("component name must start with a lowercase letter")
		}
	}

	// Check for leading/trailing or double hyphens
	if strings.HasPrefix(name, "-") || strings.HasSuffix(name, "-") || strings.Contains(name, "--") {
		return fmt.Errorf("component name cannot start/end with or contain double hyphens")
	}

	return nil
}
