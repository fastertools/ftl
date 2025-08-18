package synthesis

import (
	"bytes"
	_ "embed"
	"fmt"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	cuejson "cuelang.org/go/encoding/json"
	"cuelang.org/go/encoding/toml"
	"cuelang.org/go/encoding/yaml"
)

//go:embed patterns.cue
var ftlPatterns string

// Synthesizer is a pure CUE-based synthesizer
type Synthesizer struct {
	ctx *cue.Context
}

// NewSynthesizer creates a new CUE-first synthesizer
func NewSynthesizer() *Synthesizer {
	return &Synthesizer{
		ctx: cuecontext.New(),
	}
}

// SynthesizeYAML takes YAML input and produces a Spin manifest
func (s *Synthesizer) SynthesizeYAML(yamlData []byte) (string, error) {
	// Extract YAML directly into CUE
	file, err := yaml.Extract("input.yaml", yamlData)
	if err != nil {
		return "", fmt.Errorf("failed to extract YAML: %w", err)
	}

	value := s.ctx.BuildFile(file)
	if value.Err() != nil {
		return "", fmt.Errorf("failed to build CUE from YAML: %w", value.Err())
	}

	return s.synthesizeFromValue(value)
}

// SynthesizeJSON takes JSON input and produces a Spin manifest
func (s *Synthesizer) SynthesizeJSON(jsonData []byte) (string, error) {
	// Extract JSON directly into CUE
	decoder := cuejson.NewDecoder(nil, "input.json", bytes.NewReader(jsonData))
	expr, err := decoder.Extract()
	if err != nil {
		return "", fmt.Errorf("failed to extract JSON: %w", err)
	}

	value := s.ctx.BuildExpr(expr)
	if value.Err() != nil {
		return "", fmt.Errorf("failed to build CUE from JSON: %w", value.Err())
	}

	return s.synthesizeFromValue(value)
}

// SynthesizeCUE takes CUE source and produces a Spin manifest
func (s *Synthesizer) SynthesizeCUE(cueSource string) (string, error) {
	value := s.ctx.CompileString(cueSource)
	if value.Err() != nil {
		return "", fmt.Errorf("failed to compile CUE: %w", value.Err())
	}

	return s.synthesizeFromValue(value)
}

// encodeToTOML encodes a CUE value to TOML
func (s *Synthesizer) encodeToTOML(value cue.Value) (string, error) {
	var buf bytes.Buffer
	encoder := toml.NewEncoder(&buf)
	if err := encoder.Encode(value); err != nil {
		return "", fmt.Errorf("failed to encode to TOML: %w", err)
	}
	return buf.String(), nil
}

// synthesizeFromValue takes a CUE value and transforms it to a Spin manifest
func (s *Synthesizer) synthesizeFromValue(inputValue cue.Value) (string, error) {
	// Debug: print the input value
	// fmt.Fprintf(os.Stderr, "DEBUG: Input value: %v\n", inputValue)

	// Create the complete transformation program
	// Note: We build the app directly from inputData without intermediate schema
	program := fmt.Sprintf(`
%s

inputData: _

// Build the FTL app structure directly from input
_ftlApp: {
	name:        inputData.application.name
	version:     inputData.application.version | *"0.1.0"
	description: inputData.application.description | *""
	
	components: [ for comp in inputData.components if inputData.components != _|_ {
		id:     comp.id
		source: comp.source
		if comp.build != _|_ {
			build: comp.build
		}
		if comp.variables != _|_ {
			variables: comp.variables
		}
	}]
	
	// Pass through access mode, default to public
	if inputData.access != _|_ {
		access: inputData.access
	}
	if inputData.access == _|_ {
		access: "public"
	}
	
	// Pass through auth configuration if present
	if inputData.auth != _|_ {
		auth: inputData.auth
	}
}

// Validate against schema
app: #FTLApplication & _ftlApp

// Apply transformation
_transform: #TransformToSpin & {
	input: app
}

// Extract the final manifest
manifest: _transform.output
`, ftlPatterns)

	// Compile the complete program
	value := s.ctx.CompileString(program)
	if value.Err() != nil {
		return "", fmt.Errorf("failed to compile CUE: %w", value.Err())
	}

	// Fill in the input data
	value = value.FillPath(cue.ParsePath("inputData"), inputValue)
	if value.Err() != nil {
		return "", fmt.Errorf("failed to fill input data: %w", value.Err())
	}

	// Extract the manifest field
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
