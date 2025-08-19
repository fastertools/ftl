package synthesis

import (
	"fmt"
	"os"
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
	default:
		// Try to detect based on content
		var yamlTest interface{}
		if err := yaml.Unmarshal(data, &yamlTest); err == nil {
			return synth.SynthesizeYAML(data)
		}
		return "", fmt.Errorf("unsupported config format for file: %s", configPath)
	}
}
