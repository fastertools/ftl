package synth

import (
	"fmt"
	"strings"

	"cuelang.org/go/cue"
	"cuelang.org/go/cue/cuecontext"
	cueyaml "cuelang.org/go/encoding/yaml"
	"gopkg.in/yaml.v3"

	"github.com/fastertools/ftl-cli/go/spindl/internal/schema"
)

// Engine provides CUE-based synthesis capabilities
type Engine struct {
	ctx *cue.Context
}

// NewEngine creates a new synthesis engine
func NewEngine() *Engine {
	return &Engine{
		ctx: cuecontext.New(),
	}
}

// SynthesizeConfig takes a configuration and synthesizes it into a Spin manifest
func (e *Engine) SynthesizeConfig(configData []byte, format string) ([]byte, error) {
	// For now, use the simpler config-based synthesis without CUE validation
	// TODO: Re-enable CUE validation once schemas are properly configured
	
	// Parse YAML configuration directly
	if format != "yaml" && format != "yml" {
		return nil, fmt.Errorf("currently only YAML format is supported")
	}
	
	var cfg interface{}
	if err := yaml.Unmarshal(configData, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse YAML: %w", err)
	}
	
	// Use the config-based synthesis from config_synth.go
	tomlData, err := SynthesizeFromYAML(configData)
	if err != nil {
		return nil, fmt.Errorf("failed to synthesize: %w", err)
	}
	
	return tomlData, nil
}

// ValidateConfig validates a configuration against the schema
func (e *Engine) ValidateConfig(configData []byte, format string) error {
	// Parse input configuration
	inputValue, err := e.parseInput(configData, format)
	if err != nil {
		return fmt.Errorf("failed to parse input: %w", err)
	}
	
	// Build validation schema
	validationSchema, err := e.buildValidationSchema()
	if err != nil {
		return fmt.Errorf("failed to build validation schema: %w", err)
	}
	
	// Unify and validate
	unified := validationSchema.FillPath(cue.ParsePath("input"), inputValue)
	if err := unified.Validate(); err != nil {
		return fmt.Errorf("validation failed: %w", err)
	}
	
	return nil
}

// parseInput parses input data based on format
func (e *Engine) parseInput(data []byte, format string) (cue.Value, error) {
	switch strings.ToLower(format) {
	case "yaml", "yml":
		file, err := cueyaml.Extract("", data)
		if err != nil {
			return cue.Value{}, err
		}
		return e.ctx.BuildFile(file), nil
	case "json":
		return e.ctx.CompileBytes(data), nil
	default:
		file, err := cueyaml.Extract("", data)
		if err != nil {
			return cue.Value{}, err
		}
		return e.ctx.BuildFile(file), nil
	}
}

// buildSynthesisSchema creates the synthesis schema with embedded CUE definitions
func (e *Engine) buildSynthesisSchema() (cue.Value, error) {
	// Create synthesis transformation
	synthCue := fmt.Sprintf(`
package main

// Import embedded schemas
%s

%s

%s

// Input configuration
input: {}

// Create MCP application
app: #McpApplication & input

// Export synthesized manifest
output: app.manifest
`, 
		e.wrapSchema("core", schema.CoreManifestSchema),
		e.wrapSchema("core", schema.CoreRegistrySchema),
		e.wrapSchema("solutions", schema.MCPSolutionSchema),
	)
	
	value := e.ctx.CompileString(synthCue)
	if err := value.Err(); err != nil {
		return cue.Value{}, fmt.Errorf("failed to compile synthesis schema: %w", err)
	}
	
	return value, nil
}

// buildValidationSchema creates the validation schema
func (e *Engine) buildValidationSchema() (cue.Value, error) {
	// Create validation schema
	validationCue := fmt.Sprintf(`
package main

// Import embedded schemas
%s

%s

%s

// Input configuration (to be validated)
input: #McpApplication
`, 
		e.wrapSchema("core", schema.CoreManifestSchema),
		e.wrapSchema("core", schema.CoreRegistrySchema),
		e.wrapSchema("solutions", schema.MCPSolutionSchema),
	)
	
	value := e.ctx.CompileString(validationCue)
	if err := value.Err(); err != nil {
		return cue.Value{}, fmt.Errorf("failed to compile validation schema: %w", err)
	}
	
	return value, nil
}

// wrapSchema wraps a schema string with package declaration
func (e *Engine) wrapSchema(packageName, schemaContent string) string {
	// Remove package declaration and imports from schema content
	lines := strings.Split(schemaContent, "\n")
	var filteredLines []string
	inImport := false
	
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		
		// Skip package declaration
		if strings.HasPrefix(trimmed, "package ") {
			continue
		}
		
		// Handle import blocks
		if strings.HasPrefix(trimmed, "import (") {
			inImport = true
			continue
		}
		if inImport && trimmed == ")" {
			inImport = false
			continue
		}
		if inImport {
			continue
		}
		
		// Skip single-line imports
		if strings.HasPrefix(trimmed, "import ") && !strings.Contains(trimmed, "(") {
			continue
		}
		
		filteredLines = append(filteredLines, line)
	}
	
	return strings.Join(filteredLines, "\n")
}

// marshalToTOML converts a map to TOML format using YAML as intermediate
func (e *Engine) marshalToTOML(data map[string]interface{}) ([]byte, error) {
	// First convert to YAML
	yamlData, err := yaml.Marshal(data)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal to YAML: %w", err)
	}
	
	// Parse YAML and convert to TOML-like format
	// This is a simplified TOML conversion - for production use, consider using a proper TOML library
	return e.convertYAMLToTOML(yamlData)
}

// convertYAMLToTOML converts YAML to TOML format
func (e *Engine) convertYAMLToTOML(yamlData []byte) ([]byte, error) {
	var data map[string]interface{}
	if err := yaml.Unmarshal(yamlData, &data); err != nil {
		return nil, err
	}
	
	var result strings.Builder
	
	// Process application section
	if app, ok := data["application"].(map[string]interface{}); ok {
		result.WriteString("[application]\n")
		for k, v := range app {
			result.WriteString(fmt.Sprintf("%s = %q\n", k, v))
		}
		result.WriteString("\n")
	}
	
	// Process variables section
	if vars, ok := data["variables"].(map[string]interface{}); ok {
		for name, varData := range vars {
			result.WriteString(fmt.Sprintf("[[variables]]\n"))
			result.WriteString(fmt.Sprintf("name = %q\n", name))
			if varMap, ok := varData.(map[string]interface{}); ok {
				for k, v := range varMap {
					result.WriteString(fmt.Sprintf("%s = %q\n", k, v))
				}
			}
			result.WriteString("\n")
		}
	}
	
	// Process components section
	if components, ok := data["component"].(map[string]interface{}); ok {
		for name, compData := range components {
			result.WriteString(fmt.Sprintf("[[component]]\n"))
			result.WriteString(fmt.Sprintf("id = %q\n", name))
			if compMap, ok := compData.(map[string]interface{}); ok {
				for k, v := range compMap {
					switch k {
					case "variables", "environment":
						if subMap, ok := v.(map[string]interface{}); ok {
							for subK, subV := range subMap {
								result.WriteString(fmt.Sprintf("[component.%s]\n", k))
								result.WriteString(fmt.Sprintf("%s = %q\n", subK, subV))
							}
						}
					case "allowed_outbound_hosts":
						if hosts, ok := v.([]interface{}); ok {
							result.WriteString(fmt.Sprintf("allowed_outbound_hosts = ["))
							for i, host := range hosts {
								if i > 0 {
									result.WriteString(", ")
								}
								result.WriteString(fmt.Sprintf("%q", host))
							}
							result.WriteString("]\n")
						}
					default:
						result.WriteString(fmt.Sprintf("%s = %q\n", k, v))
					}
				}
			}
			result.WriteString("\n")
		}
	}
	
	// Process trigger section
	if triggers, ok := data["trigger"].(map[string]interface{}); ok {
		if httpTriggers, ok := triggers["http"].([]interface{}); ok {
			for _, triggerData := range httpTriggers {
				if triggerMap, ok := triggerData.(map[string]interface{}); ok {
					result.WriteString("[[trigger.http]]\n")
					for k, v := range triggerMap {
						result.WriteString(fmt.Sprintf("%s = %q\n", k, v))
					}
					result.WriteString("\n")
				}
			}
		}
	}
	
	return []byte(result.String()), nil
}