//go:generate cp ../../.release-please-manifest.json manifest.json
package scaffold

import (
	"bytes"
	_ "embed"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl/validation"
)

//go:embed templates.cue
var templatesCUE string

//go:embed manifest.json
var manifestJSON string

// Versions represents all versions used in templates
type Versions struct {
	FTLCli     string `json:"ftl"`
	SDK        SDKVersions
	Components ComponentVersions
}

// SDKVersions represents the SDK versions for each language
type SDKVersions struct {
	Go         string
	Rust       string
	Python     string
	TypeScript string
}

// ComponentVersions represents the versions for WASM components
type ComponentVersions struct {
	MCPAuthorizer string
	MCPGateway    string
}

// Scaffolder handles component generation using CUE templates
type Scaffolder struct {
	ctx       *cue.Context
	templates cue.Value
	versions  Versions
}

// NewScaffolder creates a new scaffolder with embedded templates
func NewScaffolder() (*Scaffolder, error) {
	ctx := cuecontext.New()

	// Parse the release-please manifest
	var manifest map[string]string
	if err := json.Unmarshal([]byte(manifestJSON), &manifest); err != nil {
		return nil, fmt.Errorf("failed to parse release manifest: %w", err)
	}

	// Extract versions from the manifest
	versions := Versions{
		FTLCli: manifest["."],
		SDK: SDKVersions{
			Go:         manifest["sdk/go"],
			Rust:       manifest["sdk/rust"],
			Python:     manifest["sdk/python"],
			TypeScript: manifest["sdk/typescript"],
		},
		Components: ComponentVersions{
			MCPAuthorizer: manifest["components/mcp-authorizer"],
			MCPGateway:    manifest["components/mcp-gateway"],
		},
	}

	// Create templates with versions
	templateWithVersions := fmt.Sprintf(`
%s

_versions: {
	ftl_cli:    %q
	go:         %q
	rust:       %q
	python:     %q
	typescript: %q
}
`, templatesCUE, versions.FTLCli, versions.SDK.Go, versions.SDK.Rust, versions.SDK.Python, versions.SDK.TypeScript)

	templates := ctx.CompileString(templateWithVersions)
	if templates.Err() != nil {
		return nil, fmt.Errorf("failed to compile templates: %w", templates.Err())
	}

	return &Scaffolder{
		ctx:       ctx,
		templates: templates,
		versions:  versions,
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

// GenerateProject creates a new FTL project from templates
func (s *Scaffolder) GenerateProject(projectDir, name, description, format string) error {
	// Validate format
	validFormats := []string{"yaml", "json", "cue", "go"}
	valid := false
	for _, f := range validFormats {
		if f == format {
			valid = true
			break
		}
	}
	if !valid {
		return fmt.Errorf("invalid format: %s", format)
	}

	// Get project template
	templatePath := fmt.Sprintf("#ProjectTemplates.%s", format)
	templateValue := s.templates.LookupPath(cue.ParsePath(templatePath))
	if templateValue.Err() != nil {
		return fmt.Errorf("failed to get project template: %w", templateValue.Err())
	}

	// Fill in template with values
	filled := templateValue.FillPath(cue.ParsePath("name"), name)
	filled = filled.FillPath(cue.ParsePath("description"), description)
	if filled.Err() != nil {
		return fmt.Errorf("failed to fill template: %w", filled.Err())
	}

	// Extract files
	filesValue := filled.LookupPath(cue.ParsePath("files"))
	if filesValue.Err() != nil {
		return fmt.Errorf("failed to extract files: %w", filesValue.Err())
	}

	// Generate each file
	iter, err := filesValue.Fields()
	if err != nil {
		return fmt.Errorf("failed to iterate files: %w", err)
	}

	for iter.Next() {
		filename := iter.Selector().Unquoted()
		content, err := iter.Value().String()
		if err != nil {
			return fmt.Errorf("failed to extract content for %s: %w", filename, err)
		}

		filePath := filepath.Join(projectDir, filename)
		if err := os.WriteFile(filePath, []byte(content), 0600); err != nil {
			return fmt.Errorf("failed to write %s: %w", filename, err)
		}
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
	componentDef := fmt.Sprintf(`
%s

_versions: {
	ftl_cli:    %q
	go:         %q
	rust:       %q
	python:     %q
	typescript: %q
}

component: #Templates[%q] & {
	name: %q
}
`, templatesCUE, s.versions.FTLCli, s.versions.SDK.Go, s.versions.SDK.Rust, s.versions.SDK.Python, s.versions.SDK.TypeScript, language, name)

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
	if err := os.MkdirAll(name, 0750); err != nil {
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
			if err := os.MkdirAll(dir, 0750); err != nil {
				return fmt.Errorf("failed to create directory %s: %w", dir, err)
			}
		}

		// Write file
		if err := os.WriteFile(fullPath, []byte(contentStr), 0600); err != nil {
			return fmt.Errorf("failed to write file %s: %w", fullPath, err)
		}
	}

	return nil
}

// updateFTLConfig adds the new component to ftl.yaml or ftl.json
func (s *Scaffolder) updateFTLConfig(name string, component cue.Value) error {
	// Detect configuration format
	format, configPath, err := s.detectConfigFormat()
	if err != nil {
		return err
	}

	// Handle unsupported formats with helpful messages
	if format == "go" {
		// Extract build configuration for the helpful message
		language, _ := component.LookupPath(cue.ParsePath("language")).String()
		wasmPath := s.getWasmPath(name, language)

		return fmt.Errorf("go-based configurations require manual component registration - "+
			"add this to your main.go: "+
			"app.AddComponent(\"%s\")."+
			"FromLocal(\"./%s\")."+
			"WithBuild(\"cd %s && make build\")."+
			"Build()", name, wasmPath, name)
	}

	if format == "cue" {
		return fmt.Errorf("cue configurations require manual component registration - "+
			"add component with id=%s source=./%s/%s.wasm build.command='make build' build.workdir=%s to your app.cue components array",
			name, name, name, name)
	}

	// Read existing config
	// Clean the path to prevent directory traversal
	configPath = filepath.Clean(configPath)
	data, err := os.ReadFile(configPath)
	if err != nil {
		return fmt.Errorf("failed to read %s: %w", configPath, err)
	}

	// Parse and validate the config
	v := validation.New()
	var validatedValue cue.Value
	var manifest *validation.Application

	// Parse based on format
	switch format {
	case "yaml":
		validatedValue, err = v.ValidateYAML(data)
		if err != nil {
			return fmt.Errorf("failed to validate %s: %w", configPath, err)
		}
		manifest, err = validation.ExtractApplication(validatedValue)
		if err != nil {
			return fmt.Errorf("failed to extract application: %w", err)
		}
	case "json":
		validatedValue, err = v.ValidateJSON(data)
		if err != nil {
			return fmt.Errorf("failed to validate %s: %w", configPath, err)
		}
		manifest, err = validation.ExtractApplication(validatedValue)
		if err != nil {
			return fmt.Errorf("failed to extract application: %w", err)
		}
	default:
		return fmt.Errorf("unsupported configuration format: %s", format)
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
	newComponent := &validation.Component{
		ID:     name,
		Source: &validation.LocalSource{Path: wasmPath},
		Build: &validation.BuildConfig{
			Command: command,
			Workdir: name,
			Watch:   watchPatterns,
		},
	}

	// Check for duplicate
	for _, comp := range manifest.Components {
		if comp.ID == name {
			return fmt.Errorf("component '%s' already exists", name)
		}
	}

	// Add component
	manifest.Components = append(manifest.Components, newComponent)

	// Convert to flat structure for YAML/JSON
	flatConfig := convertToFlatStructure(manifest)

	// Write back based on format
	var output []byte
	switch format {
	case "yaml":
		var buf bytes.Buffer
		encoder := yaml.NewEncoder(&buf)
		encoder.SetIndent(2)
		if err := encoder.Encode(flatConfig); err != nil {
			return fmt.Errorf("failed to encode config: %w", err)
		}
		output = buf.Bytes()
	case "json":
		var err error
		output, err = json.MarshalIndent(flatConfig, "", "  ")
		if err != nil {
			return fmt.Errorf("failed to encode config: %w", err)
		}
		// Add trailing newline for consistency
		output = append(output, '\n')
	}

	if err := os.WriteFile(configPath, output, 0600); err != nil {
		return fmt.Errorf("failed to write %s: %w", configPath, err)
	}

	return nil
}

// detectConfigFormat detects which configuration format is being used
func (s *Scaffolder) detectConfigFormat() (string, string, error) {
	// Check for YAML first (most common)
	if _, err := os.Stat("ftl.yaml"); err == nil {
		return "yaml", "ftl.yaml", nil
	}

	// Check for JSON
	if _, err := os.Stat("ftl.json"); err == nil {
		return "json", "ftl.json", nil
	}

	// Check for CUE
	if _, err := os.Stat("app.cue"); err == nil {
		return "cue", "app.cue", nil
	}

	// Check for Go
	if _, err := os.Stat("main.go"); err == nil {
		// Double-check it's actually an FTL Go config by looking for the CDK import
		data, err := os.ReadFile("main.go")
		if err == nil && strings.Contains(string(data), "synthesis.NewCDK") {
			return "go", "main.go", nil
		}
	}

	return "", "", fmt.Errorf("no FTL configuration found - not in an FTL project directory.\n" +
		"Run 'ftl init' to create a new project")
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

// convertToFlatStructure converts validation types to flat YAML/JSON structure
func convertToFlatStructure(app *validation.Application) map[string]interface{} {
	result := make(map[string]interface{})

	// Add top-level fields
	result["name"] = app.Name
	if app.Version != "" {
		result["version"] = app.Version
	}
	if app.Description != "" {
		result["description"] = app.Description
	}
	if app.Access != "" {
		result["access"] = app.Access
	}

	// Convert auth config
	if app.Auth != nil {
		auth := make(map[string]interface{})
		if app.Auth.JWTIssuer != "" {
			auth["jwt_issuer"] = app.Auth.JWTIssuer
		}
		if app.Auth.JWTAudience != "" {
			auth["jwt_audience"] = app.Auth.JWTAudience
		}
		if len(auth) > 0 {
			result["auth"] = auth
		}
	}

	// Convert components
	if len(app.Components) > 0 {
		components := make([]map[string]interface{}, 0, len(app.Components))
		for _, comp := range app.Components {
			c := make(map[string]interface{})
			c["id"] = comp.ID

			// Convert source
			switch src := comp.Source.(type) {
			case *validation.LocalSource:
				c["source"] = src.Path
			case *validation.RegistrySource:
				source := make(map[string]interface{})
				source["registry"] = src.Registry
				source["package"] = src.Package
				source["version"] = src.Version
				c["source"] = source
			}

			// Convert build config
			if comp.Build != nil {
				build := make(map[string]interface{})
				if comp.Build.Command != "" {
					build["command"] = comp.Build.Command
				}
				if comp.Build.Workdir != "" {
					build["workdir"] = comp.Build.Workdir
				}
				if len(comp.Build.Watch) > 0 {
					build["watch"] = comp.Build.Watch
				}
				if len(build) > 0 {
					c["build"] = build
				}
			}

			// Add variables
			if len(comp.Variables) > 0 {
				c["variables"] = comp.Variables
			}

			components = append(components, c)
		}
		result["components"] = components
	}

	// Add variables
	if len(app.Variables) > 0 {
		result["variables"] = app.Variables
	}

	return result
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
		if (c < 'a' || c > 'z') && (c < '0' || c > '9') && c != '-' {
			return fmt.Errorf("component name must contain only lowercase letters, numbers, and hyphens")
		}

		// First character must be a letter
		if i == 0 && (c < 'a' || c > 'z') {
			return fmt.Errorf("component name must start with a lowercase letter")
		}
	}

	// Check for leading/trailing or double hyphens
	if strings.HasPrefix(name, "-") || strings.HasSuffix(name, "-") || strings.Contains(name, "--") {
		return fmt.Errorf("component name cannot start/end with or contain double hyphens")
	}

	return nil
}
