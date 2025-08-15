package cmd

import (
	"context"
	"fmt"
	"os"

	"github.com/fastertools/ftl-cli/go/ftl/pkg/synthesis"
	"github.com/fastertools/ftl-cli/go/shared/spin"
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

			// Auto-detect config file if not specified
			if configFile == "" {
				// Try to detect the config format
				if _, err := os.Stat("ftl.yaml"); err == nil {
					configFile = "ftl.yaml"
				} else if _, err := os.Stat("ftl.json"); err == nil {
					configFile = "ftl.json"
				} else if _, err := os.Stat("app.cue"); err == nil {
					configFile = "app.cue"
				} else if _, err := os.Stat("main.go"); err == nil {
					configFile = "main.go"
				}
			}

			// Check if config file exists
			if configFile != "" && !skipSynth {
				if _, err := os.Stat(configFile); err == nil {
				fmt.Printf("%s Synthesizing spin.toml from %s\n", blue("→"), configFile)

				// Use unified synthesis helper
				manifest, err := synthesis.SynthesizeFromConfig(configFile)
				if err != nil {
					return fmt.Errorf("synthesis failed: %w", err)
				}

				// Write spin.toml
				if err := os.WriteFile("spin.toml", []byte(manifest), 0644); err != nil {
					return fmt.Errorf("failed to write spin.toml: %w", err)
				}

				fmt.Printf("%s Generated spin.toml\n", green("✓"))
				}
			} else if configFile == "" && !skipSynth {
				// No config file found, check for spin.toml
				if _, err := os.Stat("spin.toml"); os.IsNotExist(err) {
					return fmt.Errorf("no ftl.yaml, ftl.json, app.cue, or spin.toml found. Run 'ftl init' first")
				}
				fmt.Printf("%s No FTL config found, using existing spin.toml\n", yellow("ℹ"))
			} else if skipSynth {
				// When skipping synthesis, just check if spin.toml exists
				if _, err := os.Stat("spin.toml"); os.IsNotExist(err) {
					return fmt.Errorf("no spin.toml found. Run 'ftl synth' or 'ftl build' without --skip-synth first")
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

	cmd.Flags().BoolVar(&skipSynth, "skip-synth", false, "Skip synthesis of spin.toml from FTL config")
	cmd.Flags().StringVarP(&configFile, "config", "c", "", "Configuration file to synthesize (auto-detects if not specified)")

	return cmd
}
