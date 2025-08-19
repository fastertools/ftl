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
	ctx    *cue.Context
	schema cue.Value
}

// NewSynthesizer creates a new CUE-first synthesizer
func NewSynthesizer() *Synthesizer {
	ctx := cuecontext.New()
	// Compile the FTL patterns schema
	schema := ctx.CompileString(ftlPatterns, cue.Filename("patterns.cue"))
	return &Synthesizer{
		ctx:    ctx,
		schema: schema,
	}
}

// GetSchema returns the compiled FTL schema for validation
func (s *Synthesizer) GetSchema() cue.Value {
	return s.schema.LookupPath(cue.ParsePath("#FTLApplication"))
}

// GetPatterns returns the raw FTL patterns CUE source
// This is exported for use by the validation package
func GetPatterns() string {
	return ftlPatterns
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


// SynthesizeFromStruct takes a Go struct and produces a Spin manifest
// This is used by the CDK to transform its structs
func (s *Synthesizer) SynthesizeFromStruct(data interface{}) (string, error) {
	// Encode the struct to CUE
	value := s.ctx.Encode(data)
	if value.Err() != nil {
		return "", fmt.Errorf("failed to encode struct to CUE: %w", value.Err())
	}
	
	return s.synthesizeFromValue(value)
}

// synthesizeFromValue takes a CUE value and transforms it to a Spin manifest
func (s *Synthesizer) synthesizeFromValue(inputValue cue.Value) (string, error) {
	// Build a complete program with patterns and bridge
	program := fmt.Sprintf(`
%s

// Input data from YAML/JSON/CUE
inputData: _

// Apply the transformation defined in patterns.cue
transform: #InputTransform & {
	input: inputData
}

// Extract the manifest
manifest: transform.manifest
`, ftlPatterns)

	// Compile the complete program
	value := s.ctx.CompileString(program, cue.Filename("transform.cue"))
	if value.Err() != nil {
		return "", fmt.Errorf("failed to compile transformation: %w", value.Err())
	}

	// Fill in the input data
	filled := value.FillPath(cue.ParsePath("inputData"), inputValue)
	if filled.Err() != nil {
		return "", fmt.Errorf("failed to fill input data: %w", filled.Err())
	}

	// Extract the manifest
	manifestValue := filled.LookupPath(cue.ParsePath("manifest"))
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
