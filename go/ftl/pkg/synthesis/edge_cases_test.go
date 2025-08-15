package synthesis

import (
	"cuelang.org/go/cue"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// TestEdgeCases covers error paths and boundary conditions
func TestEdgeCases(t *testing.T) {
	t.Run("CDK with nil app", func(t *testing.T) {
		cdk := &CDK{}
		err := cdk.ValidateWithSchema("schema")
		if err == nil {
			t.Error("Should fail with nil app")
		}
		if !strings.Contains(err.Error(), "no application defined") {
			t.Errorf("Wrong error message: %v", err)
		}
	})

	t.Run("Synthesize with nil app", func(t *testing.T) {
		builtCDK := &CDK{}
		_, err := builtCDK.Synthesize()
		if err == nil {
			t.Error("Should fail with nil app")
		}
	})

	t.Run("ToCUE with nil app", func(t *testing.T) {
		builtCDK := &CDK{}
		_, err := builtCDK.ToCUE()
		if err == nil {
			t.Error("Should fail with nil app")
		}
	})

	t.Run("WithWatch without build", func(t *testing.T) {
		cdk := NewCDK()
		app := cdk.NewApp("test-app")

		// Try WithWatch without calling WithBuild first
		builder := app.AddComponent("comp").
			FromLocal("./comp.wasm").
			WithWatch("*.rs", "*.toml")

		// Should still work - WithWatch creates build if needed
		if builder.component.Build == nil {
			t.Error("WithWatch should create build config if missing")
		}
	})

	t.Run("YAML with no extension detection", func(t *testing.T) {
		// Test file with no extension but YAML content
		tmpDir := t.TempDir()
		noExtPath := filepath.Join(tmpDir, "config")

		yamlContent := `
application:
  name: noext-test
  version: "1.0.0"
components:
  - id: comp1
    source: ./comp1.wasm
`
		if err := os.WriteFile(noExtPath, []byte(yamlContent), 0644); err != nil {
			t.Fatalf("Failed to write test file: %v", err)
		}

		// This should detect as YAML from content
		manifest, err := SynthesizeFromConfig(noExtPath)
		if err != nil {
			t.Fatalf("Failed to synthesize: %v", err)
		}
		if !strings.Contains(manifest, "noext-test") {
			t.Error("Should parse as YAML")
		}
	})

	t.Run("Error cases in synthesizers", func(t *testing.T) {
		synth := NewSynthesizer()

		// Test YAML extraction error
		_, err := synth.SynthesizeYAML([]byte(""))
		if err == nil {
			t.Error("Should fail with empty YAML")
		}

		// Test JSON extraction error
		_, err = synth.SynthesizeJSON([]byte(""))
		if err == nil {
			t.Error("Should fail with empty JSON")
		}

		// Test CUE compilation error
		_, err = synth.SynthesizeCUE("")
		if err == nil {
			t.Error("Should fail with empty CUE")
		}
	})

	t.Run("encodeToTOML error", func(t *testing.T) {
		synth := NewSynthesizer()
		// Create a value that can't be encoded to TOML
		value := synth.ctx.CompileString(`
			invalid: _|_
		`)
		_, err := synth.encodeToTOML(value.LookupPath(cue.ParsePath("invalid")))
		if err == nil {
			t.Error("Should fail encoding invalid value")
		}
	})

	t.Run("synthesizeFromValue compilation error", func(t *testing.T) {
		synth := NewSynthesizer()
		// Create a value that will cause compilation error
		value := synth.ctx.CompileString(`invalid: _|_`)
		_, err := synth.synthesizeFromValue(value)
		if err == nil {
			t.Error("Should fail with invalid value")
		}
	})

	t.Run("synthesizeFromValue fill error", func(t *testing.T) {
		synth := NewSynthesizer()
		// Create a value that can't be filled properly
		value := synth.ctx.CompileString(`{invalidStructure: 123}`)
		_, err := synth.synthesizeFromValue(value)
		if err == nil {
			t.Error("Should fail with incompatible structure")
		}
		// The error should be about filling or extraction
		if !strings.Contains(err.Error(), "fill") && !strings.Contains(err.Error(), "extract") && !strings.Contains(err.Error(), "encode") {
			t.Errorf("Unexpected error: %v", err)
		}
	})

	t.Run("synthesizeFromValue manifest extraction error", func(t *testing.T) {
		synth := NewSynthesizer()
		// Create valid input but that produces invalid manifest
		yamlData := []byte(`
application:
  name: ""  # Empty name should fail validation
  version: "1.0.0"
`)
		_, err := synth.SynthesizeYAML(yamlData)
		// Empty name should fail CUE validation
		if err == nil {
			t.Error("Should fail with empty app name")
		}
	})
}

// TestCuePatterns verifies the embedded CUE patterns are valid
func TestCuePatterns(t *testing.T) {
	// Verify required imports are present
	if !strings.Contains(ftlPatterns, "\"strings\"") {
		t.Error("CUE patterns should import strings package")
	}

	// Verify key pattern definitions exist
	if !strings.Contains(ftlPatterns, "#FTLApplication") {
		t.Error("CUE patterns should define #FTLApplication")
	}

	if !strings.Contains(ftlPatterns, "#TransformToSpin") {
		t.Error("CUE patterns should define #TransformToSpin")
	}
}
