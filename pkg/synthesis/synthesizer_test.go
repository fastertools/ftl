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
