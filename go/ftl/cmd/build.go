package cmd

import (
	"context"
	"fmt"
	"os"

	"github.com/fastertools/ftl-cli/go/shared/spin"
	"github.com/fastertools/ftl-cli/go/spindl/pkg/spindl"
	"github.com/fatih/color"
	"github.com/spf13/cobra"
)

func newBuildCmd() *cobra.Command {
	var skipSynth bool
	var configFile string

	cmd := &cobra.Command{
		Use:   "build",
		Short: "Build the FTL application",
		Long:  `Build compiles the FTL application and its components.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := context.Background()

			// Color helpers
			blue := color.New(color.FgBlue).SprintFunc()
			green := color.New(color.FgGreen).SprintFunc()
			yellow := color.New(color.FgYellow).SprintFunc()

			// Ensure spin is installed
			if err := spin.EnsureInstalled(); err != nil {
				return err
			}

			// Check if spindl.yml exists
			if _, err := os.Stat(configFile); err == nil && !skipSynth {
				fmt.Printf("%s Synthesizing spin.toml from %s\n", blue("→"), configFile)

				// Read spindl.yml
				configData, err := os.ReadFile(configFile)
				if err != nil {
					return fmt.Errorf("failed to read %s: %w", configFile, err)
				}

				// Create synthesis engine
				engine := spindl.NewEngine()

				// Synthesize to spin.toml
				manifestData, err := engine.SynthesizeConfig(configData, "yaml")
				if err != nil {
					return fmt.Errorf("synthesis failed: %w", err)
				}

				// Write spin.toml
				if err := os.WriteFile("spin.toml", manifestData, 0644); err != nil {
					return fmt.Errorf("failed to write spin.toml: %w", err)
				}

				fmt.Printf("%s Generated spin.toml\n", green("✓"))
			} else if err != nil && !os.IsNotExist(err) {
				return fmt.Errorf("failed to check %s: %w", configFile, err)
			} else if os.IsNotExist(err) {
				// Check if spin.toml exists directly
				if _, err := os.Stat("spin.toml"); os.IsNotExist(err) {
					return fmt.Errorf("neither %s nor spin.toml found. Run 'ftl init' first", configFile)
				}
				fmt.Printf("%s Using existing spin.toml\n", yellow("ℹ"))
			}

			fmt.Printf("%s Building FTL application...\n", blue("→"))

			// Use spin build
			if err := spin.Build(ctx); err != nil {
				return fmt.Errorf("failed to build: %w", err)
			}

			fmt.Printf("%s Build completed successfully\n", green("✓"))
			return nil
		},
	}

	cmd.Flags().BoolVar(&skipSynth, "skip-synth", false, "Skip synthesis of spin.toml from spindl.yml")
	cmd.Flags().StringVarP(&configFile, "config", "c", "spindl.yml", "Configuration file to synthesize")

	return cmd
}
