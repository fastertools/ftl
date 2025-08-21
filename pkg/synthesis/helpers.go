package synthesis

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"gopkg.in/yaml.v3"
)

// SynthesizeFromConfig reads a config file and synthesizes it to a Spin manifest
func SynthesizeFromConfig(configPath string) (string, error) {
	// Clean the path to prevent directory traversal
	configPath = filepath.Clean(configPath)
	// Read the config file
	data, err := os.ReadFile(configPath)
	if err != nil {
		return "", fmt.Errorf("failed to read config file: %w", err)
	}

	// Detect format based on extension
	ext := strings.ToLower(filepath.Ext(configPath))
	synth := NewSynthesizer()

	switch ext {
	case ".yaml", ".yml":
		return synth.SynthesizeYAML(data)
	case ".json":
		return synth.SynthesizeJSON(data)
	case ".cue":
		return synth.SynthesizeCUE(string(data))
	case ".go":
		// For Go files, we need to run them to generate the manifest
		return runGoConfig(configPath)
	default:
		// Try to detect based on content
		var yamlTest interface{}
		if err := yaml.Unmarshal(data, &yamlTest); err == nil {
			return synth.SynthesizeYAML(data)
		}
		return "", fmt.Errorf("unsupported config format for file: %s", configPath)
	}
}

// runGoConfig runs a Go configuration file and captures its output
func runGoConfig(goFile string) (string, error) {
	// Check if file exists
	if _, err := os.Stat(goFile); err != nil {
		return "", fmt.Errorf("go config file not found: %w", err)
	}

	// Run the Go file and capture output
	cmd := exec.Command("go", "run", goFile)
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	if err := cmd.Run(); err != nil {
		return "", fmt.Errorf("failed to run Go config: %w\nstderr: %s", err, stderr.String())
	}

	// The Go program should output the spin.toml manifest
	manifest := stdout.String()
	if manifest == "" {
		return "", fmt.Errorf("go config produced no output")
	}

	return manifest, nil
}
