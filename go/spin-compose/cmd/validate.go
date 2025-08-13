package cmd

import (
	"os"

	"github.com/spf13/cobra"

	"github.com/fastertools/ftl-cli/go/spin-compose/internal/synth"
)

var validateCmd = &cobra.Command{
	Use:   "validate [input-file]",
	Short: "Validate configuration against schema",
	Long: `Validate a spin-compose configuration file against the schema.

The validate command checks your configuration for syntax errors, type mismatches,
and required field violations without generating output files.`,
	Args: cobra.MaximumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		inputFile := "spinc.yaml"
		if len(args) > 0 {
			inputFile = args[0]
		}
		
		return runValidate(inputFile)
	},
}

func runValidate(inputFile string) error {
	printInfo("Validating %s", inputFile)
	
	// Read input file
	configData, err := os.ReadFile(inputFile)
	if err != nil {
		printError("Failed to read %s: %v", inputFile, err)
		return err
	}
	
	// Determine input format
	format := determineFormat(inputFile)
	
	// Create synthesis engine
	engine := synth.NewEngine()
	
	// Validate configuration
	if err := engine.ValidateConfig(configData, format); err != nil {
		printError("Validation failed:")
		printError("%v", err)
		return err
	}
	
	printSuccess("Configuration is valid")
	return nil
}