package synthesis

import (
	"strings"
	"testing"
)

func TestSynthesizer_DirectYAML(t *testing.T) {
	yamlInput := `
name: yaml-app
version: "1.0.0"
description: Test YAML app
components:
  - id: tool1
    source: ./tool1.wasm
    variables:
      LOG_LEVEL: debug
access: public
`

	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeYAML([]byte(yamlInput))
	if err != nil {
		t.Fatalf("Failed to synthesize from YAML: %v", err)
	}

	// Debug
	t.Logf("Generated manifest:\n%s", manifest)

	// Verify the manifest contains expected elements
	if !strings.Contains(manifest, "spin_manifest_version = 2") {
		t.Error("Missing spin manifest version")
	}

	if !strings.Contains(manifest, "yaml-app") {
		t.Error("Missing application name from YAML")
	}

	if !strings.Contains(manifest, "[component.tool1]") {
		t.Error("Missing tool1 component")
	}
}

func TestSynthesizer_DirectJSON(t *testing.T) {
	jsonInput := `{
		"name": "json-app",
		"version": "2.0.0",
		"description": "Test JSON app",
		"components": [
			{
				"id": "tool2",
				"source": "./tool2.wasm",
				"variables": {
					"API_KEY": "secret"
				}
			}
		],
		"access": "public"
	}`

	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeJSON([]byte(jsonInput))
	if err != nil {
		t.Fatalf("Failed to synthesize from JSON: %v", err)
	}

	// Verify the manifest contains expected elements
	if !strings.Contains(manifest, "json-app") {
		t.Error("Missing application name from JSON")
	}

	if !strings.Contains(manifest, "[component.tool2]") {
		t.Error("Missing tool2 component")
	}
}

func TestSynthesizer_DirectCUE(t *testing.T) {
	cueSource := `
name: "cue-app"
version: "1.0.0"
components: [{
	id: "cue-component"
	source: "./component.wasm"
}]
access: "public"
`

	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeCUE(cueSource)
	if err != nil {
		t.Fatalf("Failed to synthesize from CUE: %v", err)
	}

	// Verify the manifest contains expected elements
	if !strings.Contains(manifest, "cue-app") {
		t.Error("Missing application name from CUE")
	}

	if !strings.Contains(manifest, "[component.cue-component]") {
		t.Error("Missing component from CUE")
	}
}

func TestSynthesizer_WithOverrides(t *testing.T) {
	synth := NewSynthesizer()

	// Test data with private access (requires authorizer)
	app := map[string]interface{}{
		"name":   "override-app",
		"access": "private",
		"components": []interface{}{
			map[string]interface{}{
				"id": "api",
				"source": map[string]interface{}{
					"registry": "ghcr.io",
					"package":  "test/api",
					"version":  "v2.0.0",
				},
			},
		},
	}

	// Platform overrides for component versions
	overrides := map[string]interface{}{
		"gateway_version":    "0.0.14-beta.1",
		"authorizer_version": "0.0.16-beta.2",
	}

	result, err := synth.SynthesizeWithOverrides(app, overrides)
	if err != nil {
		t.Fatalf("SynthesizeWithOverrides failed: %v", err)
	}

	// Check that overridden versions are used
	if !strings.Contains(result, "0.0.14-beta.1") {
		t.Error("Result should contain overridden gateway version")
	}
	if !strings.Contains(result, "0.0.16-beta.2") {
		t.Error("Result should contain overridden authorizer version")
	}
	if !strings.Contains(result, "mcp-gateway") {
		t.Error("Result should contain gateway")
	}
	if !strings.Contains(result, "mcp-authorizer") {
		t.Error("Result should contain authorizer for private app")
	}
}
