package cmd

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
	"github.com/spf13/cobra"
	"gopkg.in/yaml.v3"
)

// synthCmd represents the synth command
func newSynthCmd() *cobra.Command {
	var outputFile string

	cmd := &cobra.Command{
		Use:   "synth [file]",
		Short: "Synthesize a spin.toml from FTL configuration",
		Long: `Synthesize a spin.toml manifest from an FTL configuration file.

Supports Go, YAML, JSON, and CUE input formats.

Examples:
  # Synthesize from Go file
  ftl synth platform.go

  # Synthesize from YAML
  ftl synth platform.yaml

  # Write to file
  ftl synth platform.yaml -o spin.toml

  # Synthesize from stdin (YAML/JSON only)
  cat platform.yaml | ftl synth -`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var input []byte
			var err error

			// Read input
			if len(args) == 0 || args[0] == "-" {
				// Read from stdin
				input, err = io.ReadAll(os.Stdin)
				if err != nil {
					return fmt.Errorf("failed to read from stdin: %w", err)
				}
			} else {
				// Read from file
				input, err = os.ReadFile(args[0])
				if err != nil {
					return fmt.Errorf("failed to read file: %w", err)
				}
			}

			// Detect format and synthesize
			manifest, err := synthesizeFromInput(input, args)
			if err != nil {
				return fmt.Errorf("synthesis failed: %w", err)
			}

			// Output result
			if outputFile != "" {
				err = os.WriteFile(outputFile, []byte(manifest), 0644)
				if err != nil {
					return fmt.Errorf("failed to write output file: %w", err)
				}
				fmt.Printf("Generated %s\n", outputFile)
			} else {
				fmt.Print(manifest)
			}

			return nil
		},
	}

	cmd.Flags().StringVarP(&outputFile, "output", "o", "", "Output file (default: stdout)")

	return cmd
}

// synthesizeFromInput detects the format and synthesizes accordingly
func synthesizeFromInput(input []byte, args []string) (string, error) {
	// Detect format based on file extension or content
	var format string
	var filename string
	if len(args) > 0 && args[0] != "-" {
		filename = args[0]
		ext := strings.ToLower(filepath.Ext(filename))
		switch ext {
		case ".go":
			format = "go"
		case ".yaml", ".yml":
			format = "yaml"
		case ".json":
			format = "json"
		case ".cue":
			format = "cue"
		default:
			format = detectFormat(input)
		}
	} else {
		format = detectFormat(input)
	}

	switch format {
	case "go":
		return synthesizeFromGo(filename)
	case "yaml":
		return synthesizeFromYAML(input)
	case "json":
		return synthesizeFromJSON(input)
	case "cue":
		return synthesizeFromCUE(input)
	default:
		return "", fmt.Errorf("unable to detect input format")
	}
}

// detectFormat tries to detect the format from content
func detectFormat(input []byte) string {
	trimmed := strings.TrimSpace(string(input))
	if strings.HasPrefix(trimmed, "{") {
		return "json"
	}
	if strings.HasPrefix(trimmed, "package ") {
		return "cue"
	}
	// Default to YAML
	return "yaml"
}

// synthesizeFromYAML converts YAML to spin.toml
func synthesizeFromYAML(input []byte) (string, error) {
	// Use CUE-first synthesizer for direct YAML processing
	synth := synthesis.NewSynthesizer()
	return synth.SynthesizeYAML(input)
}

// synthesizeFromJSON converts JSON to spin.toml
func synthesizeFromJSON(input []byte) (string, error) {
	// Use CUE-first synthesizer for direct JSON processing
	synth := synthesis.NewSynthesizer()
	return synth.SynthesizeJSON(input)
}

// synthesizeFromGo runs a Go file and captures its output
func synthesizeFromGo(filename string) (string, error) {
	// Get absolute path to ensure the file can be found
	absPath, err := filepath.Abs(filename)
	if err != nil {
		return "", fmt.Errorf("failed to get absolute path: %w", err)
	}

	// Check if file exists
	if _, err := os.Stat(absPath); err != nil {
		return "", fmt.Errorf("file not found: %w", err)
	}

	// Run the Go file and capture its output
	cmd := exec.Command("go", "run", absPath)

	// Set the working directory to the file's directory
	// This ensures relative imports work correctly
	cmd.Dir = filepath.Dir(absPath)

	// Capture stdout
	output, err := cmd.Output()
	if err != nil {
		// Try to get error details if available
		if exitErr, ok := err.(*exec.ExitError); ok {
			return "", fmt.Errorf("failed to run Go file: %w\nstderr: %s", err, exitErr.Stderr)
		}
		return "", fmt.Errorf("failed to run Go file: %w", err)
	}

	// The Go program should output the manifest to stdout
	manifest := string(output)

	// Basic validation - check if it looks like a manifest
	if !strings.Contains(manifest, "spin_manifest_version") {
		return "", fmt.Errorf("Go program did not output a valid spin.toml manifest")
	}

	return manifest, nil
}

// synthesizeFromCUE converts CUE to spin.toml
func synthesizeFromCUE(input []byte) (string, error) {
	// Use CUE-first synthesizer
	synth := synthesis.NewSynthesizer()
	return synth.SynthesizeCUE(string(input))
}

// synthesizeFromFTLConfig converts the FTL config struct to spin.toml
func synthesizeFromFTLConfig(cfg interface{}) (string, error) {
	// Convert the config to YAML and use CUE-first synthesizer
	// This allows us to handle configs from Go while still using CUE
	yamlData, err := yaml.Marshal(cfg)
	if err != nil {
		return "", fmt.Errorf("failed to marshal config: %w", err)
	}

	synth := synthesis.NewSynthesizer()
	return synth.SynthesizeYAML(yamlData)
}
