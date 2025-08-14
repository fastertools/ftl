package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/spindl/internal/synth"
)

var synthCmd = &cobra.Command{
	Use:   "synth [input-file]",
	Short: "Synthesize spin.toml from configuration",
	Long: `Synthesize a Spin application manifest (spin.toml) from a high-level configuration.

The synth command takes your spindl configuration and generates a complete
Spin application manifest that can be used with 'spin up' or 'spin deploy'.`,
	Args: cobra.MaximumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		inputFile := "spindl.yml"
		if len(args) > 0 {
			inputFile = args[0]
		}
		
		outputFile, _ := cmd.Flags().GetString("output")
		environment, _ := cmd.Flags().GetString("env")
		setVars, _ := cmd.Flags().GetStringArray("set")
		overlayFiles, _ := cmd.Flags().GetStringArray("overlay")
		
		return runSynth(inputFile, outputFile, environment, setVars, overlayFiles)
	},
}

func init() {
	synthCmd.Flags().StringP("output", "o", "", "Output file (defaults to spin.toml)")
	synthCmd.Flags().StringP("env", "e", "", "Environment/stack to use")
	synthCmd.Flags().StringArrayP("set", "s", nil, "Set configuration values (can be used multiple times)")
	synthCmd.Flags().StringArray("overlay", nil, "Overlay configuration files to merge (can be used multiple times)")
}

func runSynth(inputFile, outputFile, environment string, setVars []string, overlayFiles []string) error {
	printInfo("Synthesizing %s", inputFile)
	
	// Read input file
	configData, err := os.ReadFile(inputFile)
	if err != nil {
		printError("Failed to read %s: %v", inputFile, err)
		return err
	}
	
	// Determine input format
	format := determineFormat(inputFile)
	
	// Apply set variables (simple implementation)
	if len(setVars) > 0 {
		configData, err = applySetVariables(configData, setVars, format)
		if err != nil {
			printError("Failed to apply set variables: %v", err)
			return err
		}
	}
	
	// TODO: Apply environment-specific configuration
	if environment != "" {
		printWarning("Environment-specific configuration not yet implemented")
	}
	
	// TODO: Apply overlay configurations
	if len(overlayFiles) > 0 {
		printWarning("Overlay configuration not yet fully implemented")
		// This will merge overlay configs with the base config
		// Priority: base < overlay1 < overlay2 < ... < set-vars
	}
	
	// Create synthesis engine
	engine := synth.NewEngine()
	
	// Synthesize configuration
	manifestData, err := engine.SynthesizeConfig(configData, format)
	if err != nil {
		printError("Synthesis failed: %v", err)
		return err
	}
	
	// Determine output file
	if outputFile == "" {
		outputFile = "spin.toml"
	}
	
	// Write output
	if err := os.WriteFile(outputFile, manifestData, 0644); err != nil {
		printError("Failed to write %s: %v", outputFile, err)
		return err
	}
	
	printSuccess("Generated %s", outputFile)
	
	// Print next steps
	fmt.Println()
	printInfo("Next steps:")
	fmt.Printf("  1. %s\n", infoColor.Sprint("spin up"))
	fmt.Printf("  2. %s\n", infoColor.Sprint("spin deploy"))
	
	return nil
}

// determineFormat determines the input format from file extension
func determineFormat(filename string) string {
	ext := strings.ToLower(filepath.Ext(filename))
	switch ext {
	case ".yaml", ".yml":
		return "yaml"
	case ".json":
		return "json"
	case ".toml":
		return "toml"
	case ".cue":
		return "cue"
	default:
		return "yaml" // Default to YAML
	}
}

// applySetVariables applies --set variables to the configuration
func applySetVariables(configData []byte, setVars []string, format string) ([]byte, error) {
	// This is a simplified implementation
	// In a production system, you might want to use a more sophisticated approach
	// such as JSONPath or YAMLPath for nested value setting
	
	if format != "yaml" {
		return configData, fmt.Errorf("--set is currently only supported for YAML files")
	}
	
	configStr := string(configData)
	
	for _, setVar := range setVars {
		parts := strings.SplitN(setVar, "=", 2)
		if len(parts) != 2 {
			return nil, fmt.Errorf("invalid --set format: %s (expected key=value)", setVar)
		}
		
		key, value := parts[0], parts[1]
		
		// Simple key replacement for top-level keys
		// This is a basic implementation - for production, use a proper YAML manipulation library
		if strings.Contains(configStr, key+":") {
			// Find and replace the line
			lines := strings.Split(configStr, "\n")
			for i, line := range lines {
				if strings.HasPrefix(strings.TrimSpace(line), key+":") {
					lines[i] = fmt.Sprintf("%s: %s", key, value)
					break
				}
			}
			configStr = strings.Join(lines, "\n")
		} else {
			// Add new key at the end
			configStr += fmt.Sprintf("\n%s: %s", key, value)
		}
	}
	
	return []byte(configStr), nil
}