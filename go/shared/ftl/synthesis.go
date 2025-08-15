package ftl

import (
	"bytes"
	_ "embed"
	"encoding/json"
	"fmt"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	cuejson "cuelang.org/go/encoding/json"
	"cuelang.org/go/encoding/toml"
	"gopkg.in/yaml.v3"
)

// Embed the CUE patterns directly in the package
//go:embed patterns.cue
var cuePatterns string

// SpinManifest represents the synthesized Spin manifest
type SpinManifest struct {
	SpinManifestVersion int                              `toml:"spin_manifest_version"`
	Application         SpinApplication                  `toml:"application"`
	Component           map[string]SpinComponent        `toml:"component"`
	Trigger             SpinTrigger                      `toml:"trigger"`
	Variables           map[string]SpinVariable         `toml:"variables,omitempty"`
}

// SpinApplication represents the application section of a Spin manifest
type SpinApplication struct {
	Name        string `toml:"name"`
	Version     string `toml:"version,omitempty"`
	Description string `toml:"description,omitempty"`
}

// SpinComponent represents a component in the Spin manifest
type SpinComponent struct {
	Source               interface{}               `toml:"source"`
	Build                *BuildConfig              `toml:"build,omitempty"`
	Variables            map[string]string         `toml:"variables,omitempty"`
	AllowedOutboundHosts []string                  `toml:"allowed_outbound_hosts,omitempty"`
}

// SpinTrigger represents the trigger configuration
type SpinTrigger struct {
	HTTP []SpinHTTPTrigger `toml:"http"`
}

// SpinHTTPTrigger represents an HTTP trigger
type SpinHTTPTrigger struct {
	Route     interface{} `toml:"route"`
	Component string      `toml:"component"`
}

// SpinVariable represents a Spin variable
type SpinVariable struct {
	Default  string `toml:"default,omitempty"`
	Required bool   `toml:"required,omitempty"`
	Secret   bool   `toml:"secret,omitempty"`
}

// Synthesizer handles the transformation of FTL applications to Spin manifests
type Synthesizer struct {
	ctx *cue.Context
}

// NewSynthesizer creates a new synthesizer
func NewSynthesizer() *Synthesizer {
	return &Synthesizer{
		ctx: cuecontext.New(),
	}
}

// SynthesizeToSpin transforms an FTL application to a Spin manifest
func (s *Synthesizer) SynthesizeToSpin(app *Application) (*SpinManifest, error) {
	// Set defaults
	app.SetDefaults()
	
	// Validate
	if err := app.Validate(); err != nil {
		return nil, fmt.Errorf("invalid application: %w", err)
	}
	
	// Convert to CUE value and synthesize
	appJSON, err := json.Marshal(app)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal application: %w", err)
	}
	
	// Create the synthesis program
	program := fmt.Sprintf(`
%s

inputApp: _

// Apply transformation
_transform: #TransformToSpin & {
	input: inputApp
}

// Extract the final manifest
manifest: _transform.output
`, cuePatterns)
	
	// Compile the program
	value := s.ctx.CompileString(program)
	if value.Err() != nil {
		return nil, fmt.Errorf("failed to compile CUE: %w", value.Err())
	}
	
	// Parse the application JSON into CUE
	decoder := cuejson.NewDecoder(nil, "app.json", bytes.NewReader(appJSON))
	expr, err := decoder.Extract()
	if err != nil {
		return nil, fmt.Errorf("failed to extract JSON: %w", err)
	}
	
	appValue := s.ctx.BuildExpr(expr)
	if appValue.Err() != nil {
		return nil, fmt.Errorf("failed to build CUE from JSON: %w", appValue.Err())
	}
	
	// Fill in the input application
	value = value.FillPath(cue.ParsePath("inputApp"), appValue)
	if value.Err() != nil {
		return nil, fmt.Errorf("failed to fill input app: %w", value.Err())
	}
	
	// Extract the manifest
	manifestValue := value.LookupPath(cue.ParsePath("manifest"))
	if manifestValue.Err() != nil {
		return nil, fmt.Errorf("failed to extract manifest: %w", manifestValue.Err())
	}
	
	// Convert to SpinManifest struct
	var manifest SpinManifest
	if err := manifestValue.Decode(&manifest); err != nil {
		return nil, fmt.Errorf("failed to decode manifest: %w", err)
	}
	
	return &manifest, nil
}

// SynthesizeToTOML transforms an FTL application to a Spin TOML manifest string
func (s *Synthesizer) SynthesizeToTOML(app *Application) (string, error) {
	// Set defaults and validate
	app.SetDefaults()
	if err := app.Validate(); err != nil {
		return "", fmt.Errorf("invalid application: %w", err)
	}
	
	// Convert to CUE value and synthesize
	appJSON, err := json.Marshal(app)
	if err != nil {
		return "", fmt.Errorf("failed to marshal application: %w", err)
	}
	
	// Create the synthesis program
	program := fmt.Sprintf(`
%s

inputApp: _

// Apply transformation
_transform: #TransformToSpin & {
	input: inputApp
}

// Extract the final manifest
manifest: _transform.output
`, cuePatterns)
	
	// Compile the program
	value := s.ctx.CompileString(program)
	if value.Err() != nil {
		return "", fmt.Errorf("failed to compile CUE: %w", value.Err())
	}
	
	// Parse the application JSON into CUE
	decoder := cuejson.NewDecoder(nil, "app.json", bytes.NewReader(appJSON))
	expr, err := decoder.Extract()
	if err != nil {
		return "", fmt.Errorf("failed to extract JSON: %w", err)
	}
	
	appValue := s.ctx.BuildExpr(expr)
	if appValue.Err() != nil {
		return "", fmt.Errorf("failed to build CUE from JSON: %w", appValue.Err())
	}
	
	// Fill in the input application
	value = value.FillPath(cue.ParsePath("inputApp"), appValue)
	if value.Err() != nil {
		return "", fmt.Errorf("failed to fill input app: %w", value.Err())
	}
	
	// Extract the manifest
	manifestValue := value.LookupPath(cue.ParsePath("manifest"))
	if manifestValue.Err() != nil {
		return "", fmt.Errorf("failed to extract manifest: %w", manifestValue.Err())
	}
	
	// Encode to TOML
	var buf bytes.Buffer
	encoder := toml.NewEncoder(&buf)
	if err := encoder.Encode(manifestValue); err != nil {
		return "", fmt.Errorf("failed to encode to TOML: %w", err)
	}
	
	return buf.String(), nil
}

// SynthesizeFromYAML takes YAML input and produces a Spin manifest
func (s *Synthesizer) SynthesizeFromYAML(yamlData []byte) (*SpinManifest, error) {
	var app Application
	if err := yaml.Unmarshal(yamlData, &app); err != nil {
		return nil, fmt.Errorf("failed to parse YAML: %w", err)
	}
	
	return s.SynthesizeToSpin(&app)
}

// SynthesizeFromJSON takes JSON input and produces a Spin manifest  
func (s *Synthesizer) SynthesizeFromJSON(jsonData []byte) (*SpinManifest, error) {
	var app Application
	if err := json.Unmarshal(jsonData, &app); err != nil {
		return nil, fmt.Errorf("failed to parse JSON: %w", err)
	}
	
	return s.SynthesizeToSpin(&app)
}